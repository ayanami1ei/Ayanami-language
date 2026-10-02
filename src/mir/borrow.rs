use crate::error::{Error, Result};
use crate::hir::ir::{HirType, VarId};
use crate::mir::ir::*;

/// 借用检查（保守版）：
/// - 借用是语句作用域的临时值：引用不能存入变量/字段、不能作为返回值逃逸；
/// - 同一语句内对同一变量的冲突借用（可变 × 任意、任意 × 可变）报错；
/// - 借用目标与使用顺序的完整生命周期分析（NLL）暂未实现。
pub fn check_borrows(mir_fn: &MirFn) -> Result<()> {
    if matches!(mir_fn.return_type, HirType::Ref(..)) {
        return Err(Error::Borrow(format!(
            "function `{}` cannot return a reference: borrows cannot escape",
            mir_fn.name.as_str()
        )));
    }
    let mut checker = BorrowChecker { active: Vec::new(), errors: Vec::new() };
    checker.check_stmts(&mir_fn.body);
    match checker.errors.into_iter().next() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

struct ActiveBorrow {
    var: VarId,
    mutable: bool,
}

struct BorrowChecker {
    active: Vec<ActiveBorrow>,
    errors: Vec<Error>,
}

impl BorrowChecker {
    fn check_stmts(&mut self, stmts: &[MirStmtBox]) {
        for stmt in stmts {
            // 语句内创建的借用在该语句结束时全部释放
            let saved = self.active.len();
            stmt.for_each_child_expr(&mut |e| self.collect_borrows(e));
            self.active.truncate(saved);
            // 递归检查嵌套语句（各自独立的作用域）
            stmt.for_each_child_stmt(&mut |s| self.check_stmts(&[MirStmtBox(s.clone_stmt())]));
        }
    }

    fn collect_borrows(&mut self, expr: &dyn MirNode) {
        if let Some((var, mutable)) = expr.as_ref() {
            if self.active.iter().any(|b| b.var == var && (b.mutable || mutable)) {
                self.errors.push(Error::Borrow(format!(
                    "cannot borrow v{} as {} while it is already borrowed in the same expression",
                    var.0,
                    if mutable { "mutable" } else { "immutable" }
                )));
            } else {
                self.active.push(ActiveBorrow { var, mutable });
            }
        }
        expr.for_each_child(&mut |c| self.collect_borrows(c));
    }
}
