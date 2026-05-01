use crate::parser::ast::stmt::Stmt;
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
