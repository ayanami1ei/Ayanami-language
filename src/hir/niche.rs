//! N1（docs/enum-layout.md §3.1）：niche 枚举形态判定（纯函数，不依赖 Ctx）。
//!
//! niche 表示 = **去掉 `_tag` 字段**的单指针槽：`Option[Ptr[T]]` 布局为
//! `{ %struct.Option_Some, %struct.Option_None }`（8 字节；空变体 = 载荷槽零值 = null）。
//! 字段序即 tag 序（`_data_*` 在字段表中的下标），形态自描述、跨模块一致。
//!
//! 可 niche 载荷（8 字节、可空值位型承载判别、且 null 不是合法值）：
//! `Ptr[T]`（`Unique(具名/数组)`）、`ref T`、`fn(..)`。
//! **不含 `*T`**（null 是合法裸指针值，`Some(null)` 会塌缩为 None——与 Rust 口径一致）。
use crate::hir::item::HirStructField;
use crate::hir::ty::HirType;
use crate::intern::Symbol;
use std::collections::HashMap;

/// niche 元信息（变体字段名与 tag；tag = `_data_*` 字段序）。
#[derive(Debug, Clone)]
pub(crate) struct NicheInfo {
    /// 载荷变体字段（`_data_Some`）
    pub payload_field: Symbol,
    /// 空变体字段（`_data_None`）
    pub empty_field: Symbol,
    /// 载荷变体 tag（字段序）
    pub payload_tag: i64,
    /// 空变体 tag（字段序）
    pub empty_tag: i64,
}

/// 载荷类型是否可用空指针位型承载判别。
pub(crate) fn is_nicheable(ty: &HirType) -> bool {
    match ty {
        HirType::Ref(..) | HirType::FnPtr(..) => true,
        HirType::Unique(inner) => match inner.as_ref() {
            HirType::Named(_) | HirType::FatPtr { .. } | HirType::Closure(..)
            | HirType::Array(_) | HirType::ArraySized(_, _) => true,
            // 8 字节基元：`Ptr[基元]` 仍为堆指针（`ToUnique` 装箱），布局恰为 8；
            // 4/1 字节基元（F32/Bool/Char）按值存储，排除
            HirType::Int | HirType::Float | HirType::IntN { bits: 64, .. } => true,
            _ => false,
        },
        _ => false,
    }
}

/// 已是 niche 形态（无 `_tag`；恰好 2 个 `_data_*`；恰 1 个单字段可 niche 载荷）→ 元信息。
pub(crate) fn niche_info_of(
    fields: &[HirStructField],
    defs: &HashMap<Symbol, Vec<HirStructField>>,
) -> Option<NicheInfo> {
    if fields.first().map(|f| f.name.as_str() == "_tag").unwrap_or(true) {
        return None;
    }
    let data: Vec<(i64, Symbol)> = fields.iter().enumerate()
        .filter(|(_, f)| f.name.as_str().starts_with("_data_"))
        .map(|(i, f)| (i as i64, f.name))
        .collect();
    if data.len() != 2 || data.len() != fields.len() {
        return None;
    }
    let mut payload = None;
    let mut empty = None;
    for (tag, name) in &data {
        let fty = fields.iter().find(|f| f.name == *name).map(|f| f.ty.clone())?;
        let arity = match &fty {
            // N2：boxed 变体（字段本身为指针）视为非空载荷
            HirType::Named(vn) => defs.get(vn).map(|v| v.len())?,
            _ => 1,
        };
        if arity == 0 {
            empty = Some((*tag, *name));
        } else {
            payload = Some((*tag, *name));
        }
    }
    let (payload_tag, payload_field) = payload?;
    let (empty_tag, empty_field) = empty?;
    Some(NicheInfo { payload_field, empty_field, payload_tag, empty_tag })
}

/// tag 形态判定（声明/实例化时调用）：`_tag` + 恰好 2 个 `_data_*`
/// （1 空变体 + 1 单字段可 niche 载荷变体）。返回 (payload_field, empty_field)。
pub(crate) fn niche_shape(
    fields: &[HirStructField],
    defs: &HashMap<Symbol, Vec<HirStructField>>,
) -> Option<(Symbol, Symbol)> {
    if fields.first().map(|f| f.name.as_str() != "_tag").unwrap_or(true) {
        return None;
    }
    let data: Vec<Symbol> = fields.iter()
        .filter(|f| f.name.as_str().starts_with("_data_"))
        .map(|f| f.name)
        .collect();
    if data.len() != 2 {
        return None;
    }
    let mut payload = None;
    let mut empty = None;
    for f in fields.iter().filter(|f| f.name.as_str().starts_with("_data_")) {
        // N2：boxed 变体——字段本身即指针
        if is_nicheable(&f.ty) {
            payload = Some(f.name);
            continue;
        }
        let HirType::Named(vn) = &f.ty else { return None };
        match defs.get(vn).map(|v| v.len()) {
            Some(1) => {
                if !is_nicheable(&defs.get(vn)?[0].ty) {
                    return None;
                }
                payload = Some(f.name);
            }
            Some(0) => empty = Some(f.name),
            _ => return None,
        }
    }
    Some((payload?, empty?))
}
