use super::*;

impl crate::hir::lower::Ctx {
    /// #146：`a && b` → `t = a; if t { t = b }`；`a || b` → `t = a; if !t { t = b }`
    pub(super) fn lower_short_circuit(&mut self, op: BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let l = auto_deref(self.lower_expr(lhs)?);
        let l = coerce_expr(l, &HirType::Bool, span)?;
        let tmp_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__logic_tmp"), HirType::Bool, true));
        let tmp: HirNodeBox = SVar { var: tmp_var, ty: HirType::Bool }.into();
        self.pending_stmts.push(HirStmt::Assign { target: tmp.clone(), value: l, span: *span });
        // 右侧惰性：其 pending 语句留在分支块内
        let saved = std::mem::take(&mut self.pending_stmts);
        let r = auto_deref(self.lower_expr(rhs)?);
        let mut inner = std::mem::replace(&mut self.pending_stmts, saved);
        let r = coerce_expr(r, &HirType::Bool, span)?;
        inner.push(HirStmt::Assign { target: tmp.clone(), value: r, span: *span });
        let cond = if matches!(op, BinaryOp::And) {
            tmp.clone()
        } else {
            SUn { op: UnaryOp::Not, arg: tmp.clone(), ty: HirType::Bool }.into()
        };
        self.pending_stmts.push(HirStmt::If {
            cond,
            then_block: HirBlock::new(inner),
            elifs: Vec::new(),
            else_block: None,
            span: *span,
        });
        Ok(tmp)
    }

}
