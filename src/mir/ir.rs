use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;

pub use crate::hir::ir::{HirLiteral, HirType};
pub use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::hir::ir::{FnId, VarId};
use crate::intern::Symbol;
use crate::lir::ir::{LirValue, LirLowerCtx};

pub trait MirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn MirNode>;
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue;
    fn display(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn expr_type(&self) -> HirType;
    fn as_local(&self) -> Option<VarId> { None }
    fn collect_var_ids(&self, vars: &mut HashSet<VarId>) {}
    fn record_moves(&self, moved: &mut HashSet<VarId>) {}
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
    fn as_string_literal(&self) -> Option<&str> { None }
    fn as_ref(&self) -> Option<(VarId, bool)> { None }
}

#[derive(Debug)]
pub struct MirNodeBox(pub Box<dyn MirNode>);
impl Clone for MirNodeBox {
    fn clone(&self) -> Self { MirNodeBox(self.0.clone_node()) }
}
impl std::ops::Deref for MirNodeBox {
    type Target = dyn MirNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}

pub trait MirStmtNode: std::fmt::Debug {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode>;
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx);
    fn display_stmt(&self, level: usize, w: &mut dyn FmtWrite) -> std::fmt::Result;
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {}
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) {}
    fn is_return(&self) -> bool { false }
    fn return_value(&self) -> Option<&MirNodeBox> { None }
    fn as_drop(&self) -> Option<(VarId, &HirType)> { None }
    fn as_retain(&self) -> Option<(VarId, &HirType)> { None }
    fn as_release(&self) -> Option<(VarId, &HirType)> { None }
}

#[derive(Debug)]
pub struct MirStmtBox(pub Box<dyn MirStmtNode>);
impl Clone for MirStmtBox {
    fn clone(&self) -> Self { MirStmtBox(self.0.clone_stmt()) }
}
impl std::ops::Deref for MirStmtBox {
    type Target = dyn MirStmtNode;
    fn deref(&self) -> &Self::Target { &*self.0 }
}

macro_rules! s_mir {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
    ($name:ident { $($field:ident: $ty:ty),* }) => { s_mir!($name { $($field: $ty),* , }); };
}

macro_rules! s_mstmt {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
    ($name:ident { $($field:ident: $ty:ty),* }) => { s_mstmt!($name { $($field: $ty),* , }); };
}

