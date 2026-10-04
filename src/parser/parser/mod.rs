use crate::error::{Error, Result};
use crate::lexer::{Delimiter, Keyword, Token, TokenKind};
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, Block, Expr, InterfaceMethod, Literal, Program, Stmt, Type, UnaryOp};
use crate::parser::ast::vis::Visibility;
use crate::span::Span;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// >0 时禁止把 `ident {` 当作结构体字面量（if/while 条件上下文）
    struct_lit_depth: u32,
}

mod atom;
mod core;
mod decl;
mod enum_iface;
mod expr;
mod impls;
mod macro_call;
mod stmt;
mod types;
mod unary;
