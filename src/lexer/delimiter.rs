use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
        };
        write!(f, "{}", s)
    }
}
