use crate::parser::symbol::{symbol::Symbol, tree::Tree};

pub struct Parser<'a>{
    map:Tree<Vec<Symbol<'a>>>
}

impl<'a> Parser<'a>{
    pub fn new()->Self{
        Self{map:Tree::new()}
    }
}