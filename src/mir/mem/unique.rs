use crate::hir::ir::{HirType, VarId};
use super::*;

pub struct UniqueStrategy;

impl MemStrategy for UniqueStrategy {
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
