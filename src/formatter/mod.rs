use std::fmt::Write;
use crate::parser::ast::*;
use crate::parser::ast::Stmt::*;
use crate::parser::ast::vis::Visibility;
use crate::intern::Symbol;

const INDENT: &str = "    ";

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for (i, stmt) in program.stmts.iter().enumerate() {
        if i > 0 {
            write_stmt_separator(&mut out, stmt);
        }
        write_stmt(&mut out, stmt, 0);
    }
    let s = out.trim_end().to_string();
    if s.is_empty() { s } else { s + "\n" }
}

mod expr;
mod helpers;
mod stmt;

/// 表达式 → 源码文本（A2a：标注实参渲染复用）
pub(crate) fn format_expr(expr: &Expr) -> String {
    expr::write_expr(expr)
}

use helpers::write_stmt_separator;
use stmt::write_stmt;

pub fn format_file(code: &str) -> crate::error::Result<String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()?;
    Ok(format_program(&program))
}
