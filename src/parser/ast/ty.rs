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
    Named(Symbol, Span),
    Array(Box<Type>, Span),
    Ref(Box<Type>, bool, Span),  // ref T or ref mut T
    Unique(Box<Type>, Span),
    Shared(Box<Type>, Span),
    Weak(Box<Type>, Span),
    Self_(Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s)
            | Type::Float(s)
            | Type::Char(s)
            | Type::Bool(s)
            | Type::Void(s)
            | Type::Array(_, s)
            | Type::Ref(_, _, s)
            | Type::Unique(_, s)
            | Type::Shared(_, s)
            | Type::Weak(_, s) => *s,
            Type::Named(_, s) => *s,
            Type::Self_(s) => *s,
            Type::Default => todo!(),
        }
    }

    pub fn inner(&self) -> Option<&Type> {
        match self {
            Type::Unique(ty, _) | Type::Shared(ty, _) | Type::Weak(ty, _) => Some(ty),
            _ => None,
        }
    }
}
