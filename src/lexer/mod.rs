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
