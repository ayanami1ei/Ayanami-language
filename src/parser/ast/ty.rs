use crate::intern::Symbol;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Type {
    Int(Span),
    Float(Span),
    Char(Span),
    Void(Span),
    Named(Symbol, Span),
    Ref(Box<Type>, Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s)
            | Type::Float(s)
            | Type::Char(s)
            | Type::Void(s)
            | Type::Named(_, s)
            | Type::Ref(_, s) => *s,
        }
    }
}
