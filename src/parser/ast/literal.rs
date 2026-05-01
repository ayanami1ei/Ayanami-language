use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64, Span),
    Float(f64, Span),
    Char(char, Span),
    String(String, Span),
    Bool(bool, Span),
}

impl Literal {
    pub fn span(&self) -> Span {
        match self {
            Literal::Int(_, s)
            | Literal::Float(_, s)
            | Literal::Char(_, s)
            | Literal::String(_, s)
            | Literal::Bool(_, s) => *s,
        }
    }
}
