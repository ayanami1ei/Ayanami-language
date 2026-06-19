use std::collections::HashMap;
use crate::intern::Symbol;
use crate::span::Span;
use crate::hir::*;

#[derive(Debug, Clone)]
pub struct HirStructField {
    pub name: Symbol,
    pub ty: HirType,
}

#[derive(Debug, Clone)]
pub struct HirStructDef {
    pub name: Symbol,
    pub fields: Vec<HirStructField>,
}

#[derive(Debug, Clone)]
pub struct HirLocal {
    pub name: Symbol,
    pub ty: HirType,
    pub mutable: bool,
}

impl HirLocal {
    pub fn new(name: Symbol, ty: HirType, mutable: bool) -> Self {
        Self { name, ty, mutable }
    }
}

/// Vtable entry: maps a (concrete_type, interface) pair to the function IDs
/// implementing each interface method (index 0 = destructor/drop, 1..N = methods).
#[derive(Debug, Clone)]
pub struct VtableEntry {
    pub concrete_type: Symbol,
    pub interface: Symbol,
    pub method_fn_ids: Vec<FnId>,
}

/// A method signature inside an interface definition
#[derive(Debug, Clone)]
pub struct HirInterfaceMethod {
    pub name: Symbol,
    pub self_keyword: Symbol, // "shared" or "unique"
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
}

#[derive(Debug, Clone)]
pub struct HirFn {
    pub span: Span,
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<HirLocal>,
    pub body: HirBlock,
}

#[derive(Debug, Clone)]
pub enum HirItem {
    Fn(HirFn),
    StructDef(HirStructDef),
    Namespace {
        name: Symbol,
        items: Vec<HirItem>,
    },
    InterfaceDef {
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        methods: Vec<HirInterfaceMethod>,
    },
}

/// Signature of an imported function (from a package).
#[derive(Debug, Clone)]
pub struct ImportedFnSig {
    pub fn_id: FnId,
    pub name: Symbol,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
}

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub vtables: Vec<VtableEntry>,
    pub struct_defs: HashMap<Symbol, Vec<HirStructField>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub imported_fns: Vec<ImportedFnSig>,
}
