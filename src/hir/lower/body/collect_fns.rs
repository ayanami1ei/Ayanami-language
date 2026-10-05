use super::*;

impl crate::hir::lower::Ctx {
    // ----------------------------------------------------------------
    //  阶段 1：收集函数和接口签名
    // ----------------------------------------------------------------

    pub(crate) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.collect_fns_with_ns(stmts, "")
    }
}
