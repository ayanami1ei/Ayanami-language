use super::*;
use crate::error::Error;

/// 收集语句中"使用"的变量（赋值目标是定义，不算使用）。
fn collect_stmt_var_ids(stmt: &HirStmt, vars: &mut HashSet<VarId>) {
    match stmt {
        HirStmt::Assign { value, .. } => value.collect_var_ids(vars),
        HirStmt::FieldAssign { object, value, .. } => {
            object.collect_var_ids(vars);
            value.collect_var_ids(vars);
        }
        HirStmt::IndexAssign { object, index, value, .. } => {
            object.collect_var_ids(vars);
            index.collect_var_ids(vars);
            value.collect_var_ids(vars);
        }
        HirStmt::Return { value, .. } => {
            if let Some(v) = value {
                v.collect_var_ids(vars);
            }
        }
        HirStmt::Assume { cond, .. } => cond.collect_var_ids(vars),
        HirStmt::Contract { cond, .. } => cond.collect_var_ids(vars),
        HirStmt::If { cond, then_block, elifs, else_block, .. } => {
            cond.collect_var_ids(vars);
            for s in &then_block.stmts {
                collect_stmt_var_ids(s, vars);
            }
            for (c, b) in elifs {
                c.collect_var_ids(vars);
                for s in &b.stmts {
                    collect_stmt_var_ids(s, vars);
                }
            }
            if let Some(b) = else_block {
                for s in &b.stmts {
                    collect_stmt_var_ids(s, vars);
                }
            }
        }
        HirStmt::While { cond, body, .. } => {
            cond.collect_var_ids(vars);
            for s in &body.stmts {
                collect_stmt_var_ids(s, vars);
            }
        }
        HirStmt::Expr { expr: e, .. } => e.collect_var_ids(vars),
        HirStmt::Block { stmts, .. } => {
            for s in stmts {
                collect_stmt_var_ids(s, vars);
            }
        }
        HirStmt::Break { .. } | HirStmt::Continue { .. } => {}
    }
}

impl Ctx {
    /// use-after-move 检查：语句引用的变量若已被移动则报错。
    /// 复合语句跳过（其子语句在各自 lower_stmt 时检查）。
pub(super) fn check_use_after_move(&mut self, stmt: &HirStmt) {
        if matches!(stmt, HirStmt::If { .. } | HirStmt::While { .. } | HirStmt::Block { .. }) {
            return;
        }
        let mut vars = HashSet::new();
        collect_stmt_var_ids(stmt, &mut vars);
        let mut used: Vec<VarId> = vars.into_iter().filter(|v| self.moved.contains(v)).collect();
        used.sort_by_key(|v| v.0);
        for var in used {
            let name = self
                .mir_locals
                .get(var.0)
                .map(|l| l.name.as_str().to_string())
                .unwrap_or_else(|| format!("v{}", var.0));
            let sp = stmt.span();
            self.errors.push(Error::Hir(format!(
                "use of moved value `{}` (use `.copy()` or restructure ownership) (at {}:{})",
                name, sp.start_line, sp.start_col
            )));
        }
    }

}
