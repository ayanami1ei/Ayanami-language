use crate::hir::ir::{HirType, VarId};
use crate::intern::Symbol;
use super::*;

pub struct ValueStrategy;

impl MemStrategy for ValueStrategy {
    fn on_scope_end(&self, var: VarId, ty: &HirType) -> Vec<MemAction> {
        // Named types (structs) may contain owned fields that need cleanup
        if matches!(ty, HirType::Named(_)) {
            vec![MemAction::Drop(var)]
        } else {
            vec![]
        }
    }
    fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
    fn on_clone(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
    fn on_assign_overwrite(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
}
