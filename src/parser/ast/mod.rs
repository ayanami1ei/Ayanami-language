pub mod expr;
pub mod stmt;
pub mod visitor;

use crate::lexer::Token;
use crate::span::Span;
use anyhow::Result;

pub use expr::*;
pub use stmt::*;

pub trait AstNode: Default {
    fn gen_from(&mut self, tokens: &[Token], index: &mut usize) -> Result<()>;
    fn span(&self) -> Span;
}
