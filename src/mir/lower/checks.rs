use super::*;
use crate::error::Error;
use crate::span::Span;

impl Ctx {
    /// #116：块是否在所有路径上发散（`return` / noreturn 调用）。
    /// 发散路径的移动不合并到后续代码（避免正常路径误跳过 drop）；
    /// `break`/`continue` 的移动保守保留（它们会到达循环出口，避免漏报）。
    pub(super) fn block_diverges(&self, stmts: &[HirStmt]) -> bool {
        match stmts.last() {
            Some(HirStmt::Return { .. }) => true,
            Some(HirStmt::Block { stmts, .. }) => self.block_diverges(stmts),
            Some(HirStmt::If { then_block, elifs, else_block, .. }) => {
                self.block_diverges(&then_block.stmts)
                    && elifs.iter().all(|(_, b)| self.block_diverges(&b.stmts))
                    && else_block.as_ref().map_or(false, |b| self.block_diverges(&b.stmts))
            }
            // noreturn 调用（panic 等）类型为 Never → 分支发散
            Some(HirStmt::Expr { expr, .. }) => matches!(expr.expr_type(), HirType::Never),
            _ => false,
        }
    }

    /// use-after-move 检查（按语句内求值顺序）：语句引用的变量若已被移动则报错；
    /// 遍历中同时按顺序记录本语句产生的移动（重复移动 / 移动后使用同样报错）。
    /// 复合语句跳过（其子语句在各自 lower_stmt 时检查）。
    pub(super) fn check_use_after_move(&mut self, stmt: &HirStmt) {
        let sp = stmt.span();
        match stmt {
            HirStmt::Assign { value, .. }
            | HirStmt::DerefAssign { value, .. }
            | HirStmt::Expr { expr: value, .. } => self.walk_stmt_moves(&**value, sp),
            HirStmt::FieldAssign { object, value, .. } => {
                self.walk_stmt_moves(&**object, sp);
                self.walk_stmt_moves(&**value, sp);
            }
            HirStmt::IndexAssign { object, index, value, .. } => {
                self.walk_stmt_moves(&**object, sp);
                self.walk_stmt_moves(&**index, sp);
                self.walk_stmt_moves(&**value, sp);
            }
            HirStmt::Return { value: Some(v), .. } => self.walk_stmt_moves(&**v, sp),
            HirStmt::Assume { cond, .. } | HirStmt::Contract { cond, .. } => {
                self.walk_stmt_moves(&**cond, sp);
            }
            HirStmt::Return { value: None, .. }
            | HirStmt::Break { .. }
            | HirStmt::Continue { .. }
            | HirStmt::If { .. }
            | HirStmt::While { .. }
            | HirStmt::Block { .. } => {}
        }
    }

    fn walk_stmt_moves(&mut self, expr: &dyn HirNode, sp: Span) {
        if let Some(id) = expr.as_local() {
            if self.moved.contains(&id) {
                let name = self
                    .mir_locals
                    .get(id.0)
                    .map(|l| l.name.as_str().to_string())
                    .unwrap_or_else(|| format!("v{}", id.0));
                                self.errors.push(Error::Hir(format!(
                    "use of moved value `{}` (use `.copy()` or restructure ownership) (at {}:{})",
                    name, sp.start_line, sp.start_col
                )));
            }
        }
        expr.for_each_child(&mut |c| self.walk_stmt_moves(c, sp));
        if let Some(inner) = expr.as_move() {
            if !inner.expr_type().is_copy() {
                if let Some(id) = inner.as_local() {
                    self.moved.insert(id);
                }
            }
        }
        if let Some(inner) = expr.as_to_unique() {
            if let Some(id) = inner.as_local() {
                self.moved.insert(id);
            }
        }
    }
}
