use crate::parser::ast::expr::Expr;
use crate::parser::ast::stmt::Stmt;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// 裸尾表达式（块末尾无分号）：函数体作为隐式返回值
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, span: Span) -> Self {
        Self { stmts, tail: None, span }
    }
}
