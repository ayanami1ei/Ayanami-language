use crate::{symbol_table::SymbolTable, types::Stmt};

pub(crate) mod implement;

pub(crate) struct SemanticAnalysiser{
    stmts:Vec<Stmt>,
    symbol_table:SymbolTable,

    dummy:Stmt
}