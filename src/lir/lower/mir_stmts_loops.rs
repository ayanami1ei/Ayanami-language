//! While / Drop 语句的 LIR 降级（含条件每轮 pre/post 与动态数组计数释放）。
use super::*;
use super::util::*;

impl MirStmtNode for SMirWhileStmt { fn span(&self) -> crate::span::Span { self.span }
    fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.cond); }
    fn for_each_child_stmt_mut(&mut self, f: &mut dyn FnMut(&mut MirStmtBox)) { for s in &mut self.pre_cond { f(s); } for s in &mut self.post_cond { f(s); } for s in &mut self.body { f(s); } }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let cond_lbl = ctx.next_block_label("while.cond");
        let body_lbl = ctx.next_block_label("while.body");
        let end_lbl = ctx.next_block_label("while.end");

        let cond_lbl2 = cond_lbl.clone();
        ctx.loop_stack_mut().push((cond_lbl.clone(), end_lbl.clone()));
        ctx.emit(SLirBr { label: cond_lbl.clone() }.into());

        ctx.set_current_block(cond_lbl2);
        for s in &self.pre_cond { s.lower_to_lir_stmt(ctx); }
        let cond_val = self.cond.lower_to_lir(ctx);
        for s in &self.post_cond { s.lower_to_lir_stmt(ctx); }
        ctx.emit(SLirBrCond { cond: cond_val, true_block: body_lbl.clone(), false_block: end_lbl.clone() }.into());

        ctx.set_current_block(body_lbl);
        for s in &self.body { s.lower_to_lir_stmt(ctx); }
        ctx.emit(SLirBr { label: cond_lbl }.into());

        ctx.loop_stack_mut().pop();
        ctx.set_current_block(end_lbl);
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}While", "", width = level * 2)?;
        writeln!(w, "{:width$}  cond:", "", width = level * 2)?;
        self.cond.display(level + 1, w)?;
        if !self.pre_cond.is_empty() { writeln!(w, "{:width$}  pre_cond:", "", width = level * 2)?; write_stmt_block(&self.pre_cond, level + 2, w)?; }
        if !self.post_cond.is_empty() { writeln!(w, "{:width$}  post_cond:", "", width = level * 2)?; write_stmt_block(&self.post_cond, level + 2, w)?; }
        writeln!(w, "{:width$}  body:", "", width = level * 2)?;
        write_stmt_block(&self.body, level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.pre_cond { f(&**s); } for s in &self.post_cond { f(&**s); } for s in &self.body { f(&**s); } }
    fn as_while(&self) -> Option<WhileParts<'_>> { Some((&self.cond, &self.body)) }

}

impl MirStmtNode for SMirDropCounted { fn span(&self) -> crate::span::Span { self.span }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        ctx.emit(SLirDropArray { var: self.var, elem_ty: self.elem_ty.clone(), count_var: self.count_var }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}DropCounted(v{} : {:?}, count=v{})", "", self.var.0, self.elem_ty, self.count_var.0, width = level * 2)
    }
}

impl MirStmtNode for SMirDropStmt { fn span(&self) -> crate::span::Span { self.span }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        ctx.emit(SLirDropValue { var: self.var, ty: self.ty.clone() }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Drop(v{} : {})", "", self.var.0, display_hir_type(&self.ty), width = level * 2)
    }
    fn as_drop(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
}
