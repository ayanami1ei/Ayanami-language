//! A4b：`#[follow_with]` 来源匹配（参数名优先；类型名唯一匹配作简写）。

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::ty::{HirType, VarId};
use crate::intern::Symbol;
use crate::hir::ty::FnId;
use crate::mir::ir::{MirFn, MirItem, MirProgram};

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

/// 被调函数的来源解析结果：来源名 → 实参位置。
#[derive(Debug, Clone, Default)]
pub struct FollowInfo {
    pub resolved: Vec<(Symbol, usize)>,
}

/// 收集整个程序的 `follow_with` 表（跨函数传播用）。
pub fn build_table(mir: &MirProgram) -> HashMap<FnId, FollowInfo> {
    let mut out = HashMap::new();
    collect_items(&mir.items, &mut out);
    out
}

fn collect_items(items: &[MirItem], out: &mut HashMap<FnId, FollowInfo>) {
    for item in items {
        match item {
            MirItem::Fn(f) => {
                if f.follow_sources.is_empty() {
                    continue;
                }
                let ref_params: Vec<(VarId, bool)> = f.params.iter().enumerate()
                    .filter_map(|(i, (_, ty))| match ty {
                        HirType::Ref(_, m) => Some((VarId(i), *m)),
                        _ => None,
                    })
                    .collect();
                let resolved = f.follow_sources.iter()
                    .filter_map(|s| match_source(f, s, &ref_params).ok().map(|v| (*s, v.0)))
                    .collect();
                out.insert(f.fn_id, FollowInfo { resolved });
            }
            MirItem::Namespace { items, .. } => collect_items(items, out),
            _ => {}
        }
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
