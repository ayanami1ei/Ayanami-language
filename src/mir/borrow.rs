use std::collections::HashMap;
use crate::hir::ir::VarId;
use crate::mir::ir::*;

pub fn check_borrows(mir_fn: &MirFn) -> Result<(), String> {
    let mut checker = BorrowChecker::new(mir_fn);
    checker.check()
}

struct Borrow {
    var: VarId,
    mutable: bool,
    start: usize,
}

struct BorrowChecker<'a> {
    mir_fn: &'a MirFn,
    active: HashMap<VarId, Vec<Borrow>>,
}

impl<'a> BorrowChecker<'a> {
    fn new(mir_fn: &'a MirFn) -> Self {
        Self { mir_fn, active: HashMap::new() }
    }

    fn check(&mut self) -> Result<(), String> {
        self.check_stmts(&self.mir_fn.body, 0)
    }

    fn check_stmts(&mut self, stmts: &[MirStmt], base: usize) -> Result<(), String> {
        for (i, stmt) in stmts.iter().enumerate() {
            let idx = base + i;
            match stmt {
                MirStmt::Assign { target, value } => {
                    self.check_expr(value)?;
                    if let MirExpr::Local(var, _, _) = target { self.check_write(*var)?; }
                }
                MirStmt::FieldAssign { object, value, .. } => {
                    self.check_expr(value)?;
                    if let MirExpr::Local(var, _, _) = object.as_ref() { self.check_write(*var)?; }
                }
                MirStmt::IndexAssign { object, index, value } => {
                    self.check_expr(value)?;
                    self.check_expr(index)?;
                    if let MirExpr::Local(var, _, _) = object.as_ref() { self.check_write(*var)?; }
                }
                MirStmt::Return { value } => { if let Some(v) = value { self.check_expr(v)?; } }
                MirStmt::Expr(expr) => self.check_expr(expr)?,
                MirStmt::Block(stmts) => { self.check_stmts(stmts, idx + 1)?; }
                MirStmt::If { cond, then_block, elifs, else_block, .. } => {
                    self.check_expr(cond)?;
                    self.check_stmts(then_block, idx + 1)?;
                    for (c, b) in elifs { self.check_expr(c)?; self.check_stmts(b, idx + 1)?; }
                    if let Some(b) = else_block { self.check_stmts(b, idx + 1)?; }
                }
                MirStmt::While { cond, body, .. } => {
                    self.check_expr(cond)?;
                    self.check_stmts(body, idx + 1)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &MirExpr) -> Result<(), String> {
        match expr {
            MirExpr::Local(var, _, _) => {
                if let Some(borrows) = self.active.get(var) {
                    if let Some(_b) = borrows.iter().find(|b| b.mutable) {
                        return Err(format!("cannot read v{}, mutably borrowed", var.0));
                    }
                }
            }
            MirExpr::Ref { expr, mutable, .. } => {
                if let MirExpr::Local(var, _, _) = expr.as_ref() {
                    self.add_borrow(*var, *mutable)?;
                }
            }
            MirExpr::Binary { lhs, rhs, .. } => { self.check_expr(lhs)?; self.check_expr(rhs)?; }
            MirExpr::Unary { arg, .. } => self.check_expr(arg)?,
            MirExpr::Call { args, .. } => { for a in args { self.check_expr(a)?; } }
            MirExpr::Move(inner, _) | MirExpr::Clone(inner, _) => self.check_expr(inner)?,
            MirExpr::ToUnique(inner, _) | MirExpr::ToShared(inner, _) | MirExpr::ToWeak(inner, _) => self.check_expr(inner)?,
            MirExpr::VirtualCall { receiver, args, .. } => {
                self.check_expr(receiver)?;
                for a in args { self.check_expr(a)?; }
            }
            MirExpr::MakeFatPtr { value, .. } => self.check_expr(value)?,
            MirExpr::FieldAccess { object, .. } => self.check_expr(object)?,
            MirExpr::StructLiteral { fields, .. } => { for (_, e) in fields { self.check_expr(e)?; } }
            MirExpr::ArrayLiteral(elems, _) => { for e in elems { self.check_expr(e)?; } }
            MirExpr::ArraySized { count, .. } => self.check_expr(count)?,
            MirExpr::Index { object, index, .. } => { self.check_expr(object)?; self.check_expr(index)?; }
            _ => {}
        }
        Ok(())
    }

    fn check_write(&mut self, var: VarId) -> Result<(), String> {
        if let Some(borrows) = self.active.get(&var) {
            for b in borrows {
                return Err(format!(
                    "cannot write to v{}, borrowed as {}mut",
                    var.0, if b.mutable { "" } else { "im" }
                ));
            }
        }
        Ok(())
    }

    fn add_borrow(&mut self, var: VarId, mutable: bool) -> Result<(), String> {
        if let Some(borrows) = self.active.get(&var) {
            for b in borrows {
                if mutable || b.mutable {
                    return Err(format!(
                        "cannot borrow v{} as {}mut, already borrowed",
                        var.0, if mutable { "" } else { "im" }
                    ));
                }
            }
        }
        self.active.entry(var).or_default().push(Borrow { var, mutable, start: 0 });
        Ok(())
    }
}
