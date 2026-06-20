use crate::hir::ir::{HirType, VarId};
use super::*;

/// Strategy for struct-typed variables.
/// When the struct goes out of scope, each field that has ownership
/// (Shared/Unique) needs proper cleanup.
pub struct StructStrategy {
    /// (field_index, field_type) for fields that need cleanup
    pub fields: Vec<(usize, HirType)>,
}

impl MemStrategy for StructStrategy {
    fn on_scope_end(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        self.fields.iter().map(|(_, ft)| match ft {
            HirType::Shared(_) => MemAction::Release(var),
            HirType::Unique(_) => MemAction::Drop(var),
            _ => unreachable!(),
        }).collect()
    }

    fn on_move_out(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        // Moving a struct out — no cleanup needed, ownership transfers
        vec![]
    }

    fn on_clone(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        // Cloning a struct: retain shared fields, unique fields are cloned
        self.fields.iter().map(|(_, ft)| match ft {
            HirType::Shared(_) => MemAction::Retain(var),
            HirType::Unique(_) => MemAction::Drop(var), // unique clone = deep copy
            _ => unreachable!(),
        }).collect()
    }

    fn on_assign_overwrite(&self, var: VarId, _ty: &HirType) -> Vec<MemAction> {
        // Overwriting a struct: release/drop old field values
        self.fields.iter().map(|(_, ft)| match ft {
            HirType::Shared(_) => MemAction::Release(var),
            HirType::Unique(_) => MemAction::Drop(var),
            _ => unreachable!(),
        }).collect()
    }
}
