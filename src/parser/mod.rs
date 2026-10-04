/// Parser: recursive-descent parser from token stream to AST.
///
/// Parses function declarations, namespaces, interface/impl blocks,
/// expressions, statements, and control flow. Uses a `Parser` struct
/// with `parse_*` methods for each grammar rule.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod ast;

use crate::error::Result;

pub use parser::Parser;

/// 解析源码为 AST（手写递归下降解析器，唯一解析路径）
pub fn parse_source(source: &str) -> Result<crate::parser::ast::program::Program> {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens: Vec<_> = lexer.tokenize_all().into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut hp = crate::parser::Parser::new(tokens);
    hp.parse_program()
}

/// A5c-2：解析单个表达式（函数宏展开产物用）
pub fn parse_expression(source: &str) -> Result<crate::parser::ast::expr::Expr> {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens: Vec<_> = lexer.tokenize_all().into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut p = crate::parser::Parser::new(tokens);
    p.parse_expr_entry()
}

