use std::fmt;

use crate::lexer::token_kind::TokenKind;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        line: usize,
        col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Token {
            kind,
            line,
            col,
            start_byte,
            end_byte,
        }
    }

    pub fn span(&self) -> Span {
        Span::new(
            self.line,
            self.col,
            self.line,
            self.col,
            self.start_byte,
            self.end_byte,
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}:{}", self.kind, self.line, self.col)
    }
}
