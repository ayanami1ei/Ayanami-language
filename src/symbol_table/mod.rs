use std::{
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use crate::types::VarType;
pub(crate) mod implement;

#[derive(Debug, Clone)]
pub(crate) struct Symbol {
    pub(crate) name: String,

    pub(crate) is_func: bool,
    pub(crate) args: Vec<Symbol>,

    pub(crate) is_argc: bool,
    #[allow(unused)]
    pub(crate) is_ref: bool,

    #[allow(unused)]
    pub(crate) is_var: bool,
    pub(crate) its_type: HashSet<VarType>,

    pub(crate) is_arr:bool,
    pub(crate) elem_type:Vec<HashSet<VarType>>,

    pub(crate) id: i32,
    pub(crate) level: i32,

    // the id of the Scope this symbol was declared in
    #[allow(unused)]
    pub(crate) scope_id: i32,

    pub(crate) area: Weak<RefCell<Scope>>,
    pub(crate) body_scope_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub(crate) struct Scope {
    parent: Option<Weak<RefCell<Scope>>>,
    sons: Vec<Rc<RefCell<Scope>>>,

    pub(crate) id: i32,

    symbol: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub(crate) struct SymbolTable {
    area: Rc<RefCell<Scope>>,
    pub(crate) area_ptr: Weak<RefCell<Scope>>,
    next_scope_id: i32,
    pub(crate) now_level: i32,
}
