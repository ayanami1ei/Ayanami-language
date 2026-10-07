//! `match` 降级（Phase 1.3）：字面量 / `_` / 绑定 / 枚举变体 / 或模式 + guard。
//! 结构：求值目标 → 逐臂顺序 `if !matched && cond`（guard 走臂内嵌套 if，支持 fallthrough）。
use super::*;
use crate::parser::ast::pattern::Pattern;
use crate::parser::ast::stmt::MatchArm;

struct LoweredArm {
    cond: Option<HirNodeBox>,
    bindings: Vec<HirStmt>,
    guard: Option<HirNodeBox>,
    body: HirNodeBox,
}

impl crate::hir::lower::Ctx {
    /// 求值 match 目标 → 临时局部；返回 (值节点, 值类型)
    fn setup_match_value(&mut self, value: &Expr, span: &Span) -> Result<(HirNodeBox, HirType)> {
        let hir_value = auto_deref(self.lower_expr(value)?);
        let value_ty = expr_type(&hir_value);
        let base = strip_ownership(value_ty.clone());
        if !matches!(base,
            HirType::Named(_) | HirType::Int | HirType::IntN { .. } | HirType::Char
            | HirType::Bool | HirType::Float | HirType::F32)
        {
            return Err(Error::Hir(format!(
                "match on unsupported type `{}` at {}:{}",
                hir_type_display(&value_ty), span.start_line, span.start_col
            )));
        }
        let val_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_val"), value_ty.clone(), false));
        let val_node: HirNodeBox = SVar { var: val_var, ty: value_ty.clone() }.into();
        self.pending_stmts.push(HirStmt::Assign {
            target: val_node.clone(),
            value: hir_value,
            span: *span,
        });
        Ok((val_node, value_ty))
    }

    /// 模式归一化：枚举 scrutinee 下裸标识符若匹配变体名 → 无载荷变体模式
    fn normalize_pattern(&self, p: &Pattern, val_ty: &HirType) -> Pattern {
        match p {
            Pattern::Binding(n) => {
                if let HirType::Named(en) = &strip_ownership(val_ty.clone()) {
                    if self.variant_tag(en, n).is_some() {
                        return Pattern::Enum { name: *n, args: Vec::new() };
                    }
                }
                p.clone()
            }
            Pattern::Or(ps) => Pattern::Or(ps.iter().map(|x| self.normalize_pattern(x, val_ty)).collect()),
            other => other.clone(),
        }
    }

    fn lower_arm_body(&mut self, arm: &MatchArm) -> Result<(Vec<HirStmt>, HirNodeBox)> {
        let saved = std::mem::take(&mut self.pending_stmts);
        let body = self.lower_expr(&arm.body);
        let arm_pending = std::mem::replace(&mut self.pending_stmts, saved);
        Ok((arm_pending, body?))
    }

    /// 逐臂降级（绑定/guard/body 在臂作用域内解析）
    fn lower_match_arms(
        &mut self,
        val: &HirNodeBox,
        val_ty: &HirType,
        arms: &[MatchArm],
        span: &Span,
    ) -> Result<Vec<LoweredArm>> {
        let mut out = Vec::new();
        for arm in arms {
            let pat = self.normalize_pattern(&arm.pattern, val_ty);
            self.push_scope();
            let (cond, mut bindings) = self.pattern_match(&pat, val, val_ty, span)?;
            let guard = match &arm.guard {
                Some(g) => Some(auto_deref(self.lower_expr(g)?)),
                None => None,
            };
            let (mut pending, body) = self.lower_arm_body(arm)?;
            self.pop_scope();
            bindings.append(&mut pending);
            out.push(LoweredArm { cond, bindings, guard, body });
        }
        Ok(out)
    }

