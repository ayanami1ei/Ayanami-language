use crate::hir::ir::{HirType, VarId};

#[derive(Debug, Clone)]
pub enum MemAction {
    Drop(VarId),
}

pub trait MemStrategy {
    fn on_scope_end(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
    fn on_move_out(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
    fn on_clone(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
    fn on_assign_overwrite(&self, var: VarId, ty: &HirType) -> Vec<MemAction>;
}

pub mod drop;
pub mod value;

pub use drop::DropStrategy;
pub use value::ValueStrategy;
