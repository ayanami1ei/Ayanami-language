use crate::{symbol_table::SymbolTable, types::Token};
pub mod implement;

pub struct Parser{
    tokens:Vec<Vec<Token>>,
    symbol_table:SymbolTable,
    i:usize,
    j:usize
}