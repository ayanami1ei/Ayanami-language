// ── HirNode trait — all HIR nodes implement this ──

use std::fmt::Write as FmtWrite;
use crate::mir::ir::MirNodeBox;
use std::collections::HashSet;
use crate::hir::ty::{VarId, HirType, HirLiteral};

pub trait HirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn HirNode>;
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox;
    fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn expr_type(&self) -> HirType;

    // Helper methods for pattern-match-free traversal
    fn as_local(&self) -> Option<VarId> { None }
    fn is_move_or_clone(&self) -> bool { false }
    fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {}
    fn record_moves(&self, moved: &mut HashSet<VarId>) {}
    fn as_const(&self) -> Option<&HirLiteral> { None }
    fn as_move(&self) -> Option<&HirNodeBox> { None }
    fn as_clone(&self) -> Option<&HirNodeBox> { None }
}

/// Clone-safe wrapper for Box<dyn HirNode>
#[derive(Debug)]
pub struct HirNodeBox(pub Box<dyn HirNode>);
impl Clone for HirNodeBox {
    fn clone(&self) -> Self { HirNodeBox(self.0.clone_node()) }
}
impl std::ops::Deref for HirNodeBox {
    type Target = dyn HirNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}
impl From<HirNodeBox> for Box<dyn HirNode> {
    fn from(b: HirNodeBox) -> Self { b.0 }
}

use crate::hir::{SBin, SUn, SCall, SConst, SVar, SMove, SClone, SToUnique, SToShared, SToWeak, SField, SStruct, SArrLit, SArrSz, SAsm, SRef, SIdx, SVCall, SMFP, SEnumC, SEnumM, SFnPtr, SCallP};

// ── From<Struct> for HirNodeBox ──
macro_rules! impl_into_hir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for HirNodeBox {
            fn from(v: $ty) -> Self { HirNodeBox(Box::new(v) as Box<dyn HirNode>) }
        })*
    };
}
impl_into_hir_node_box!(SBin, SUn, SCall, SConst, SVar, SMove, SClone, SToUnique, SToShared, SToWeak, SField, SStruct, SArrLit, SArrSz, SAsm, SRef, SIdx, SVCall, SMFP, SEnumC, SEnumM, SFnPtr, SCallP);
