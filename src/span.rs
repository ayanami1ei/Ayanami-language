/// Source location tracking (line, column, byte offsets).
///
/// Used by the lexer and parser for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Span {
    pub fn new(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            start_byte,
            end_byte,
        }
    }

    pub fn from_bytes(start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
            start_byte,
            end_byte,
        }
    }

    pub fn merge(self, other: Span) -> Self {
        Self {
            start_line: self.start_line,
            start_col: self.start_col,
            end_line: other.end_line,
            end_col: other.end_col,
            start_byte: self.start_byte,
            end_byte: other.end_byte,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0, 0)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}..{}:{}",
            self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}


