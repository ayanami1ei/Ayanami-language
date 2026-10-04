use super::*;

impl MirStmtNode for SMirAssumeStmt {
    fn span(&self) -> crate::span::Span { self.span }
    fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let cond = self.cond.lower_to_lir(ctx);
        ctx.emit(SLirAssume { cond }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Assume", "", width = level * 2)?;
        self.cond.display(level + 1, w)
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {
        f(&*self.cond);
    }
}

impl MirStmtNode for SMirContractStmt {
    fn span(&self) -> crate::span::Span { self.span }
    fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let cond = self.cond.lower_to_lir(ctx);
        ctx.emit(SLirContractCheck {
            kind: self.kind, cond, line: self.line, col: self.col,
        }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}{}({}:{})", "", self.kind.label(), self.line, self.col, width = level * 2)?;
        self.cond.display(level + 1, w)
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {
        f(&*self.cond);
    }
}
