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
            // #211：条件表达式不是语句，其（按值方法接收者等）移动必须在此记录；
            // then/else 分支的语句由各自 lower_stmt 检查（路径敏感性另行设计）。
            HirStmt::If { cond, .. } | HirStmt::While { cond, .. } => {
                self.walk_stmt_moves(&**cond, sp);
            }
            HirStmt::Return { value: None, .. }
            | HirStmt::Break { .. }
            | HirStmt::Continue { .. }
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
                } else {
                    self.check_move_from_borrow(inner, sp);
                }
            }
        }
        if let Some(inner) = expr.as_to_unique() {
            if let Some(id) = inner.as_local() {
                self.moved.insert(id);
            } else {
                self.check_move_from_borrow(inner, sp);
            }
        }
    }
}

fn place_root(e: &dyn HirNode) -> Option<VarId> {
    if let Some(id) = e.as_local() { return Some(id); }
    if let Some((obj, _, _)) = e.as_field_access() { return place_root(&**obj); }
    if let Some((obj, _)) = e.as_index() { return place_root(&**obj); }
    if let Some(inner) = e.as_deref() { return place_root(&**inner); }
    None
}

fn type_maybe_generic(t: &HirType) -> bool {
    let mentions = |s: &str| s.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|p| p.len() == 1 && p.chars().next().map_or(false, |c| c.is_ascii_uppercase()));
    match t {
        HirType::Named(n) => mentions(&n.as_str()),
        HirType::Unique(i) | HirType::Ref(i, _) | HirType::RawPtr(i) => type_maybe_generic(i),
        HirType::Array(i) | HirType::ArraySized(i, _) => type_maybe_generic(i),
        _ => false,
    }
}

impl Ctx {
    fn check_move_from_borrow(&mut self, inner: &HirNodeBox, sp: Span) {
        if inner.expr_type().is_copy() { return; }
        if type_maybe_generic(&inner.expr_type()) { return; }
        let Some(root) = place_root(&**inner) else { return };
        let Some(local) = self.mir_locals.get(root.0) else { return };
        if matches!(local.ty, HirType::Ref(..)) {
            self.errors.push(Error::Hir(format!(
                "cannot move out of borrowed content `{}` (non-Copy value behind a `ref`; use `ref` or clone) (at {}:{})",
                local.name.as_str(), sp.start_line, sp.start_col
            )));
        }
    }
}
