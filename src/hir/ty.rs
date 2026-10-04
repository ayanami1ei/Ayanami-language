use crate::intern::Symbol;

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
    /// M1.4：32 位浮点（`f32`）；`float` / `f64` 为 `Float`
    F32,
    Char,
    Void,
    Bool,
    Unique(Box<HirType>),
    Named(Symbol),
    /// Fat pointer `{ data_ptr, vtable_ptr }` for interface dispatch.
    /// `name` is the interface name, `kind` preserves ownership (Shared/Unique).
    FatPtr { name: Symbol, kind: Box<HirType> },
    /// Array of elements of the inner type (unsized, heap)
    Array(Box<HirType>),
    /// Sized array of elements of the inner type (stack-allocated if value, heap if shared/unique)
    ArraySized(Box<HirType>, usize),
    /// Reference: Ref(inner, mutable)
    Ref(Box<HirType>, bool),
    FnPtr(Vec<HirType>, Box<HirType>),
    /// M1：定宽整数（i8..i128 / u8..u128 / isize / usize）
    IntN { bits: u8, signed: bool },
}

impl HirType {
    /// Copy 类型：赋值/传参时不移动（仅基元、函数指针与借用）。
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            HirType::Int
                | HirType::Float
                | HirType::F32
                | HirType::Char
                | HirType::Bool
                | HirType::Void
                | HirType::FnPtr(..)
                | HirType::Ref(..)
                | HirType::IntN { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
}
