use anyhow::Result;

use crate::lexer::{Lexer, Token};

pub mod assign;
pub mod expr;

pub trait AstNode: Default {
    fn gen_from(&mut self, tokens:& Vec<Token>, index:&mut usize)->Result<()>;
}