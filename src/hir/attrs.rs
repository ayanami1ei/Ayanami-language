use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

/// A0 属性白名单。A1 起逐个接入 LLVM 语义；未知属性一律报错（ADR-2）。
pub const ALLOWED: &[&str] = &[
    "inline",
    "cold",
    "noreturn",
    "pure",
    "readonly",
    "nounwind",
    "willreturn",
    "noalias",
    "nonnull",
];

/// 属性是否出现在列表中。
pub fn has(attrs: &[Attr], name: &str) -> bool {
    attrs.iter().any(|a| a.name.as_str() == name)
}

/// 校验属性名是否在白名单内。
pub fn validate(attrs: &[Attr]) -> Result<()> {
    for a in attrs {
        if !ALLOWED.contains(&a.name.as_str().as_str()) {
            return Err(Error::Hir(format!(
                "unknown attribute #[{}] (at {}:{})",
                a.name.as_str(),
                a.span.start_line,
                a.span.start_col
            )));
        }
    }
    Ok(())
}

/// 递归校验整个程序的声明属性（含命名空间/impl/接口方法）。
pub fn validate_program(program: &Program) -> Result<()> {
    for stmt in &program.stmts {
        validate_stmt(stmt)?;
    }
    Ok(())
}

fn validate_stmt(stmt: &Stmt) -> Result<()> {
    match stmt {
        Stmt::FnDecl { attrs, .. }
        | Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => validate(attrs)?,
        _ => {}
    }
    match stmt {
        Stmt::Namespace { items, .. } => {
            for s in items {
                validate_stmt(s)?;
            }
        }
        Stmt::ImplBlock { methods, .. } => {
            for s in methods {
                validate_stmt(s)?;
            }
        }
        Stmt::InterfaceDef { methods, .. } => {
            for m in methods {
                validate(&m.attrs)?;
            }
        }
        _ => {}
    }
    Ok(())
}
