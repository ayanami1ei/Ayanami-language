use crate::intern::Symbol;
use crate::span::Span;

#[derive(Debug, Clone, Default)]
pub enum Type {
    #[default]
    Default,
    Int(Span),
    Float(Span),
    Char(Span),
    Bool(Span),
    Void(Span),
    /// M1.9：never 类型 `!`（发散表达式）
    Never(Span),
    Named(Symbol, Span),
    Generic(Symbol, Vec<Type>, Span),  // Foo[int]
    Array(Box<Type>, Span),
    /// M6.2c：`[T; N]`（N 为字面量整数）
    ArraySized(Box<Type>, usize, Span),
    Ref(Box<Type>, bool, Span),  // ref T or ref mut T
    Unique(Box<Type>, Span),
    Self_(Span),
    FnPtr(Vec<Type>, Box<Type>, Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s)
            | Type::Float(s)
            | Type::Char(s)
            | Type::Bool(s)
            | Type::Void(s)
            | Type::Never(s)
            | Type::Generic(_, _, s)
            | Type::Array(_, s)
            | Type::ArraySized(_, _, s)
            | Type::Ref(_, _, s)
            | Type::Unique(_, s)
            | Type::FnPtr(_, _, s) => *s,
            Type::Named(_, s) => *s,
            Type::Self_(s) => *s,
            Type::Default => todo!(),
        }
    }

    pub fn inner(&self) -> Option<&Type> {
        match self {
            Type::Unique(ty, _) => Some(ty),
            _ => None,
        }
    }
}
