//! A5b/A5d：库注解引用校验（导入加载后）。
//!
//! - 宏注解（`#[macro]`）必须已被宏展开消费，残留即内部错误；
//! - 优化注解（`#[pass]`）/ 只读检查（`#[check]`）保留到 MIR 阶段执行，这里只校验存在性；
//! - 裸名自动作用域：跨所有已导入包查找，重名报歧义；
//! - 解析与宏展开共用 `AnnotationTables`（同包跨类别唯一性由打包/加载校验）。

use std::collections::HashMap;

use crate::annotation_table::{AnnKind, AnnotationTables};
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, Program, Stmt};

use super::attrs::ALLOWED;

/// 是否为编译器内置属性（白名单或注册效应；含 `core::` 别名）。
fn is_compiler_attr(a: &Attr) -> bool {
    let name = a.name.as_str();
    if crate::hir::effects::is_effect(&name) {
        return true;
    }
    if a.qualifier.is_empty() || (a.qualifier.len() == 1 && a.qualifier[0].as_str() == "core") {
        return ALLOWED.contains(&name.as_str());
    }
    // E1a：环境提供标注（编译器内置命名空间）
    if a.qualifier.len() == 1 && a.qualifier[0].as_str() == "env" {
        return true;
    }
    false
}

/// 导入加载完成后校验库注解引用。
pub fn validate_macros(
    program: &Program,
    macros: &HashMap<Symbol, Vec<Symbol>>,
    passes: &HashMap<Symbol, Vec<Symbol>>,
    checks: &HashMap<Symbol, Vec<Symbol>>,
) -> Result<()> {
    let tables = AnnotationTables::from_kind_maps(macros, passes, checks);
    let mut out = Ok(());
    for stmt in &program.stmts {
        validate_macros_stmt(stmt, &tables, &mut out);
        if out.is_err() { return out; }
    }
    out
}

fn validate_macros_stmt(stmt: &Stmt, tables: &AnnotationTables, out: &mut Result<()>) {
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
        match tables.resolve_kind(a) {
            Err(e) => { *out = Err(e); return; }
            Ok(None) => {
                *out = Err(Error::Hir(format!(
                    "unknown annotation #[{}]: not exported by any imported package (at {}:{})",
                    a.path_str(), a.span.start_line, a.span.start_col
                )));
                return;
            }
            Ok(Some(AnnKind::Pass)) | Ok(Some(AnnKind::Check)) => {
                // 优化/检查注解保留到 MIR 阶段执行
            }
            Ok(Some(AnnKind::Macro)) => {
                // 宏展开应在 HIR 之前完成；这里出现说明展开遗漏
                *out = Err(Error::Hir(format!(
                    "macro #[{}] was not expanded (internal error) (at {}:{})",
                    a.path_str(), a.span.start_line, a.span.start_col
                )));
                return;
            }
        }
    }
    match stmt {
        Stmt::FnDecl { body, .. } => validate_block(body, tables, out),
        Stmt::If { then_block, elifs, else_block, .. } => {
            validate_block(then_block, tables, out);
            for (_, b) in elifs { validate_block(b, tables, out); }
            if let Some(b) = else_block { validate_block(b, tables, out); }
        }
        Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::ForIn { body, .. } => {
            validate_block(body, tables, out);
        }
        Stmt::Namespace { items, .. } => {
            for s in items { validate_macros_stmt(s, tables, out); if out.is_err() { return; } }
        }
        Stmt::ImplBlock { methods, .. } => {
            for s in methods { validate_macros_stmt(s, tables, out); if out.is_err() { return; } }
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
        Stmt::Attributed { stmt: inner, .. } => validate_macros_stmt(inner, tables, out),
        _ => {}
    }
}

fn validate_block(
    block: &crate::parser::ast::Block,
    tables: &AnnotationTables,
    out: &mut Result<()>,
) {
    for s in &block.stmts {
        validate_macros_stmt(s, tables, out);
        if out.is_err() { return; }
    }
}
