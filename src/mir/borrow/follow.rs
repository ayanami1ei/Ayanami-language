//! A4b：`#[follow_with]` 来源匹配（参数名优先；类型名唯一匹配作简写）。

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::ty::{HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirFn;

/// 将来源名解析为引用参数 VarId。
pub fn match_source(
    mir_fn: &MirFn,
    source: &Symbol,
    ref_params: &[(VarId, bool)],
) -> Result<VarId> {
    // 1) 参数名精确匹配
    for (i, (name, _)) in mir_fn.params.iter().enumerate() {
        if *name == *source && ref_params.iter().any(|(v, _)| v.0 == i) {
            return Ok(VarId(i));
        }
    }
    // 2) 类型名（唯一匹配）
    let mut matched: Vec<VarId> = Vec::new();
    for (i, (_, ty)) in mir_fn.params.iter().enumerate() {
        if !ref_params.iter().any(|(v, _)| v.0 == i) {
            continue;
        }
        if let HirType::Ref(inner, _) = ty {
            if let HirType::Named(n) = inner.as_ref() {
                if *n == *source {
                    matched.push(VarId(i));
                }
            }
        }
    }
    match matched.len() {
        0 => Err(Error::Borrow(format!(
            "#[follow_with({})] does not match any reference parameter of `{}`",
            source.as_str(),
            mir_fn.name.as_str()
        ))),
        1 => Ok(matched[0]),
        _ => Err(Error::Borrow(format!(
            "#[follow_with({})] is ambiguous: multiple reference parameters have this type; use parameter names",
            source.as_str()
        ))),
    }
}

/// 供未来跨函数传播使用：来源名 → 参数名集合。
pub fn source_names(mir_fn: &MirFn) -> HashMap<VarId, Symbol> {
    mir_fn
        .params
        .iter()
        .enumerate()
        .map(|(i, (n, _))| (VarId(i), *n))
        .collect()
}
