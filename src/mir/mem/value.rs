use crate::hir::ir::{HirType, VarId};
use super::*;

pub struct ValueStrategy;

impl MemStrategy for ValueStrategy {
    fn on_scope_end(&self, _var: VarId, _ty: &HirType) -> Vec<MemAction> {
        vec![]
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
