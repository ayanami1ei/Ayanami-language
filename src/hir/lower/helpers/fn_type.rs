//! M2：可调用类型统一辅助。
//!
//! - 安全代码只允许 `Fn(T)->U`；`fn(...)` 仅出现在 `extern "C"`（含 `#[export]`）签名；
//! - 导入旧包（.lcl）源码里的 `fn(...)` 统一降级为拥有型 `Fn`。

use super::*;

/// extern C 签名专用：顶层 `fn(...)` 保留裸指针语义（FFI 例外；安全代码禁用 `fn` 类型）。
pub(crate) fn ast_type_to_hir_extern(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType {
    if let Type::FnPtr(params, ret, _) = ty {
        return HirType::FnPtr(
            params.iter().map(|p| ast_type_to_hir_extern(p, interfaces)).collect(),
            Box::new(ast_type_to_hir_extern(ret, interfaces)),
        );
    }
    ast_type_to_hir(ty, interfaces)
}

/// 形参/返回类型转换：extern "C" / `#[export]` 时保留裸 `fn(...)`，否则统一为 `Fn`。
pub(crate) fn ast_type_to_hir_param(
    ty: &Type,
    extern_c: bool,
    attrs: &[Attr],
    interfaces: &HashMap<Symbol, InterfaceReg>,
) -> HirType {
    if extern_c || crate::hir::attrs::has(attrs, "export") {
        ast_type_to_hir_extern(ty, interfaces)
    } else {
        ast_type_to_hir(ty, interfaces)
    }
}

/// M2：旧包/HIR 里的裸 `fn` 类型统一为拥有型 `Fn`（导入兼容）。
pub(crate) fn unify_legacy_fn_type(ty: &HirType) -> HirType {
    match ty {
        HirType::FnPtr(ps, ret) => HirType::Closure(
            ps.iter().map(unify_legacy_fn_type).collect(),
            Box::new(unify_legacy_fn_type(ret)),
            true,
            false,
        ),
        HirType::Unique(inner) => HirType::Unique(Box::new(unify_legacy_fn_type(inner))),
        HirType::Array(inner) => HirType::Array(Box::new(unify_legacy_fn_type(inner))),
        HirType::ArraySized(inner, n) => HirType::ArraySized(Box::new(unify_legacy_fn_type(inner)), *n),
        HirType::Ref(inner, m) => HirType::Ref(Box::new(unify_legacy_fn_type(inner)), *m),
        HirType::FatPtr { name, kind } => HirType::FatPtr {
            name: *name,
            kind: Box::new(unify_legacy_fn_type(kind)),
        },
        HirType::Closure(ps, ret, owns, once) => HirType::Closure(
            ps.iter().map(unify_legacy_fn_type).collect(),
            Box::new(unify_legacy_fn_type(ret)),
            *owns,
            *once,
        ),
        other => other.clone(),
    }
}
