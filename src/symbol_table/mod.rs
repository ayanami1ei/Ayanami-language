use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::types::VarType;
pub(crate) mod implement;

#[derive(Debug, Clone)]
pub(crate) struct Symbol{
    name:String,

    pub(crate) is_func:bool,
    pub(crate) args:Vec<Symbol>,

    pub(crate) is_argc:bool,
    pub(crate) is_ref:bool,

    pub(crate) is_var:bool,
    pub(crate) its_type:VarType,

    area:Weak<RefCell<Scope>>
}

#[derive(Debug,Clone)]
pub(crate) struct Scope{
    parent:Option<Weak<RefCell<Scope>>>,
    sons:Vec<Rc<RefCell<Scope>>>,

    symbol:Vec<Symbol>
}

#[derive(Debug, Clone)]
pub(crate) struct SymbolTable{
    area:Rc<RefCell<Scope>>,
    pub(crate)area_ptr:Weak<RefCell<Scope>>
}