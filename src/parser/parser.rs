use crate::parser::symbol::{symbol::SemanticSymbol, tree::Tree};

#[allow(dead_code)]
pub struct Parser {
    map: Tree<Vec<SemanticSymbol>>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            map: Tree::new(),
        }
    }
}