use crate::error::{Error, Result};
use crate::lexer::{Delimiter, Keyword, Token, TokenKind};
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, Block, Expr, InterfaceMethod, Literal, Program, Stmt, Type, UnaryOp};
use crate::parser::ast::vis::Visibility;
use crate::span::Span;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

mod atom;
mod core;
mod decl;
mod enum_iface;
mod expr;
mod impls;
mod stmt;
mod types;
mod unary;
