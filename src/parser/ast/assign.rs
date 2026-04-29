use std::{cell::RefCell, rc::Rc};

use anyhow::{Result, anyhow};
use ast_macro::ast;

use crate::{lexer::Token, parser::{ast::AstNode, symbol::symbol::Symbol}};

//#[ast]
#[derive(Default)]
pub struct Assign<'a>{
    left:Rc<RefCell<Symbol<'a>>>,
    right:Rc<RefCell<Symbol<'a>>>,
}

impl<'a> AstNode for Assign<'a>{
    fn gen_from(&mut self, tokens:& Vec<Token>, index:&mut usize)->Result<()> {
        


        Ok(())
    }
}