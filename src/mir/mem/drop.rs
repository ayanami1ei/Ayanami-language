use crate::hir::ir::{HirType, VarId};
use super::*;

/// Strategy for values that own heap memory (unique pointers, structs with
/// owned fields, fat pointers). LIR emits the actual recursive drop code.
pub struct DropStrategy;

impl MemStrategy for DropStrategy {
    fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![MemAction::Drop(var)]
    }
    fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
    fn on_clone(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
    fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![MemAction::Drop(var)]
    }
}
