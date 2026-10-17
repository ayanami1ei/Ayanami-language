/// Lexer: tokenizes Ayanami source text into a stream of tokens.
///
/// The lexer recognizes:
/// - Identifiers and keywords (fn, for, if, while, interface, impl, etc.)
/// - Literals: integers, floats, chars, strings, booleans
/// - Delimiters: parens, braces, brackets, comma, semicolon, arrow, dot
/// - Operators: +, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, !
pub mod delimiter;
pub mod keyword;
pub mod lexer;
pub mod token;
pub mod token_kind;
pub mod test;

pub use delimiter::Delimiter;
pub use keyword::Keyword;
pub use lexer::Lexer;
pub use token::Token;
pub use token_kind::TokenKind;

/// 源码注释（#179：格式化保留）。`span` 记录起止行列（行首/行尾判定用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub text: String,
    pub span: crate::span::Span,
}
