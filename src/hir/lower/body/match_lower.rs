//! `match` 降级：语句形式与表达式形式。
//! 值统一先存入临时局部（避免分支重复求值），分支体在独立作用域内绑定载荷。

use super::*;
use crate::parser::ast::stmt::MatchArm;

impl crate::hir::lower::Ctx {
    /// 求值 match 目标 → 临时局部；返回 (值节点, 枚举类型名)
    fn setup_match_value(&mut self, value: &Expr, span: &Span) -> Result<(HirNodeBox, Symbol)> {
        let hir_value = auto_deref(self.lower_expr(value)?);
        let value_ty = expr_type(&hir_value);
        let name = match &value_ty {
            HirType::Named(n) => *n,
            _ => return Err(Error::Hir(format!(
                "match on non-enum type at {}:{}", span.start_line, span.start_col
            ))),
        };
        let val_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_val"), value_ty.clone(), false));
        let val_node: HirNodeBox = SVar { var: val_var, ty: value_ty.clone() }.into();
        self.pending_stmts.push(HirStmt::Assign {
            target: val_node.clone(),
            value: hir_value,
            span: *span,
        });
        Ok((val_node, name))
    }

    fn tag_cond(val_node: &HirNodeBox, tag: usize) -> HirNodeBox {
        SBin {
            op: crate::parser::ast::BinaryOp::Eq,
            lhs: SField {
                object: val_node.clone(),
                field: Symbol::intern("_tag"),
                field_index: 0,
                ty: HirType::Int,
            }.into(),
            rhs: SConst { val: HirLiteral::Int(tag as i64), ty: HirType::Int }.into(),
            ty: HirType::Int,
        }.into()
    }

    /// 分支载荷绑定：`V(x)` → x = __match_val._data_V._0（类型取自变体结构体）
    fn match_arm_bindings(
        &mut self,
        arm: &MatchArm,
        val_node: &HirNodeBox,
        value_ty_name: Symbol,
        arm_index: usize,
        span: &Span,
    ) -> Result<Vec<HirStmt>> {
        let data_field = Symbol::intern(&format!("_data_{}", arm.variant_name));
        let var_struct = Symbol::intern(&format!("{}_{}", value_ty_name, arm.variant_name));
        let mut stmts = Vec::new();
        for (j, (bind_name, _)) in arm.bindings.iter().enumerate() {
            let inner_acc: HirNodeBox = SField {
                object: val_node.clone(),
                field: data_field,
                field_index: arm_index + 1,
                ty: HirType::Named(var_struct),
            }.into();
            let payload_field = Symbol::intern(&format!("_{}", j));
            let fty = self.find_field_type(&HirType::Named(var_struct), &payload_field, span)?;
            let fval: HirNodeBox = SField {
                object: inner_acc,
                field: payload_field,
                field_index: j,
                ty: fty.clone(),
            }.into();
            let (bid, _, _) = self.register_or_lookup(*bind_name, fty.clone());
            stmts.push(HirStmt::Assign {
                target: SVar { var: bid, ty: fty }.into(),
                value: fval,
                span: *span,
            });
        }
        Ok(stmts)
    }

    /// 分支体降级（保存/恢复 pending，保证内联语句留在分支内）
    fn lower_arm_body(&mut self, arm: &MatchArm) -> Result<(Vec<HirStmt>, HirNodeBox)> {
        let saved = std::mem::take(&mut self.pending_stmts);
        let body = self.lower_expr(&arm.body);
        let arm_pending = std::mem::replace(&mut self.pending_stmts, saved);
        Ok((arm_pending, body?))
    }

    /// 组装 If 链：最后一条分支仅在穷尽时作为 else
    fn build_match_if(
        &self,
        value_ty_name: Symbol,
        mut conds: Vec<HirNodeBox>,
        mut blocks: Vec<HirBlock>,
        span: &Span,
    ) -> Option<HirStmt> {
        if conds.is_empty() {
            return None;
        }
        let exhaustive = self.enum_variant_count(&value_ty_name) == conds.len();
        let first_cond = conds.remove(0);
        let first_block = blocks.remove(0);
        let else_block = if exhaustive && !blocks.is_empty() {
            blocks.pop()
        } else {
            None
        };
        let elifs: Vec<(HirNodeBox, HirBlock)> = conds.into_iter().zip(blocks).collect();
        Some(HirStmt::If {
            cond: first_cond,
            then_block: first_block,
            elifs,
            else_block,
            span: *span,
        })
    }

    fn enum_variant_count(&self, name: &Symbol) -> usize {
        self.struct_defs.get(name)
            .map(|fields| fields.iter().filter(|f| f.name.as_str().starts_with("_data_")).count())
            .unwrap_or(0)
    }

    /// `match` 语句
    pub(crate) fn lower_match_stmt(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirStmt> {
        let (val_node, value_ty_name) = self.setup_match_value(value, span)?;
        let mut conds = Vec::new();
        let mut blocks = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            conds.push(Self::tag_cond(&val_node, i));
            self.push_scope();
            let mut arm_stmts = self.match_arm_bindings(arm, &val_node, value_ty_name, i, span)?;
            let (mut pending, body) = self.lower_arm_body(arm)?;
            self.pop_scope();
            arm_stmts.append(&mut pending);
            arm_stmts.push(HirStmt::Expr { expr: body, span: *span });
            blocks.push(HirBlock { stmts: arm_stmts });
        }
        match self.build_match_if(value_ty_name, conds, blocks, span) {
            Some(if_stmt) => Ok(HirStmt::Block {
                stmts: vec![if_stmt],
                span: *span,
            }),
            None => Ok(HirStmt::Expr {
                expr: SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into(),
                span: *span,
            }),
        }
    }

    /// `match` 表达式（各分支赋值给结果临时变量）
    pub(crate) fn lower_match_expr(&mut self, value: &Expr, arms: &[MatchArm], span: &Span) -> Result<HirNodeBox> {
        let (val_node, value_ty_name) = self.setup_match_value(value, span)?;
        let mut lowered: Vec<(Vec<HirStmt>, HirNodeBox)> = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            self.push_scope();
            let mut arm_stmts = self.match_arm_bindings(arm, &val_node, value_ty_name, i, span)?;
            let (mut pending, body) = self.lower_arm_body(arm)?;
            self.pop_scope();
            arm_stmts.append(&mut pending);
            lowered.push((arm_stmts, body));
        }
        if lowered.is_empty() {
            return Err(Error::Hir(format!("empty match at {}:{}", span.start_line, span.start_col)));
        }
        if self.enum_variant_count(&value_ty_name) != lowered.len() {
            return Err(Error::Hir(format!(
                "non-exhaustive match expression at {}:{}", span.start_line, span.start_col
            )));
        }
        let res_ty = expr_type(&lowered[0].1);
        let res_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_res"), res_ty.clone(), true));
        let res_node: HirNodeBox = SVar { var: res_var, ty: res_ty.clone() }.into();

        let mut conds = Vec::new();
        let mut blocks = Vec::new();
        for (i, (mut arm_stmts, body)) in lowered.into_iter().enumerate() {
            conds.push(Self::tag_cond(&val_node, i));
            arm_stmts.push(HirStmt::Assign {
                target: res_node.clone(),
                value: body,
                span: *span,
            });
            blocks.push(HirBlock { stmts: arm_stmts });
        }
        if let Some(if_stmt) = self.build_match_if(value_ty_name, conds, blocks, span) {
            self.pending_stmts.push(if_stmt);
        }
        Ok(res_node)
    }
}
