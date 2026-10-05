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

/// 约束/泛型实参的类型文本化（`Into[float]` → `Into<float>`，与 HIR 泛型名约定一致）
fn ast_type_text(ty: &Type) -> String {
    match ty {
        Type::Default => "_".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(n, _) => n.as_str().to_string(),
        Type::Generic(n, args, _) => format!("{}<{}>", n, args.iter().map(ast_type_text).collect::<Vec<_>>().join(",")),
        Type::Array(inner, _) => format!("[{}]", ast_type_text(inner)),
        Type::Ref(inner, m, _) => format!("ref {}{}", if *m { "mut " } else { "" }, ast_type_text(inner)),
        Type::Unique(inner, _) => ast_type_text(inner),
        Type::Self_(_) => "Self".into(),
        Type::FnPtr(..) => "fn".into(),
    }
}

mod atom;
mod const_decl;
mod self_type;
mod core;
mod decl;
mod enum_iface;
mod expr;
mod impls;
mod literal_text;
mod macro_call;
mod stmt;
mod types;
mod unary;
