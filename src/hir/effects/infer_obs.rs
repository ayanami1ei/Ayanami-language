//! #206：写目标是否“外部可见”（引用/拥有指针/裸指针，沿解引用/字段/索引链回溯根）。
//! 从 infer.rs 拆出，保持文件 ≤300 行。
use crate::hir::{HirNodeBox, HirType};

pub(super) fn is_observable_target(object: &HirNodeBox) -> bool {
    // 引用/拥有指针，或**裸指针**（`*p = v` / `(*p).f = v`）→ 外部可见写
    if matches!(object.expr_type(),
        HirType::Ref(..) | HirType::Unique(..) | HirType::RawPtr(..))
    {
        return true;
    }
    // 解引用/字段/索引链回到底层指针（`(*p).f` / `r.a.b`）
    if let Some(inner) = object.as_deref() {
        return is_observable_target(inner);
    }
    if let Some((obj, _, _)) = object.as_field_access() {
        return is_observable_target(obj);
    }
    if let Some((obj, _)) = object.as_index() {
        return is_observable_target(obj);
    }
    false
}
