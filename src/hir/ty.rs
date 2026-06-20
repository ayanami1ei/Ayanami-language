use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::span::Span;

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
    /// Array of elements of the inner type (unsized, heap)
    Array(Box<HirType>),
    /// Sized array of elements of the inner type (stack-allocated if value, heap if shared/unique)
    ArraySized(Box<HirType>, usize),
    /// Reference: Ref(inner, mutable)
    Ref(Box<HirType>, bool),
    FnPtr(Vec<HirType>, Box<HirType>),
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
}
