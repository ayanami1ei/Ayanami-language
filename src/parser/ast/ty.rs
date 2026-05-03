use crate::intern::Symbol;
use crate::span::Span;

#[derive(Debug, Clone, Default)]
pub enum Type {
    #[default]
    Default,
    Int(Span),
    Float(Span),
    Char(Span),
    Void(Span),
    Named(Symbol, Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s) | Type::Float(s) | Type::Char(s) | Type::Void(s) | Type::Named(_, s) => *s,
            Type::Default => todo!(),
        }
    }
}
