//! A2e：`#[ensures(cond)]` 后置条件注入。
//!
//! 为函数分配隐藏局部 `result` 绑定返回值，在每个 `return` 点插入
//! 赋值 + 检查；无显式 return 的兜底路径用基元默认值检查。

use super::*;

impl crate::hir::lower::Ctx {
    /// 把 `#[ensures(cond)]` 注入到函数体（result 绑定返回值）。
    pub(crate) fn inject_ensures(
        &mut self,
        body: &mut HirBlock,
        attrs: &[crate::parser::ast::Attr],
        return_type: &HirType,
        span: Span,
    ) -> Result<()> {
        let ensures = crate::hir::contracts::ensure_conditions(attrs);
        if ensures.is_empty() {
            return Ok(());
        }
        if *return_type == HirType::Void {
            return Err(Error::Hir(format!(
                "#[ensures] requires a non-void return type (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        let checks = crate::hir::contracts::checks_enabled();
        let result_sym = Symbol::intern("result");
        let result_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(result_sym, return_type.clone(), true));
        self.bind_var(result_sym, result_var, return_type.clone(), false);

        let mut check_stmts: Vec<HirStmt> = Vec::new();
        for (cond, line, col) in &ensures {
            // #148：前置语句随检查注入到各 return 点
            let mark = self.pending_stmts.len();
            let hir_cond = self.lower_expr(cond)?;
            check_stmts.extend(self.pending_stmts.split_off(mark));
            crate::hir::contracts::ensure_bool_condition(&hir_cond, "ensures", *line, *col)?;
            if checks {
                check_stmts.push(HirStmt::Contract {
                    kind: ContractKind::Ensure,
                    cond: hir_cond,
                    line: *line,
                    col: *col,
                });
            } else {
                check_stmts.push(HirStmt::Assume { cond: hir_cond, span: Span::default() });
            }
        }

        rewrite_returns(&mut body.stmts, result_var, return_type, &check_stmts);
        if !always_returns(&body.stmts) {
            append_default_return(body, result_var, return_type, &check_stmts, span)?;
        }
        Ok(())
    }
}

impl crate::hir::lower::Ctx {
    /// A2f：`#[invariant(cond)]` 检查语句（每轮循环体首）。
    pub(crate) fn loop_check_stmts(
        &mut self,
        invariants: &[(&crate::parser::ast::Expr, usize, usize)],
    ) -> Result<Vec<HirStmt>> {
        let checks = crate::hir::contracts::checks_enabled();
        let mut out = Vec::new();
        for (cond, line, col) in invariants {
            // #148：条件降级的前置语句（短路/if 表达式）须在每轮检查前执行
            let mark = self.pending_stmts.len();
            let hir_cond = self.lower_expr(cond)?;
            out.extend(self.pending_stmts.split_off(mark));
            crate::hir::contracts::ensure_bool_condition(&hir_cond, "invariant", *line, *col)?;
            if checks {
                out.push(HirStmt::Contract {
                    kind: ContractKind::Invariant,
                    cond: hir_cond,
                    line: *line,
                    col: *col,
                });
            } else {
                out.push(HirStmt::Assume { cond: hir_cond, span: Span::default() });
            }
        }
        Ok(out)
    }
}

fn result_node(var: VarId, ty: &HirType) -> HirNodeBox {
    SVar { var, ty: ty.clone() }.into()
}

/// 递归重写 `return v` → `result = v; 检查...; return result`。
fn rewrite_returns(stmts: &mut Vec<HirStmt>, var: VarId, ty: &HirType, checks: &[HirStmt]) {
    for stmt in stmts.iter_mut() {
        match stmt {
            HirStmt::Return { value, .. } => {
                if let Some(v) = value.take() {
                    let mut seq = vec![HirStmt::Assign {
                        target: result_node(var, ty),
                        value: v,
                        span: Span::default(),
                    }];
                    seq.extend(checks.iter().cloned());
                    seq.push(HirStmt::Return { value: Some(result_node(var, ty)), span: Span::default() });
                    *stmt = HirStmt::Block { stmts: seq, span: Span::default() };
                }
            }
            HirStmt::If { then_block, elifs, else_block, .. } => {
                rewrite_returns(&mut then_block.stmts, var, ty, checks);
                for (_, b) in elifs {
                    rewrite_returns(&mut b.stmts, var, ty, checks);
                }
                if let Some(b) = else_block {
                    rewrite_returns(&mut b.stmts, var, ty, checks);
                }
            }
            HirStmt::While { body, .. } => rewrite_returns(&mut body.stmts, var, ty, checks),
            HirStmt::Block { stmts: inner, .. } => rewrite_returns(inner, var, ty, checks),
            _ => {}
        }
    }
}

/// 语句列表是否在所有路径上都返回（粗略分析，够用即可）。
fn always_returns(stmts: &[HirStmt]) -> bool {
    match stmts.last() {
        Some(HirStmt::Return { .. }) => true,
        Some(HirStmt::Block { stmts: inner, .. }) => always_returns(inner),
        Some(HirStmt::If { then_block, elifs, else_block: Some(eb), .. }) => {
            always_returns(&then_block.stmts)
                && elifs.iter().all(|(_, b)| always_returns(&b.stmts))
                && always_returns(&eb.stmts)
        }
        _ => false,
    }
}

fn default_literal(ty: &HirType) -> Option<HirLiteral> {
    match ty {
        HirType::Int => Some(HirLiteral::Int(0)),
        HirType::Float => Some(HirLiteral::Float(0.0)),
        HirType::Bool => Some(HirLiteral::Bool(false)),
        HirType::Char => Some(HirLiteral::Char('\0')),
        _ => None,
    }
}

/// 兜底路径：`result = 默认值; 检查...; return result`。
fn append_default_return(
    body: &mut HirBlock,
    var: VarId,
    ty: &HirType,
    checks: &[HirStmt],
    span: Span,
) -> Result<()> {
    let Some(lit) = default_literal(ty) else {
        return Err(Error::Hir(format!(
            "#[ensures] requires explicit return statements for non-primitive return types (at {}:{})",
            span.start_line, span.start_col
        )));
    };
    body.stmts.push(HirStmt::Assign {
        target: result_node(var, ty),
        value: SConst { val: lit, ty: ty.clone() }.into(),
        span,
    });
    body.stmts.extend(checks.iter().cloned());
    body.stmts.push(HirStmt::Return { value: Some(result_node(var, ty)), span });
    Ok(())
}
