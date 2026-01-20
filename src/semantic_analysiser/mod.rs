use std::{cell::RefCell, rc::Rc};

use crate::{symbol_table::SymbolTable, types::Stmt};

pub(crate) mod implement;

pub(crate) struct SemanticAnalysiser {
    stmts: Vec<Stmt>,
    symbol_table: SymbolTable,

    dummy: Rc<RefCell<Stmt>>,
}
