use crate::{symbol_table::SymbolTable, types::Token};
pub(crate) mod implement;

pub(crate) struct Parser{
    tokens:Vec<Vec<Token>>,
    symbol_table:SymbolTable,
    i:usize,
    j:usize
}