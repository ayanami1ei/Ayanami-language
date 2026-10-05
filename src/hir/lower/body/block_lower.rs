use super::*;

impl crate::hir::lower::Ctx {
    // ----------------------------------------------------------------
    //  块/语句降级：lower_block → lower_stmt → lower_for
    // ----------------------------------------------------------------

    pub(crate) fn lower_block(&mut self, block: &Block) -> Result<HirBlock> {
        let (b, _) = self.lower_block_impl(block, false)?;
        Ok(b)
    }

    /// `tail_as_value`：块尾表达式作为值返回（函数体）；否则求值后丢弃（if/while 等）
    pub(crate) fn lower_block_impl(&mut self, block: &Block, tail_as_value: bool) -> Result<(HirBlock, Option<HirNodeBox>)> {
        self.push_scope();
        let saved_hints = std::mem::take(&mut self.usage_hints);
        self.usage_hints = self.collect_usage_hints(&block.stmts);
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            let mark = self.pending_stmts.len();
            let lowered = self.lower_stmt(stmt)?;
            // A3d：表达式内联语句（如 `?`）必须先于本语句执行
            let pending: Vec<HirStmt> = self.pending_stmts.split_off(mark);
            stmts.extend(pending);
            stmts.push(lowered);
        }
        let mut tail_value = None;
        if let Some(t) = &block.tail {
            let mark = self.pending_stmts.len();
            let lowered = self.lower_expr(t)?;
            let pending: Vec<HirStmt> = self.pending_stmts.split_off(mark);
            stmts.extend(pending);
            if tail_as_value {
                tail_value = Some(lowered);
            } else {
                stmts.push(HirStmt::Expr { expr: lowered, span: block.span });
            }
        }
        self.usage_hints = saved_hints;
        self.pop_scope();
        Ok((HirBlock::new(stmts), tail_value))
    }
}
