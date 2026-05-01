pub mod binary_op;
pub mod block;
pub mod expr;
pub mod literal;
pub mod program;
pub mod stmt;
pub mod ty;
pub mod unary_op;
pub mod visitor;

use crate::lexer::Token;
use crate::span::Span;
use anyhow::Result;

pub use binary_op::BinaryOp;
pub use block::Block;
pub use expr::Expr;
pub use literal::Literal;
pub use program::Program;
pub use stmt::Stmt;
pub use ty::Type;
pub use unary_op::UnaryOp;

pub trait AstNode: Default {
    fn gen_from(&mut self, tokens: &[Token], index: &mut usize) -> Result<()>;
    fn span(&self) -> Span;
}
