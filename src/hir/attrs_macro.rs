//! A5b/A5d：库注解引用校验（导入加载后）。
//!
//! - 宏注解（`#[macro]`）必须已被宏展开消费，残留即内部错误；
//! - 优化注解（`#[pass]`）保留到 MIR 阶段执行，这里只校验存在性；
//! - 裸名自动作用域：跨所有已导入包查找，重名报歧义。

use std::collections::HashMap;

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
    false
}

/// 导入加载完成后校验库注解引用。
pub fn validate_macros(
    program: &Program,
    macros: &HashMap<Symbol, Vec<Symbol>>,
    passes: &HashMap<Symbol, Vec<Symbol>>,
    checks: &HashMap<Symbol, Vec<Symbol>>,
) -> Result<()> {
    let mut out = Ok(());
    for stmt in &program.stmts {
        validate_macros_stmt(stmt, macros, passes, checks, &mut out);
        if out.is_err() { return out; }
    }
    out
}

/// 注解类别
enum AnnKind { Macro, Pass, Check }

/// 查找注解：全限定查指定包；裸名跨所有已导入包。
/// 返回 Ok(Some(kind)) / Ok(None)（未找到）/ Err（裸名歧义）。
fn lookup_annotation(
    a: &Attr,
    macros: &HashMap<Symbol, Vec<Symbol>>,
    passes: &HashMap<Symbol, Vec<Symbol>>,
    checks: &HashMap<Symbol, Vec<Symbol>>,
) -> Result<Option<AnnKind>> {
    let name = a.name.as_str();
    if !a.qualifier.is_empty() {
        let pkg = a.qualifier[0];
        let rest: Vec<String> = a.qualifier[1..].iter().map(|s| s.as_str()).collect();
        let qualified = if rest.is_empty() { name.clone() } else { format!("{}.{}", rest.join("."), name) };
        if macros.get(&pkg).map_or(false, |ms| ms.iter().any(|m| m.as_str() == qualified)) {
            return Ok(Some(AnnKind::Macro));
        }
        if passes.get(&pkg).map_or(false, |ms| ms.iter().any(|m| m.as_str() == qualified)) {
            return Ok(Some(AnnKind::Pass));
        }
        if checks.get(&pkg).map_or(false, |ms| ms.iter().any(|m| m.as_str() == qualified)) {
            return Ok(Some(AnnKind::Check));
        }
        return Ok(None);
    }
    let mut hits: Vec<(String, AnnKind)> = Vec::new();
    for (pkg, ms) in macros {
        if ms.iter().any(|m| m.as_str() == name) {
            hits.push((pkg.as_str(), AnnKind::Macro));
        }
    }
    for (pkg, ms) in passes {
        if ms.iter().any(|m| m.as_str() == name) {
            hits.push((pkg.as_str(), AnnKind::Pass));
        }
    }
    for (pkg, ms) in checks {
        if ms.iter().any(|m| m.as_str() == name) {
            hits.push((pkg.as_str(), AnnKind::Check));
        }
    }
    match hits.len() {
        0 => Ok(None),
        1 => Ok(Some(match hits[0].1 {
            AnnKind::Macro => AnnKind::Macro,
            AnnKind::Pass => AnnKind::Pass,
            AnnKind::Check => AnnKind::Check,
        })),
        _ => {
            let list: Vec<String> = hits.iter().map(|(p, _)| format!("{}::{}", p, name)).collect();
            Err(Error::Hir(format!(
                "ambiguous annotation #[{}]: exported by multiple packages ({}); use a qualified name (at {}:{})",
                name, list.join(", "), a.span.start_line, a.span.start_col
            )))
        }
    }
}

fn validate_macros_stmt(
    stmt: &Stmt,
    macros: &HashMap<Symbol, Vec<Symbol>>,
    passes: &HashMap<Symbol, Vec<Symbol>>,
    checks: &HashMap<Symbol, Vec<Symbol>>,
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
        match lookup_annotation(a, macros, passes, checks) {
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
        Stmt::FnDecl { body, .. } => validate_block(body, macros, passes, checks, out),
        Stmt::If { then_block, elifs, else_block, .. } => {
            validate_block(then_block, macros, passes, checks, out);
            for (_, b) in elifs { validate_block(b, macros, passes, checks, out); }
            if let Some(b) = else_block { validate_block(b, macros, passes, checks, out); }
        }
        Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::ForIn { body, .. } => {
            validate_block(body, macros, passes, checks, out);
        }
        Stmt::Namespace { items, .. } => {
            for s in items { validate_macros_stmt(s, macros, passes, checks, out); if out.is_err() { return; } }
        }
        Stmt::ImplBlock { methods, .. } => {
            for s in methods { validate_macros_stmt(s, macros, passes, checks, out); if out.is_err() { return; } }
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
        Stmt::Attributed { stmt: inner, .. } => validate_macros_stmt(inner, macros, passes, checks, out),
        _ => {}
    }
}

fn validate_block(
    block: &crate::parser::ast::Block,
    macros: &HashMap<Symbol, Vec<Symbol>>,
    passes: &HashMap<Symbol, Vec<Symbol>>,
    checks: &HashMap<Symbol, Vec<Symbol>>,
    out: &mut Result<()>,
) {
    for s in &block.stmts {
        validate_macros_stmt(s, macros, passes, checks, out);
        if out.is_err() { return; }
    }
}