    /// 穷尽性：不可反驳臂（无 guard）或枚举全变体 / bool 双值覆盖
    fn is_exhaustive(&self, val_ty: &HirType, arms: &[MatchArm]) -> bool {
        let pats: Vec<Pattern> = arms.iter().map(|a| self.normalize_pattern(&a.pattern, val_ty)).collect();
        if arms.iter().zip(&pats).any(|(a, p)| a.guard.is_none() && p.is_irrefutable()) {
            return true;
        }
        match &strip_ownership(val_ty.clone()) {
            HirType::Named(en) => {
                let total = self.enum_variant_count(en);
                let mut covered = std::collections::HashSet::new();
                for (a, p) in arms.iter().zip(&pats) {
                    if a.guard.is_none() { collect_variants(p, &mut covered); }
                }
                total > 0 && covered.len() == total
            }
            HirType::Bool => {
                let mut t = false;
                let mut f = false;
                for (a, p) in arms.iter().zip(&pats) {
                    if a.guard.is_none() { collect_bools(p, &mut t, &mut f); }
                }
                t && f
            }
            _ => false,
        }
    }

    fn enum_variant_count(&self, name: &Symbol) -> usize {
        self.struct_defs.get(name)
            .map(|fields| fields.iter().filter(|f| f.name.as_str().starts_with("_data_")).count())
            .unwrap_or(0)
    }

    /// 顺序 if + matched 标志；`assign` 为表达式形式的结果变量
    fn build_match_stmts(
        &mut self,
        arms: &[LoweredArm],
        assign: Option<(&HirNodeBox, &HirType)>,
        span: &Span,
    ) -> Vec<HirStmt> {
        let mut stmts = Vec::new();
        if arms.is_empty() {
            return stmts;
        }
        let mvar = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_matched"), HirType::Bool, true));
        let mnode: HirNodeBox = SVar { var: mvar, ty: HirType::Bool }.into();
        let bool_const = |b: bool| -> HirNodeBox {
            SConst { val: HirLiteral::Bool(b), ty: HirType::Bool }.into()
        };
        stmts.push(HirStmt::Assign { target: mnode.clone(), value: bool_const(false), span: *span });
        for arm in arms {
            let mut block_stmts = arm.bindings.clone();
            let mut done = vec![
                match assign {
                    Some((res, _)) => HirStmt::Assign { target: res.clone(), value: arm.body.clone(), span: *span },
                    None => HirStmt::Expr { expr: arm.body.clone(), span: *span },
                },
                HirStmt::Assign { target: mnode.clone(), value: bool_const(true), span: *span },
            ];
            if let Some(g) = &arm.guard {
                // guard 失败 → matched 保持 false，后续臂继续匹配（fallthrough）
                block_stmts.push(HirStmt::If {
                    cond: g.clone(),
                    then_block: HirBlock::new(done),
                    elifs: Vec::new(),
                    else_block: None,
                    span: *span,
                });
            } else {
                block_stmts.append(&mut done);
            }
            let not_matched: HirNodeBox = SUn {
                op: crate::parser::ast::UnaryOp::Not,
                arg: mnode.clone(),
                ty: HirType::Bool,
            }.into();
            let cond = match &arm.cond {
                Some(c) => SBin {
                    op: crate::parser::ast::BinaryOp::And,
                    lhs: not_matched,
                    rhs: c.clone(),
                    ty: HirType::Bool,
                }.into(),
                None => not_matched,
            };
            stmts.push(HirStmt::If {
                cond,
                then_block: HirBlock::new(block_stmts),
                elifs: Vec::new(),
                else_block: None,
                span: *span,
            });
        }
        stmts
    }

