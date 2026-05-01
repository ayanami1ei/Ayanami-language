use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Return,
    For,
    If,
    While,
    Int,
    Float,
    Char,
    Mut,
    Shared,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Keyword::Fn => "fn",
            Keyword::Return => "return",
            Keyword::For => "for",
            Keyword::If => "if",
            Keyword::While => "while",
            Keyword::Int => "int",
            Keyword::Float => "float",
            Keyword::Char => "char",
            Keyword::Mut => "mut",
            Keyword::Shared => "shared",
        };
        write!(f, "{}", s)
    }
}
