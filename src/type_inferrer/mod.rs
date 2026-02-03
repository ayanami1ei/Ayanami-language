use std::{cell::RefCell, rc::Rc};

use crate::{symbol_table::SymbolTable, types::Stmt};

pub mod implement;

pub struct TypeInferrer {
    stmts: Vec<Stmt>,
    symbol_table: std::rc::Rc<std::cell::RefCell<SymbolTable>>,

    dummy: Rc<RefCell<Stmt>>,
}
