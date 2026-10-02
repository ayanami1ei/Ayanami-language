use super::*;

pub(super) fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy> {
    match ty {
        HirType::Unique(_) => Box::new(UniqueStrategy),
        HirType::Shared(_) => Box::new(SharedStrategy),
        HirType::Weak(_) => Box::new(ValueStrategy),
        HirType::Named(s) => {
            // Named struct — generate cleanup for each field
            if let Some(fields) = struct_defs.get(s) {
                let actions: Vec<(usize, HirType)> = fields.iter().enumerate()
                    .filter_map(|(i, (_, ft))| match ft {
                        HirType::Shared(_) | HirType::Unique(_) => Some((i, ft.clone())),
                        _ => None,
                    })
                    .collect();
                if actions.is_empty() {
                    Box::new(ValueStrategy)
                } else {
                    Box::new(StructStrategy { fields: actions })
                }
            } else {
                Box::new(ValueStrategy)
            }
        }
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
