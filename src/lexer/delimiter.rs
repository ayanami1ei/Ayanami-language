use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delimiter {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Arrow,
    Dot,
    Colon,
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Delimiter::LParen => "(",
            Delimiter::RParen => ")",
            Delimiter::LBrace => "{",
            Delimiter::RBrace => "}",
            Delimiter::LBracket => "[",
            Delimiter::RBracket => "]",
            Delimiter::Comma => ",",
            Delimiter::Semicolon => ";",
            Delimiter::Arrow => "->",
            Delimiter::Dot => ".",
            Delimiter::Colon => ":",
        };
        write!(f, "{}", s)
    }
}
