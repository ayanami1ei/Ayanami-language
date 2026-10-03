//! A5b：宏引用校验（导入加载后；展开执行在 A5b-2）。

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, Program, Stmt};

use super::attrs::{Imports, ALLOWED};

/// 是否为编译器内置属性（白名单或注册效应；含 `core::` 别名）。
fn is_compiler_attr(a: &Attr) -> bool {
    let name = a.name.as_str();
    if crate::hir::effects::is_effect(&name) {
        return true;
    }
    if a.qualifier.is_empty() || (a.qualifier.len() == 1 && a.qualifier[0].as_str() == "core") {
        return ALLOWED.contains(&name.as_str());
    }
    false
}

/// A5b：导入加载完成后校验宏引用（存在性；展开执行在 A5b-2）。
pub fn validate_macros(
    program: &Program,
    tables: &HashMap<Symbol, Vec<Symbol>>,
) -> Result<()> {
    let imports = Imports::collect(program);
    let mut out = Ok(());
    for stmt in &program.stmts {
        validate_macros_stmt(stmt, tables, &imports, &mut out);
        if out.is_err() { return out; }
    }
    out
}

fn validate_macros_stmt(
    stmt: &Stmt,
    tables: &HashMap<Symbol, Vec<Symbol>>,
    imports: &Imports,
    out: &mut Result<()>,
) {
    let attrs: Vec<&Attr> = match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => attrs.iter().collect(),
        Stmt::Attributed { attrs, .. } => attrs.iter().collect(),
        _ => Vec::new(),
    };
    for a in attrs {
        if is_compiler_attr(a) {
            continue;
        }
        let name = a.name.as_str();
        let (pkg_sym, found) = if !a.qualifier.is_empty() {
            let pkg = a.qualifier[0];
            let rest: Vec<String> = a.qualifier[1..].iter().map(|s| s.as_str()).collect();
            let qualified = if rest.is_empty() { name.clone() } else { format!("{}.{}", rest.join("."), name) };
            let found = tables.get(&pkg).map_or(false, |ms| {
                ms.iter().any(|m| m.as_str() == qualified || m.as_str() == name)
            });
            (Some(pkg), found)
        } else {
            match imports.macros.iter().find_map(|(p, ms)| ms.iter().any(|m| m == &name).then(|| p.clone())) {
                Some(pkg_sym) => {
                    let pkg = Symbol::intern(&pkg_sym);
                    let found = tables.get(&pkg).map_or(false, |ms| ms.iter().any(|m| m.as_str() == name));
                    (Some(pkg), found)
                }
                None => (None, false),
            }
        };
        if !found {
            *out = Err(Error::Hir(format!(
                "macro #[{}] not found in package `{}` (at {}:{})",
                a.path_str(),
                pkg_sym.map(|s| s.as_str()).unwrap_or_default(),
                a.span.start_line,
                a.span.start_col
            )));
            return;
        }
        // 已解析到宏：展开执行在 A5b-2
        // 宏展开（A5b-2）应在 HIR 之前完成；这里出现说明展开遗漏
        *out = Err(Error::Hir(format!(
            "macro #[{}] was not expanded (internal error) (at {}:{})",
            a.path_str(),
            a.span.start_line,
            a.span.start_col
        )));
        return;
    }
    match stmt {
        Stmt::FnDecl { body, .. } => validate_block(body, tables, imports, out),
        Stmt::If { then_block, elifs, else_block, .. } => {
            validate_block(then_block, tables, imports, out);
            for (_, b) in elifs { validate_block(b, tables, imports, out); }
            if let Some(b) = else_block { validate_block(b, tables, imports, out); }
        }
        Stmt::For { body, .. } | Stmt::While { body, .. } => {
            validate_block(body, tables, imports, out);
        }
        Stmt::Namespace { items, .. } => {
            for s in items { validate_macros_stmt(s, tables, imports, out); if out.is_err() { return; } }
        }
        Stmt::ImplBlock { methods, .. } => {
            for s in methods { validate_macros_stmt(s, tables, imports, out); if out.is_err() { return; } }
        }
        Stmt::InterfaceDef { methods, .. } => {
            for m in methods {
                for a in &m.attrs {
                    if is_compiler_attr(a) { continue; }
                    *out = Err(Error::Hir(format!(
                        "macro #[{}] is not supported on interface methods (at {}:{})",
                        a.path_str(), a.span.start_line, a.span.start_col
                    )));
                    return;
                }
            }
        }
        Stmt::Attributed { stmt: inner, .. } => validate_macros_stmt(inner, tables, imports, out),
        _ => {}
    }
}

fn validate_block(
    block: &crate::parser::ast::Block,
    tables: &HashMap<Symbol, Vec<Symbol>>,
    imports: &Imports,
    out: &mut Result<()>,
) {
    for s in &block.stmts {
        validate_macros_stmt(s, tables, imports, out);
        if out.is_err() { return; }
    }
}
