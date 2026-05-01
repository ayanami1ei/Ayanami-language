use std::fmt;

use crate::lexer::token_kind::TokenKind;

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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}:{}", self.kind, self.line, self.col)
    }
}
