use std::cell::RefCell;
use std::collections::HashMap;
use crate::hir::ir::{VarId, HirType};
use crate::mir::ir::*;

pub fn check_borrows(mir_fn: &MirFn) -> Result<(), String> {
    let checker = BorrowChecker::new(mir_fn);
    checker.check()
}

struct Borrow {
    var: VarId,
    mutable: bool,
    start: usize,
}

struct BorrowChecker<'a> {
    mir_fn: &'a MirFn,
    active: RefCell<HashMap<VarId, Vec<Borrow>>>,
}

impl<'a> BorrowChecker<'a> {
    fn new(mir_fn: &'a MirFn) -> Self {
        Self { mir_fn, active: RefCell::new(HashMap::new()) }
    }

    fn check(&self) -> Result<(), String> {
        self.check_stmts(&self.mir_fn.body, 0);
        Ok(())
    }

    fn check_stmts(&self, stmts: &[MirStmtBox], base: usize) {
        for stmt in stmts {
            let mut err = None;
            stmt.for_each_child_expr(&mut |child| {
                if let Err(e) = self.check_expr(child) {
                    err = Some(e);
                }
            });
            if let Some(e) = err { panic!("borrow error: {}", e); }
            stmt.for_each_child_stmt(&mut |child| {
                child.for_each_child_expr(&mut |c| {
                    let _ = self.check_expr(c);
                });
            });
        }
    }

    fn check_expr(&self, expr: &dyn MirNode) -> Result<(), String> {
        if let Some(var) = expr.as_local() {
            let active = self.active.borrow();
            if let Some(borrows) = active.get(&var) {
                if borrows.iter().any(|b| b.mutable) {
                    return Err(format!("cannot read v{}, mutably borrowed", var.0));
                }
            }
        }
        if let Some((var, mutable)) = expr.as_ref() {
            self.add_borrow(var, mutable)?;
        }
        expr.for_each_child(&mut |child| {
            let _ = self.check_expr(child);
        });
        Ok(())
    }

    fn add_borrow(&self, var: VarId, mutable: bool) -> Result<(), String> {
        let mut active = self.active.borrow_mut();
        if let Some(borrows) = active.get(&var) {
            for b in borrows {
                if mutable || b.mutable {
                    return Err(format!(
                        "cannot borrow v{} as {}mut, already borrowed",
                        var.0, if mutable { "" } else { "im" }
                    ));
                }
            }
        }
        active.entry(var).or_default().push(Borrow { var, mutable, start: 0 });
        Ok(())
    }
}
