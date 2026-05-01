use crate::intern::Symbol;
use crate::parser::ast::expr::{Expr, Type};
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, span: Span) -> Self {
        Self { stmts, span }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    FnDecl {
        name: Symbol,
        params: Vec<(Symbol, Type)>,
        return_type: Type,
        body: Block,
        span: Span,
    },
    Assign {
        name: Symbol,
        value: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    If {
        cond: Expr,
        then_block: Block,
        elifs: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },
    For {
        iterator: Symbol,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Block,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        span: Span,
    },
    ExprStmt {
        expr: Expr,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::FnDecl { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::For { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ExprStmt { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self { stmts }
    }
}
