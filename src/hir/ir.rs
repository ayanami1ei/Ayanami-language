// Compatibility re-exports — all HIR types moved to submodules.
pub use super::ty::*;
pub use super::node::*;
pub use super::stmt::*;
pub use super::item::*;
pub use super::{SBin, SUn, SCall, SConst, SVar, SMove, SClone, SToUnique, SToShared, SToWeak,
    SField, SStruct, SArrLit, SArrSz, SAsm, SRef, SIdx, SVCall, SMFP, SEnumC, SEnumM, SFnPtr, SCallP};
