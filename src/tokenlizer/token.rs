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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Keyword(Keyword),
    IntLiteral(String),
    FloatLiteral(String),
    CharLiteral(String),
    StringLiteral(String),
    Symbol(String),
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
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
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "Identifier({})", s),
            TokenKind::Keyword(k) => write!(f, "Keyword({})", k),
            TokenKind::IntLiteral(s) => write!(f, "IntLiteral({})", s),
            TokenKind::FloatLiteral(s) => write!(f, "FloatLiteral({})", s),
            TokenKind::CharLiteral(s) => write!(f, "CharLiteral({})", s),
            TokenKind::StringLiteral(s) => write!(f, "StringLiteral(\"{}\")", s),
            TokenKind::Symbol(s) => write!(f, "Symbol({})", s),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}:{}", self.kind, self.line, self.col)
    }
}
