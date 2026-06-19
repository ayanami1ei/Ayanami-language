pub mod ir;
pub mod ty;
pub mod node;
pub mod stmt;
pub mod item;
pub mod lower;
pub mod display;

pub use ty::*;
pub use node::*;
pub use stmt::*;
pub use item::*;
pub use lower::lower_program;
pub use display::{display_hir_program, hir_program_to_string};

use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::hir::ty::{VarId, FnId, HirType, HirLiteral};

// ── Struct-based IR nodes ──

macro_rules! s_hir {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, asuka::IrNode)]
        pub struct $name {
            $(pub $field: $ty),*
        }
    };
    ($name:ident { $($field:ident: $ty:ty),* }) => { s_hir!($name { $($field: $ty),* , }); };
}

s_hir!(SBin { op: BinaryOp, lhs: HirNodeBox, rhs: HirNodeBox, ty: HirType });
s_hir!(SUn { op: UnaryOp, arg: HirNodeBox, ty: HirType });
s_hir!(SCall { fn_id: FnId, args: Vec<HirNodeBox>, ty: HirType });
s_hir!(SConst { val: HirLiteral, ty: HirType });
s_hir!(SVar { var: VarId, ty: HirType });
s_hir!(SMove { expr: HirNodeBox, ty: HirType });
s_hir!(SClone { expr: HirNodeBox, ty: HirType });
s_hir!(SToUnique { expr: HirNodeBox, ty: HirType });
s_hir!(SToShared { expr: HirNodeBox, ty: HirType });
s_hir!(SToWeak { expr: HirNodeBox, ty: HirType });
s_hir!(SField { object: HirNodeBox, field: Symbol, field_index: usize, ty: HirType });
s_hir!(SStruct { type_name: Symbol, fields: Vec<(Symbol, HirNodeBox)>, ty: HirType });
s_hir!(SArrLit { elems: Vec<HirNodeBox>, ty: HirType });
s_hir!(SArrSz { count: HirNodeBox, elem_ty: HirType, ty: HirType });
s_hir!(SAsm { template: String, outputs: Vec<(String, HirNodeBox)>, inputs: Vec<(String, HirNodeBox)>, ty: HirType });
s_hir!(SRef { expr: HirNodeBox, mutable: bool, ty: HirType });
s_hir!(SIdx { object: HirNodeBox, index: HirNodeBox, ty: HirType });
s_hir!(SVCall { receiver: HirNodeBox, interface: Symbol, method_index: usize, args: Vec<HirNodeBox>, concrete_type: Symbol, ty: HirType });
s_hir!(SMFP { value: HirNodeBox, concrete_type: Symbol, interface_name: Symbol, ty: HirType });
s_hir!(SEnumC { enum_name: Symbol, variant_name: Symbol, variant_struct: Symbol, args: Vec<HirNodeBox>, ty: HirType });
s_hir!(SEnumM { value: HirNodeBox, arms: Vec<(i64, HirNodeBox)>, ty: HirType });
s_hir!(SFnPtr { fn_id: FnId, ty: HirType });
s_hir!(SCallP { fn_ptr: HirNodeBox, args: Vec<HirNodeBox>, ty: HirType });
