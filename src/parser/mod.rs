/// Parser: recursive-descent parser from token stream to AST.
///
/// Parses function declarations, namespaces, interface/impl blocks,
/// expressions, statements, and control flow. Uses a `Parser` struct
/// with `parse_*` methods for each grammar rule.
#[allow(clippy::module_inception)]
pub mod parser;
pub mod ast;
pub mod gen_bridge;

pub use parser::Parser;

/// Parse source code using the generated parser (from Asuka grammar).
/// Falls back to the hand-written parser if the generated one fails.
pub fn parse_source(source: &str) -> Result<crate::parser::ast::program::Program, String> {
    // Use generated parser
    let tokens = crate::generated::tokenize(source);
    let mut p = crate::generated::Parser::new(tokens);
    match p.pprogram() {
        Ok(val) => {
            if let asuka::runtime::Value::Node(node) = val {
                let block = gen_bridge::node_to_program(&node)?;
                Ok(crate::parser::ast::program::Program::new(block.stmts))
            } else {
                fallback_parse(source)
            }
        }
        Err(_) => fallback_parse(source),
    }
}

fn fallback_parse(source: &str) -> Result<crate::parser::ast::program::Program, String> {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens: Vec<_> = lexer.tokenize_all().into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut hp = crate::parser::Parser::new(tokens);
    hp.parse_program()
}
