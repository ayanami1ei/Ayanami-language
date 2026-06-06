use crate::hir::ir::{HirType, VarId};
use super::*;

pub struct SharedStrategy;

impl MemStrategy for SharedStrategy {
    fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![MemAction::Release(var)]
    }
    fn on_move_out(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
    }
    fn on_clone(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![MemAction::Retain(var)]
    }
    fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![MemAction::Release(var)]
    }
}
