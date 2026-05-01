use std::fmt;

use crate::lexer::delimiter::Delimiter;
use crate::lexer::keyword::Keyword;

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
