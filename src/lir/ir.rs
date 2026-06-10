use std::collections::{HashMap, HashSet};

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirLocal;
use crate::parser::ast::{BinaryOp, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvKind {
    ToUnique,
    ToShared,
    ToWeak,
}

#[derive(Debug, Clone)]
pub enum LirValue {
    Var(VarId),
    Tmp(u64),
    Param(u64),
    Literal(HirLiteral, HirType),
}

#[derive(Debug, Clone)]
pub enum LirInst {
    Alloca(VarId, HirType),
    Store {
        dest: VarId,
        src: LirValue,
        ty: HirType,
    },
    Load {
        dest: u64,
        src: VarId,
        ty: HirType,
    },
    BinOp {
        dest: u64,
        op: BinaryOp,
        lhs: LirValue,
        rhs: LirValue,
        /// LLVM operand type (e.g., Int → i64 for icmp/add)
        ty: HirType,
        /// Result type (for comparisons this is Bool, otherwise same as ty)
        result_ty: HirType,
    },
    UnaryOp {
        dest: u64,
        op: UnaryOp,
        src: LirValue,
        ty: HirType,
    },
    Call {
        dest: Option<u64>,
        fn_id: FnId,
        args: Vec<(LirValue, HirType)>,
        ret_ty: HirType,
    },
    /// Call via function pointer
    CallPtr {
        dest: u64,
        fn_ptr: LirValue,
        args: Vec<(LirValue, HirType)>,
        ret_ty: HirType,
    },
    /// Get a function pointer value
    FnAddr { dest: u64, fn_id: FnId },
    StrGlobal { dest: u64, str_idx: u64 },
    /// Deep copy a value and change ownership: used by ToUnique/ToShared/ToWeak
    Conv {
        dest: u64,
        /// Pre-allocated temp for alloca (used when src is a struct value)
        alloca_tmp: u64,
        /// Pre-allocated temp for malloc result
        malloc_tmp: u64,
        src: LirValue,
        /// The conversion kind: unique/shared/weak
        kind: ConvKind,
        /// The source type before conversion
        src_ty: HirType,
        /// The target type after conversion
        ty: HirType,
    },
    DropValue(VarId, HirType),
    RetainValue(VarId, HirType),
    ReleaseValue(VarId, HirType),
    Br(String),
    BrCond {
        cond: LirValue,
        true_block: String,
        false_block: String,
    },
    Ret(Option<(LirValue, HirType)>),
    /// Make a fat pointer from a concrete value: heap-allocates, stores value, gets vtable
    MakeFatPtr {
        dest: u64,
        /// Pre-allocated temp for malloc result
        malloc_tmp: u64,
        /// Pre-allocated temp for bitcast result
        bc_tmp: u64,
        /// Pre-allocated temp for vtable GEP result
        vtable_gep_tmp: u64,
        /// Pre-allocated temp for first insertvalue result
        iv_tmp: u64,
        value_src: LirValue,
        value_ty: HirType,
        vtable_name: String,
        ty: HirType,  // FatPtr type
    },
    /// Extract a field from a struct value via extractvalue
    FieldAccess {
        dest: u64,
        /// Pre-allocated temp for GEP (used for heap-allocated structs)
        gep_tmp: u64,
        src: LirValue,
        field_index: usize,
        field_ty: HirType,
        struct_ty: HirType,
    },
    /// Inline assembly instruction
    Asm {
        dest: Option<u64>,
        template: String,
        output_constraints: Vec<String>,
        input_operands: Vec<(LirValue, HirType)>,
        input_constraints: Vec<String>,
        ret_ty: HirType,
    },
    /// Take a reference: emit the address of a variable as ptr
    RefInst {
        dest: u64,
        var_id: VarId,
        mutable: bool,
        ty: HirType,
    },
    /// Allocate a zeroed array on the heap with count elements
    ArraySized {
        dest: u64,
        /// Pre-allocated temp for malloc/calloc result
        malloc_tmp: u64,
        /// Pre-allocated temp for count evaluaton
        count_tmp: u64,
        /// Pre-allocated temp for size calculation
        size_tmp: u64,
        elem_count: LirValue,
        elem_size: u64,
        elem_ty: HirType,
        ty: HirType,
    },
    /// Allocate an array on the heap and store elements
    ArrayLit {
        dest: u64,
        /// Pre-allocated temp for malloc result
        malloc_tmp: u64,
        /// Pre-allocated temps for GEP to each element
        elem_geps: Vec<u64>,
        /// Element values and their types
        elems: Vec<(LirValue, HirType)>,
        /// The element type
        elem_ty: HirType,
        /// The array type
        ty: HirType,
    },
    /// Index into an array: arr[index]
    IndexAccess {
        dest: u64,
        /// Pre-allocated temp for GEP
        gep_tmp: u64,
        /// Pre-allocated temp for the loaded value
        load_tmp: u64,
        arr: LirValue,
        index: LirValue,
        elem_ty: HirType,
        ty: HirType,
    },
    /// Construct a struct value from field values via insertvalue
    StructLit {
        dest: u64,
        /// Pre-allocated temp for alloca
        alloca_tmp: u64,
        /// Pre-allocated temps for each field's GEP
        field_geps: Vec<u64>,
        fields: Vec<(LirValue, HirType)>,
        struct_name: Symbol,
        struct_ty: HirType,
    },
    /// Virtual dispatch through fat pointer
    VirtualCall {
        fn_dest: Option<u64>,
        receiver_tmp: u64,
        /// Pre-allocated temp for data extractvalue
        data_tmp: u64,
        /// Pre-allocated temp for vtable extractvalue
        vtable_tmp: u64,
        /// Pre-allocated temp for GEP result
        gep_tmp: u64,
        /// Pre-allocated temp for load vtable slot
        fn_ptr_tmp: u64,
        method_index: usize,
        args: Vec<(LirValue, HirType)>,
        ret_ty: HirType,
    },
    /// Store a value into a struct field
    FieldStore {
        /// Temp holding the object (value or pointer)
        dest: u64,
        /// Alloca variable (for value types — store modified struct back)
        var_id: Option<VarId>,
        /// Pre-allocated temp for GEP result
        gep_tmp: u64,
        /// Pre-allocated temp for insertvalue (value types only)
        iv_tmp: u64,
        /// Value to store
        src: LirValue,
        field_index: usize,
        field_ty: HirType,
        struct_ty: HirType,
    },
    /// Store a value into an array element
    IndexStore {
        /// Temp holding the array pointer
        dest: u64,
        /// Pre-allocated temp for GEP result
        gep_tmp: u64,
        /// Value to store
        src: LirValue,
        /// Index value
        index: LirValue,
        elem_ty: HirType,
        array_ty: HirType,
    },
}

#[derive(Debug, Clone)]
pub struct LirBlock {
    pub label: String,
    pub insts: Vec<LirInst>,
}

#[derive(Debug, Clone)]
pub struct LirFn {
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
    pub blocks: Vec<LirBlock>,
}

/// Describes a vtable global constant for interface dispatch
#[derive(Debug, Clone)]
pub struct VtableDesc {
    pub name: String,
    pub fn_ids: Vec<FnId>,
}

#[derive(Debug, Clone)]
pub struct LirProgram {
    pub strings: Vec<String>,
    pub fn_names: HashMap<FnId, String>,
    pub functions: Vec<LirFn>,
    pub vtables: Vec<VtableDesc>,
    pub struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub imported_fn_ids: HashSet<FnId>,
}
