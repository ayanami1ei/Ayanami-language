//! 宏展开辅助（从 mod.rs 拆出，保持文件 ≤300 行）：解析/追踪/标注工具。
use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

/// mcl.3：展开追踪（`AYANAMI_MACRO_TRACE=1` 时打印到 stderr；调试/IDE 用）。
pub(super) fn trace_expansion(macro_name: &str, input: &str, output: &str) {
    let on = std::env::var("AYANAMI_MACRO_TRACE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);
    if on {
        eprintln!("── macro trace: `{}`", macro_name);
        eprintln!("   input:  {}", input.replace('\n', "\\n"));
        eprintln!("   output: {}", output.replace('\n', "\\n"));
    }
}

pub(super) fn parse_source(code: &str) -> Result<Program> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    parser.parse_program().map_err(|e| Error::Compile(format!("macro output parse error: {}", e)))
}

pub(super) fn is_compiler_attr(a: &Attr) -> bool {
    // E1a/E2：环境提供标注为编译器内置命名空间
    if a.qualifier.len() == 1 && a.qualifier[0].as_str() == "env" {
        return true;
    }
    let name = a.name.as_str();
    if crate::hir::effects::is_effect(&name) {
        return true;
    }
    if a.is_builtin() {
        return crate::hir::attrs::ALLOWED.contains(&name.as_str());
    }
    false
}

pub(super) fn attrs_of(stmt: &Stmt) -> Option<&Vec<Attr>> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => Some(attrs),
        _ => None,
    }
}

pub(super) fn attrs_of_mut(stmt: &mut Stmt) -> Option<&mut Vec<Attr>> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => Some(attrs),
        _ => None,
    }
}
