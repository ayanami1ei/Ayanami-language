use super::*;

/// 类型离开作用域时是否需要释放。
///
/// P1 只对显式所有权（`unique`/`shared`/胖指针）自动释放；
/// 普通结构体值（如 `String`）暂不自动 drop，等 P2 引入默认 move 语义后再开启，
/// 否则 `t = s` 这类复制会导致双释放。
fn needs_drop(ty: &HirType, _struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> bool {
    matches!(ty, HirType::Unique(_) | HirType::Shared(_) | HirType::FatPtr { .. })
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
