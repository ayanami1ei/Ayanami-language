use crate::hir::ir::{FnId, VtableEntry, VarId};
use std::collections::HashMap;
pub use crate::hir::ir::{HirLiteral, HirType};
pub use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::intern::Symbol;

#[derive(Debug, Clone)]
pub enum MirExpr {
    Local(VarId, HirType, bool),
    Literal(HirLiteral, HirType),
    Binary {
        op: BinaryOp,
        lhs: Box<MirExpr>,
        rhs: Box<MirExpr>,
        ty: HirType,
    },
    Unary {
        op: UnaryOp,
        arg: Box<MirExpr>,
        ty: HirType,
    },
    Call {
        fn_id: FnId,
        args: Vec<MirExpr>,
        ty: HirType,
    },
    Move(Box<MirExpr>, HirType),
    Clone(Box<MirExpr>, HirType),
    ToUnique(Box<MirExpr>, HirType),
    ToShared(Box<MirExpr>, HirType),
    ToWeak(Box<MirExpr>, HirType),
    VirtualCall {
        receiver: Box<MirExpr>,
        interface: Symbol,
        method_index: usize,
        args: Vec<MirExpr>,
        ty: HirType,
    },
    MakeFatPtr {
        value: Box<MirExpr>,
        concrete_type: Symbol,
        interface_name: Symbol,
        ty: HirType,
    },
    FieldAccess {
        object: Box<MirExpr>,
        field: Symbol,
        field_index: usize,
        ty: HirType,
    },
    StructLiteral {
        type_name: Symbol,
        fields: Vec<(Symbol, MirExpr)>,
        ty: HirType,
    },
    ArrayLiteral(Vec<MirExpr>, HirType),
    ArraySized { count: Box<MirExpr>, elem_ty: HirType, ty: HirType },
    Index {
        object: Box<MirExpr>,
        index: Box<MirExpr>,
        ty: HirType,
    },
}

#[derive(Debug, Clone)]
pub enum MirStmt {
    Assign {
        target: MirExpr,
        value: MirExpr,
    },
    FieldAssign {
        object: Box<MirExpr>,
        field: Symbol,
        field_index: usize,
        field_ty: HirType,
        value: MirExpr,
    },
    IndexAssign {
        object: Box<MirExpr>,
        index: Box<MirExpr>,
        value: MirExpr,
    },
    Return {
        value: Option<MirExpr>,
    },
    If {
        cond: MirExpr,
        then_block: Vec<MirStmt>,
        elifs: Vec<(MirExpr, Vec<MirStmt>)>,
        else_block: Option<Vec<MirStmt>>,
    },
    While {
        cond: MirExpr,
        body: Vec<MirStmt>,
    },
    Expr(MirExpr),
    Block(Vec<MirStmt>),
    Drop(VarId, HirType),
    Retain(VarId, HirType),
    Release(VarId, HirType),
}

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
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
    pub body: Vec<MirStmt>,
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
    pub vtables: Vec<VtableEntry>,
    pub struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    pub imported_fns: Vec<crate::hir::ir::ImportedFnSig>,
}
