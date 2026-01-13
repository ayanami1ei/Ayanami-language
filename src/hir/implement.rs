use core::fmt;

use crate::{
    error_type::Error, hir::{HIR, HisIsa}, types::{Expr, Stmt}
};

impl HIR {
    fn gen_assign_ir(_left:&Box<Expr>,_right:&Box<Expr>)->Result<HisIsa, Error> {
        let mut var=String::new();
        if let Expr::Var(name, var_type)=_left.as_ref(){
            var.push_str(name);
        }

        Err(Error::new_warning("".to_string()))
    }

    pub(crate) fn gen_ir(&mut self) {
        if let Stmt::Assign(left, right) = self.stmts[self.i].clone() {
        } else if let Stmt::For(_, _, _, _, _) = self.stmts[self.i] {
        } else if let Stmt::While(_, _) = self.stmts[self.i] {
        } else if let Stmt::If(_, _) = self.stmts[self.i] {
        } else if let Stmt::Func(_, _, _, _) = self.stmts[self.i] {
        } else if let Stmt::Return(_) = self.stmts[self.i] {
        }
    }
}

impl fmt::Display for HisIsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let HisIsa::Assign(right, left) = self {
            write!(f, "{} = {}", left, right)
        } else if let HisIsa::Jmp(label) = self {
            write!(f, "{}", label)
        } else if let HisIsa::Label(label) = self {
            write!(f, "{}:", label)
        } else {
            write!(f, "unkonwn isa")
        }
    }
}
