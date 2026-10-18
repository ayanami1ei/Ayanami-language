//! `#[copy]` 结构体注册（dld）：trivial Copy 句柄类型。
//!
//! 校验：仅结构体、非泛型、所有字段类型为 Copy（基元/引用/指针/已登记的 `#[copy]`
//! 结构体——按源码顺序，前向引用报错）。注册表供 `HirType::is_copy` 使用。
use super::*;
use crate::parser::ast::{Attr, Stmt};

/// 扫描（cfg 裁剪后的）程序，登记全部 `#[copy]` 结构体。
pub(crate) fn register_copy_structs(stmts: &[Stmt]) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::StructDef { name, fields, attrs, generic_params, span, .. } => {
                if !attrs.iter().any(|a| is_copy_attr(a)) {
                    continue;
                }
                if !generic_params.is_empty() {
                    return Err(Error::Hir(format!(
                        "#[copy] struct cannot be generic (at {}:{})",
                        span.start_line, span.start_col
                    )));
                }
                let ifaces = std::collections::HashMap::new();
                for (fname, fty) in fields {
                    let hir_ty = ast_type_to_hir(fty, &ifaces);
                    if !hir_ty.is_copy() {
                        return Err(Error::Hir(format!(
                            "#[copy] struct `{}` field `{}` has non-Copy type `{}` (at {}:{})",
                            name.as_str(), fname.as_str(), hir_type_display(&hir_ty),
                            span.start_line, span.start_col
                        )));
                    }
                }
                crate::hir::ty::register_copy_struct(*name);
            }
            Stmt::Namespace { items, .. } => register_copy_structs(items)?,
            _ => {}
        }
    }
    Ok(())
}

/// `#[copy]` 内置标注（无实参）。
fn is_copy_attr(a: &Attr) -> bool {
    a.is_builtin() && a.name.as_str() == "copy"
}
