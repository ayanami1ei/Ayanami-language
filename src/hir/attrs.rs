use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

/// A0 属性白名单。A1 起逐个接入 LLVM 语义；未知属性一律报错（ADR-2）。
pub const ALLOWED: &[&str] = &[
    "cfg",
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

/// 形参位置允许的属性（A1b）。
pub const PARAM_ALLOWED: &[&str] = &["noalias", "nonnull"];

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

/// 校验形参标注：仅 noalias/nonnull，且作用于指针类型（ref/unique/[T]/fn）。
fn validate_param(attrs: &[Attr], ty: &crate::parser::ast::Type) -> Result<()> {
    use crate::parser::ast::Type;
    for a in attrs {
        let name = a.name.as_str();
        if !PARAM_ALLOWED.contains(&name.as_str()) {
            return Err(Error::Hir(format!(
                "attribute #[{}] is not allowed on parameters (at {}:{})",
                name, a.span.start_line, a.span.start_col
            )));
        }
        let is_ptr = matches!(ty, Type::Ref(..) | Type::Unique(..) | Type::Array(..) | Type::FnPtr(..));
        if !is_ptr {
            return Err(Error::Hir(format!(
                "#[{}] requires a pointer parameter (at {}:{})",
                name, a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

fn validate_stmt(stmt: &Stmt) -> Result<()> {
    match stmt {
        Stmt::FnDecl { attrs, params, param_attrs, .. } => {
            validate(attrs)?;
            for (i, pa) in param_attrs.iter().enumerate() {
                if let Some((_, ty)) = params.get(i) {
                    validate_param(pa, ty)?;
                }
            }
        }
        Stmt::StructDef { attrs, .. }
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
