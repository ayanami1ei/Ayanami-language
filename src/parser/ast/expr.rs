use std::{cell::RefCell, rc::Rc};

use crate::{lexer::TokenKind, parser::symbol::symbol::Symbol};

pub enum Expr<'a>{
    BinOp{
        lhs:Rc<RefCell<Expr<'a>>>,
        op:String,
        rhs:Rc<RefCell<Expr<'a>>>,
    },
    SingleOp{
        arg:Rc<RefCell<Expr<'a>>>,
        op:String,
    },
    FuncCall(Rc<RefCell<Symbol<'a>>>),
    Var(Rc<RefCell<Symbol<'a>>>),
    Const(TokenKind),
}