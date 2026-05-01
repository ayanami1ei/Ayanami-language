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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Keyword(Keyword),
    IntLiteral(String),
    FloatLiteral(String),
    CharLiteral(String),
    StringLiteral(String),
    Delimiter(Delimiter),
    Operator(String),
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

    pub fn span(&self) -> crate::span::Span {
        crate::span::Span::new(self.line, self.col, self.line, self.col)
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
            Keyword::Mut => "mut",
            Keyword::Shared => "shared",
        };
        write!(f, "{}", s)
    }
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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "Identifier({})", s),
            TokenKind::Keyword(k) => write!(f, "Keyword({})", k),
            TokenKind::IntLiteral(s) => write!(f, "IntLiteral({})", s),
            TokenKind::FloatLiteral(s) => write!(f, "FloatLiteral({})", s),
            TokenKind::CharLiteral(s) => write!(f, "CharLiteral({})", s),
            TokenKind::StringLiteral(s) => write!(f, "StringLiteral(\"{}\")", s),
            TokenKind::Delimiter(d) => write!(f, "Delimiter({})", d),
            TokenKind::Operator(s) => write!(f, "Operator({})", s),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}:{}", self.kind, self.line, self.col)
    }
}
