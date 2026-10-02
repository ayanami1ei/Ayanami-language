//! A2b：`#[cfg(...)]` 编译期条件裁剪（宿主目标）。
//!
//! 支持谓词：
//! - `target = "linux"` / `arch = "x86_64"`（键值形式）
//! - 裸标识符：`unix` `windows` `linux` `macos` `x86_64` `aarch64`
//! - `!` 取反：`#[cfg(!windows)]`
//!
//! 多个 cfg 标注/多个谓词之间为「与」；未知谓词报错（ADR-2 严格校验）。

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, AttrArg, Expr, Literal, Program, Stmt, UnaryOp};

/// 过滤整个程序中的 cfg 项（顶层 + namespace/impl/interface 内）。
pub fn filter_program(program: &Program) -> Result<Program> {
    Ok(Program::new(filter_stmts(&program.stmts)?))
}

/// 递归过滤语句列表：cfg 为假者删除。
pub fn filter_stmts(stmts: &[Stmt]) -> Result<Vec<Stmt>> {
    let mut out = Vec::new();
    for stmt in stmts {
        if !cfg_enabled(attrs_of(stmt))? {
            continue;
        }
        match stmt {
            Stmt::Namespace { vis, name, items, span } => out.push(Stmt::Namespace {
                vis: *vis, name: *name, items: filter_stmts(items)?, span: *span,
            }),
            Stmt::ImplBlock { attrs, type_name, generic_params, methods, span } => out.push(Stmt::ImplBlock {
                attrs: attrs.clone(), type_name: *type_name,
                generic_params: generic_params.clone(), methods: filter_stmts(methods)?, span: *span,
            }),
            Stmt::InterfaceDef { attrs, name, generic_params, methods, span } => {
                let mut kept = Vec::new();
                for m in methods {
                    if cfg_enabled(Some(&m.attrs))? { kept.push(m.clone()); }
                }
                out.push(Stmt::InterfaceDef {
                    attrs: attrs.clone(), name: *name,
                    generic_params: generic_params.clone(), methods: kept, span: *span,
                });
            }
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}

/// 该语句是否通过 cfg（供包导出等 AST 级消费者；无效 cfg 保守视为启用，交由 HIR 报错）。
pub fn stmt_enabled(stmt: &Stmt) -> bool {
    cfg_enabled(attrs_of(stmt)).unwrap_or(true)
}

fn attrs_of(stmt: &Stmt) -> Option<&[Attr]> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => Some(attrs),
        _ => None,
    }
}

fn cfg_enabled(attrs: Option<&[Attr]>) -> Result<bool> {
    let Some(attrs) = attrs else { return Ok(true) };
    for a in attrs.iter().filter(|a| a.name.as_str() == "cfg") {
        if a.args.is_empty() {
            return Err(Error::Hir(format!(
                "#[cfg] requires at least one predicate (at {}:{})", a.span.start_line, a.span.start_col)));
        }
        for arg in &a.args {
            if !eval_predicate(arg, a.span)? { return Ok(false); }
        }
    }
    Ok(true)
}

fn eval_predicate(arg: &AttrArg, span: crate::span::Span) -> Result<bool> {
    match arg {
        AttrArg::KeyValue(key, val) => {
            let value = match val.as_ref() {
                AttrArg::Expr(e) => match e.as_ref() {
                    Expr::Literal(Literal::String(s, _)) => s.clone(),
                    Expr::Ident(s, _) => s.as_str().to_string(),
                    _ => return Err(unsupported(span)),
                },
                _ => return Err(unsupported(span)),
            };
            match key.as_str().as_str() {
                "target" => Ok(value == std::env::consts::OS),
                "arch" => Ok(value == std::env::consts::ARCH),
                other => Err(Error::Hir(format!(
                    "unknown cfg key `{}` (supported: target, arch) (at {}:{})",
                    other, span.start_line, span.start_col))),
            }
        }
        AttrArg::Expr(e) => eval_bare(e, span),
    }
}

fn eval_bare(e: &Expr, span: crate::span::Span) -> Result<bool> {
    match e {
        Expr::Ident(name, _) => match name.as_str().as_str() {
            "unix" => Ok(cfg!(unix)),
            "windows" => Ok(cfg!(windows)),
            "linux" => Ok(std::env::consts::OS == "linux"),
            "macos" => Ok(std::env::consts::OS == "macos"),
            "x86_64" => Ok(std::env::consts::ARCH == "x86_64"),
            "aarch64" => Ok(std::env::consts::ARCH == "aarch64"),
            other => Err(Error::Hir(format!(
                "unknown cfg predicate `{}` (at {}:{})", other, span.start_line, span.start_col))),
        },
        Expr::Unary { op: UnaryOp::Not, arg, .. } => Ok(!eval_bare(arg, span)?),
        _ => Err(unsupported(span)),
    }
}

fn unsupported(span: crate::span::Span) -> Error {
    Error::Hir(format!(
        "unsupported #[cfg] predicate: use `target = \"...\"`, `arch = \"...\"`, a bare target name, or `!` negation (at {}:{})",
        span.start_line, span.start_col))
}
