use super::*;

/// 类型离开作用域时是否需要释放（与 LIR 的递归 drop 规则一致）。
///
/// 枚举（首字段 `_tag`）的 payload 释放尚未实现，暂不自动 drop。
fn needs_drop(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool {
    match ty {
        HirType::Unique(_) | HirType::Shared(_) => true,
        HirType::FatPtr { kind, .. } => !matches!(kind.as_ref(), HirType::Ref(..)),
        HirType::Named(name) => {
            let Some(fields) = struct_defs.get(name) else { return false; };
            fields.iter().any(|(_, ft)| needs_drop(ft, struct_defs))
        }
        _ => false,
    }
}

pub(super) fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy> {
    match ty {
        HirType::Shared(_) => Box::new(SharedStrategy),
        _ if needs_drop(ty, struct_defs) => Box::new(DropStrategy),
        _ => Box::new(ValueStrategy),
    }
}

pub(super) fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmtBox {
    match action {
        MemAction::Drop(v) => SMirDropStmt { var: *v, ty: ty.clone() }.into(),
        MemAction::Retain(v) => SMirRetainStmt { var: *v, ty: ty.clone() }.into(),
        MemAction::Release(v) => SMirReleaseStmt { var: *v, ty: ty.clone() }.into(),
    }
}
