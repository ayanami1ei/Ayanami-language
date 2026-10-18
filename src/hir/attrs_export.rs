//! `#[export]` 校验（#84 ③）：仅允许非泛型函数声明。
use crate::error::{Error, Result};
use crate::parser::ast::Attr;

/// `#[export]` / `#[extern]`（C ABI 标注，同规则）
fn find_export(attrs: &[Attr]) -> Option<&Attr> {
    attrs.iter().find(|a| {
        if !a.is_builtin() { return false; }
        let n = a.name.as_str();
        n == "export" || n == "extern"
    })
}

/// 函数声明：`#[export]`/`#[extern]` 不允许泛型。
pub(super) fn validate_fn_export(attrs: &[Attr], has_generics: bool) -> Result<()> {
    if let Some(a) = find_export(attrs) {
        if has_generics {
            return Err(Error::Hir(format!(
                "#[{}] functions cannot be generic (at {}:{})",
                a.name.as_str(), a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

/// 函数声明：`#[compile_time]` 不允许泛型。
pub(super) fn validate_compile_time(attrs: &[Attr], has_generics: bool) -> Result<()> {
    if let Some(a) = attrs.iter().find(|a| a.is_builtin() && a.name.as_str() == "compile_time") {
        if has_generics {
            return Err(Error::Hir(format!(
                "#[compile_time] functions cannot be generic (at {}:{})",
                a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

/// 非函数声明：`#[compile_time]` 一律拒绝。
pub(super) fn reject_compile_time(attrs: &[Attr], place: &str) -> Result<()> {
    if let Some(a) = attrs.iter().find(|a| a.is_builtin() && a.name.as_str() == "compile_time") {
        return Err(Error::Hir(format!(
            "#[compile_time] is only allowed on functions (not on {}) (at {}:{})",
            place, a.span.start_line, a.span.start_col
        )));
    }
    Ok(())
}

/// 非函数声明：`#[export]` 一律拒绝。
pub(super) fn reject_export(attrs: &[Attr], place: &str) -> Result<()> {
    if let Some(a) = find_export(attrs) {
        return Err(Error::Hir(format!(
            "#[{}] is only allowed on functions (not on {}) (at {}:{})",
            a.name.as_str(), place, a.span.start_line, a.span.start_col
        )));
    }
    Ok(())
}
