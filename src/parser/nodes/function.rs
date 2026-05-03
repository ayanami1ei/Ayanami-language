use crate::parser::{ast::{AstNode, Stmt}, symbol::fn_symbol::FnSymbol};

#[derive(Default)]
pub struct Function{
    symbol:FnSymbol,
    block:Vec<Stmt>,
}

impl AstNode for Function{
    fn gen_from(&mut self, _tokens: &[crate::lexer::Token], _index: &mut usize) -> anyhow::Result<()> {
        todo!()
    }

    fn span(&self) -> crate::span::Span {
        todo!()
    }
}