// 23 SMir* expression structs
s_mir!(SMirLocal { var: VarId, ty: HirType, moved: bool });
s_mir!(SMirLiteral { val: HirLiteral, ty: HirType });
s_mir!(SMirBinary { op: BinaryOp, lhs: MirNodeBox, rhs: MirNodeBox, ty: HirType });
s_mir!(SMirUnary { op: UnaryOp, arg: MirNodeBox, ty: HirType });
s_mir!(SMirCall { fn_id: FnId, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirMove { expr: MirNodeBox, ty: HirType });
s_mir!(SMirClone { expr: MirNodeBox, ty: HirType });
s_mir!(SMirToUnique { expr: MirNodeBox, ty: HirType });
s_mir!(SMirToShared { expr: MirNodeBox, ty: HirType });
s_mir!(SMirToWeak { expr: MirNodeBox, ty: HirType });
s_mir!(SMirVirtualCall { receiver: MirNodeBox, interface: Symbol, method_index: usize, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirMakeFatPtr { value: MirNodeBox, concrete_type: Symbol, interface_name: Symbol, ty: HirType });
s_mir!(SMirEnumConstruct { enum_name: Symbol, variant_name: Symbol, variant_struct: Symbol, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirFnPtr { fn_id: FnId, ty: HirType });
s_mir!(SMirCallPtr { fn_ptr: MirNodeBox, args: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirEnumMatch { value: MirNodeBox, arms: Vec<(i64, MirNodeBox)>, ty: HirType });
s_mir!(SMirFieldAccess { object: MirNodeBox, field: Symbol, field_index: usize, ty: HirType });
s_mir!(SMirStructLiteral { type_name: Symbol, fields: Vec<(Symbol, MirNodeBox)>, ty: HirType });
s_mir!(SMirArrayLiteral { elems: Vec<MirNodeBox>, ty: HirType });
s_mir!(SMirArraySized { count: MirNodeBox, elem_ty: HirType, ty: HirType });
s_mir!(SMirRef { expr: MirNodeBox, mutable: bool, ty: HirType });
s_mir!(SMirIndex { object: MirNodeBox, index: MirNodeBox, ty: HirType });
s_mir!(SMirAsm { template: String, outputs: Vec<(String, MirNodeBox)>, inputs: Vec<(String, MirNodeBox)>, ty: HirType });

// 13 SMir*Stmt structs
s_mstmt!(SMirAssignStmt { target: MirNodeBox, value: MirNodeBox });
s_mstmt!(SMirFieldAssignStmt { object: MirNodeBox, field: Symbol, field_index: usize, field_ty: HirType, value: MirNodeBox });
s_mstmt!(SMirIndexAssignStmt { object: MirNodeBox, index: MirNodeBox, value: MirNodeBox });
s_mstmt!(SMirReturnStmt { value: Option<MirNodeBox> });
s_mstmt!(SMirIfStmt { cond: MirNodeBox, then_block: Vec<MirStmtBox>, elifs: Vec<(MirNodeBox, Vec<MirStmtBox>)>, else_block: Option<Vec<MirStmtBox>> });
s_mstmt!(SMirWhileStmt { cond: MirNodeBox, body: Vec<MirStmtBox> });
s_mstmt!(SMirBreakStmt { });
s_mstmt!(SMirContinueStmt { });
s_mstmt!(SMirExprStmt { expr: MirNodeBox });
s_mstmt!(SMirBlockStmt { stmts: Vec<MirStmtBox> });
s_mstmt!(SMirDropStmt { var: VarId, ty: HirType });
s_mstmt!(SMirRetainStmt { var: VarId, ty: HirType });
s_mstmt!(SMirReleaseStmt { var: VarId, ty: HirType });

macro_rules! impl_into_mir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for MirNodeBox {
            fn from(v: $ty) -> Self { MirNodeBox(Box::new(v) as Box<dyn MirNode>) }
        })*
    };
}
impl_into_mir_node_box!(SMirLocal, SMirLiteral, SMirBinary, SMirUnary, SMirCall, SMirMove, SMirClone, SMirToUnique, SMirToShared, SMirToWeak, SMirVirtualCall, SMirMakeFatPtr, SMirEnumConstruct, SMirFnPtr, SMirCallPtr, SMirEnumMatch, SMirFieldAccess, SMirStructLiteral, SMirArrayLiteral, SMirArraySized, SMirRef, SMirIndex, SMirAsm);

macro_rules! impl_into_mir_stmt_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for MirStmtBox {
            fn from(v: $ty) -> Self { MirStmtBox(Box::new(v) as Box<dyn MirStmtNode>) }
        })*
    };
}
impl_into_mir_stmt_box!(SMirAssignStmt, SMirFieldAssignStmt, SMirIndexAssignStmt, SMirReturnStmt, SMirIfStmt, SMirWhileStmt, SMirBreakStmt, SMirContinueStmt, SMirExprStmt, SMirBlockStmt, SMirDropStmt, SMirRetainStmt, SMirReleaseStmt);

#[derive(Debug, Clone)]
pub struct MirLocal {
    pub name: Symbol,
    pub ty: HirType,
    pub mutable: bool,
}

impl MirLocal {
    pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self {
        Self { name, ty, mutable }
    }
}

#[derive(Debug, Clone)]
pub struct MirFn {
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
    pub body: Vec<MirStmtBox>,
}

#[derive(Debug, Clone)]
pub enum MirItem {
    Fn(MirFn),
    StructDef {
        name: Symbol,
        fields: Vec<(Symbol, HirType)>,
    },
    Namespace {
        name: Symbol,
        items: Vec<MirItem>,
    },
}

#[derive(Debug, Clone)]
pub struct MirProgram {
    pub items: Vec<MirItem>,
    pub vtables: Vec<crate::hir::ir::VtableEntry>,
    pub struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub imported_fns: Vec<crate::hir::ir::ImportedFnSig>,
}
