use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, UnaryOp};

/// Index into the current function's locals array.
/// Used in instructions to reference stack-allocated variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

/// Index into the global function table (Ctx::fns).
/// Used to statically resolve function calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnId(pub usize);

/// HIR type system.
///
/// Represents both concrete types (int, float, char, void, bool)
/// and type constructors (Unique/Shared/Weak pointers, Named symbols,
/// FatPtr for interface dispatch).
/// Named corresponds to user-defined types (structs).
/// FatPtr { data_ptr, vtable_ptr } is used for interface dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum HirType {
    Int,
    Float,
    Char,
    Void,
    Bool,
    Unique(Box<HirType>),
    Shared(Box<HirType>),
    Weak(Box<HirType>),
    Named(Symbol),
    /// Fat pointer `{ data_ptr, vtable_ptr }` for interface dispatch.
    /// `name` is the interface name, `kind` preserves ownership (Shared/Unique).
    FatPtr { name: Symbol, kind: Box<HirType> },
    /// Array of elements of the inner type
    Array(Box<HirType>),
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
}

/// HIR expressions.
///
/// After lowering from AST, all names are resolved to IDs (VarId, FnId).
/// Method calls are desugared:
/// - `obj.method(args)` → `Call(method, [obj, args...])` for static dispatch
/// - `iface.method(args)` → `VirtualCall { receiver, interface, .. }` for dynamic dispatch
/// - Interface parameter passing → `MakeFatPtr` wrapping concrete → fat pointer
#[derive(Debug, Clone)]
pub enum HirExpr {
    /// A literal constant with its resolved type.
    Literal(HirLiteral, HirType),
    /// Reference to a local variable.
    Local(VarId, HirType),
    Binary {
        op: BinaryOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
        ty: HirType,
    },
    Unary {
        op: UnaryOp,
        arg: Box<HirExpr>,
        ty: HirType,
    },
    Call {
        fn_id: FnId,
        args: Vec<HirExpr>,
        ty: HirType,
    },
    Move(Box<HirExpr>, HirType),
    Clone(Box<HirExpr>, HirType),
    /// Convert to unique ownership (deep copy)
    ToUnique(Box<HirExpr>, HirType),
    /// Convert to shared ownership (deep copy)
    ToShared(Box<HirExpr>, HirType),
    /// Convert to weak ownership (deep copy)
    ToWeak(Box<HirExpr>, HirType),
    /// Access a field of a struct value
    FieldAccess {
        object: Box<HirExpr>,
        field: Symbol,
        field_index: usize,
        ty: HirType,
    },
    /// Construct a struct value from field expressions
    StructLiteral {
        type_name: Symbol,
        fields: Vec<(Symbol, HirExpr)>,
        ty: HirType,
    },
    /// Array literal [a, b, c]
    ArrayLiteral(Vec<HirExpr>, HirType),
    /// Index expression arr[i]
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
        ty: HirType,
    },
    /// Virtual dispatch through interface fat pointer
    VirtualCall {
        receiver: Box<HirExpr>,
        interface: Symbol,
        method_index: usize,
        args: Vec<HirExpr>,
        ty: HirType,
    },
    /// Wrap a concrete value into a {data, vtable} fat pointer for interface dispatch
    MakeFatPtr {
        value: Box<HirExpr>,
        concrete_type: Symbol,
        interface_name: Symbol,
        ty: HirType,
    },
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
}

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

impl HirBlock {
    pub fn new(stmts: Vec<HirStmt>) -> Self {
        Self { stmts }
    }
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Assign {
        target: HirExpr,
        value: HirExpr,
    },
    Return {
        value: Option<HirExpr>,
    },
    If {
        cond: HirExpr,
        then_block: HirBlock,
        elifs: Vec<(HirExpr, HirBlock)>,
        else_block: Option<HirBlock>,
    },
    While {
        cond: HirExpr,
        body: HirBlock,
    },
    Expr(HirExpr),
    Block(Vec<HirStmt>),
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
    pub fn_id: FnId,
    pub name: Symbol,
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
    pub imported_fns: Vec<ImportedFnSig>,
}
