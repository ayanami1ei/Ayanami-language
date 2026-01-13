use crate::{types::Stmt};
pub(crate) mod implement;

pub(crate) struct HIR{
    stmts:Vec<Stmt>,
    i:usize
}

pub(crate) enum HisIsa {
    Assign(String,String),
    Label(String),
    Jmp(String)
}