//! 类型布局（size/align）——自然对齐规则，与 `%struct.*` 的 LLVM 结构体布局一致。
//! HIR 内置（`size_of[T]()`/`align_of[T]()`）与 LIR 数组元素布局共用这一实现。
use crate::intern::Symbol;
use crate::hir::ty::HirType;
use std::collections::HashMap;

/// 结构体字段表（名称 → 类型）——HIR/LIR 两层的通用形状。
pub(crate) type StructDefs = HashMap<Symbol, Vec<(Symbol, HirType)>>;

/// 类型布局：`(size, align)`（字节；自然对齐）。
/// - 基元/指针/引用按固定尺寸；`IntN` 按位宽；`[T]`/`[T; n]` 值为指针（8）；
/// - `FatPtr`/闭包为胖指针（16）；
/// - `Named`：结构体按字段顺序自然对齐（union 标记 `__union` 取各字段最大）；
/// - 未知结构回退 `(16, 8)`（保守）。
pub(crate) fn type_layout(ty: &HirType, defs: &StructDefs) -> (u64, u64) {
    match ty {
        HirType::Void | HirType::Never => (0, 1),
        HirType::Char | HirType::Bool => (1, 1),
        HirType::Int | HirType::Float | HirType::Ref(_, _) | HirType::FnPtr(..) | HirType::RawPtr(_) => (8, 8),
        HirType::F32 => (4, 4),
        HirType::IntN { bits, .. } => { let sz = (*bits / 8) as u64; (sz, sz.min(16)) }
        HirType::FatPtr { .. } | HirType::Closure(..) => (16, 8),
        HirType::Array(_) | HirType::ArraySized(_, _) => (8, 8),
        HirType::Unique(inner) => match &**inner {
            HirType::Named(_) | HirType::FatPtr { .. }
            | HirType::Array(_) | HirType::ArraySized(_, _) => (8, 8),
            _ => type_layout(inner, defs),
        },
        HirType::Named(name) => {
            let fields = defs.get(name).or_else(|| {
                let n = name.as_str();
                n.find('<').map(|pos| Symbol::intern(&n[..pos])).and_then(|b| defs.get(&b))
            });
            match fields {
                Some(fs) => {
                    // P3b：union —— 字段共享存储：size/align 取各字段最大
                    if fs.first().map(|(n, _)| n.as_str() == "__union").unwrap_or(false) {
                        let mut size = 0u64;
                        let mut align = 1u64;
                        for (_, ft) in fs {
                            let (sz, a) = type_layout(ft, defs);
                            size = size.max(sz);
                            align = align.max(a);
                        }
                        return ((size + align - 1) / align * align, align);
                    }
                    let mut off = 0u64;
                    let mut align = 1u64;
                    for (_, ft) in fs {
                        let (sz, a) = type_layout(ft, defs);
                        off = (off + a - 1) / a * a + sz;
                        align = align.max(a);
                    }
                    ((off + align - 1) / align * align, align)
                }
                None => (16, 8), // 未知结构保守回退
            }
        }
    }
}

/// 字段偏移（字节）：仅支持结构体（`Named` 且字段表已知）。
/// 自然对齐规则与 `type_layout` 一致；字段不存在或类型未知返回 `None`。
pub(crate) fn field_offset(ty: &HirType, field: Symbol, defs: &StructDefs) -> Option<u64> {
    let HirType::Named(name) = ty else { return None };
    let fields = defs.get(name).or_else(|| {
        let n = name.as_str();
        n.find('<').map(|pos| Symbol::intern(&n[..pos])).and_then(|b| defs.get(&b))
    })?;
    let mut off = 0u64;
    for (fname, fty) in fields {
        let (sz, a) = type_layout(fty, defs);
        off = (off + a - 1) / a * a;
        if *fname == field {
            return Some(off);
        }
        off += sz;
    }
    None
}