    /// `match` 语句
    pub(crate) fn lower_match_stmt(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirStmt> {
        if arms.is_empty() {
            return Err(Error::Hir(format!("empty match at {}:{}", span.start_line, span.start_col)));
        }
        let (val_node, val_ty) = self.setup_match_value(value, span)?;
        if !self.is_exhaustive(&val_ty, arms) {
            return Err(Error::Hir(format!(
                "non-exhaustive match at {}:{} (add `_ =>` or cover all variants)",
                span.start_line, span.start_col
            )));
        }
        let lowered = self.lower_match_arms(&val_node, &val_ty, arms, span)?;
        let stmts = self.build_match_stmts(&lowered, None, span);
        Ok(HirStmt::Block { stmts, span: *span })
    }

    /// `match` 表达式（各分支赋值给结果临时变量）
    pub(crate) fn lower_match_expr(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirNodeBox> {
        if arms.is_empty() {
            return Err(Error::Hir(format!("empty match at {}:{}", span.start_line, span.start_col)));
        }
        let (val_node, val_ty) = self.setup_match_value(value, span)?;
        if !self.is_exhaustive(&val_ty, arms) {
            return Err(Error::Hir(format!(
                "non-exhaustive match at {}:{} (add `_ =>` or cover all variants)",
                span.start_line, span.start_col
            )));
        }
        let lowered = self.lower_match_arms(&val_node, &val_ty, arms, span)?;
        // M1.9：分支分类 —— Void / 值(ty) / 发散(`!`)
        let mut never: Vec<bool> = Vec::new();
        let mut value_tys: Vec<HirType> = Vec::new();
        let mut has_void = false;
        for a in &lowered {
            let ty = strip_ownership(expr_type(&a.body));
            if ty == HirType::Never { never.push(true); }
            else if ty == HirType::Void { never.push(false); has_void = true; }
            else { never.push(false); value_tys.push(ty); }
        }
        if has_void && !value_tys.is_empty() {
            return Err(Error::Hir(format!(
                "match branches have inconsistent types (void vs value) (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        // 无值分支：按语句求值；全发散 → 结果类型 `!`
        if value_tys.is_empty() {
            let stmts = self.build_match_stmts(&lowered, None, span);
            self.pending_stmts.extend(stmts);
            let ty = if never.iter().all(|d| *d) { HirType::Never } else { HirType::Void };
            return Ok(SConst { val: HirLiteral::Int(0), ty }.into());
        }
        let res_ty = value_tys.iter()
            .fold(None::<HirType>, |acc, t| Some(match acc {
                Some(a) => super::match_pattern::match_result_type(&a, t),
                None => t.clone(),
            }))
            .unwrap_or(HirType::Int);
        // #135：分支构造回退到枚举基名时，用当前函数返回类型细化
        let res_ty = self.refine_enum_result_type(res_ty);
        // 臂体按结果类型实例化/强转（#119/#120：无载荷变体按公共类型实例化）
        let mut lowered = lowered;
        for (i, a) in lowered.iter_mut().enumerate() {
            if never[i] { continue; }
            let body = std::mem::replace(&mut a.body, SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into());
            let body = self.instantiate_enum_value(body, &res_ty)?;
            a.body = coerce_expr(body, &res_ty, span)?;
        }
        let res_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_res"), res_ty.clone(), true));
        let res_node: HirNodeBox = SVar { var: res_var, ty: res_ty.clone() }.into();
        let stmts = self.build_match_stmts(&lowered, Some((&res_node, &res_ty)), span);
        self.pending_stmts.extend(stmts);
        Ok(res_node)
    }
}

fn collect_variants(p: &Pattern, out: &mut std::collections::HashSet<Symbol>) {
    match p {
        Pattern::Enum { name, .. } => { out.insert(*name); }
        Pattern::Or(ps) => for x in ps { collect_variants(x, out); },
        _ => {}
    }
}

fn collect_bools(p: &Pattern, t: &mut bool, f: &mut bool) {
    match p {
        Pattern::Literal(crate::parser::ast::Literal::Bool(b, _)) => { if *b { *t = true; } else { *f = true; } }
        Pattern::Or(ps) => for x in ps { collect_bools(x, t, f); },
        _ => {}
    }
}
