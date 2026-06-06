/// Parser: recursive-descent parser from token stream to AST.
///
/// Parses function declarations, namespaces, interface/impl blocks,
/// expressions, statements, and control flow. Uses a `Parser` struct
/// with `parse_*` methods for each grammar rule.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod ast;

pub use parser::Parser;
