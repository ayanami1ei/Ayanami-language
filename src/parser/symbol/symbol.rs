use crate::parser::symbol::types::Types;

#[derive(Clone)]
pub struct VarSymbol<'a> {
    name: &'a str,
    type_: Types,
}

#[derive(Clone)]
pub struct FnSymbol<'a> {
    name: &'a str,
    rtn_type: Types,
    args: Vec<VarSymbol<'a>>,
}

#[derive(Clone)]
pub enum Symbol<'a> {
    Fn(FnSymbol<'a>),
    Var(VarSymbol<'a>),
    Default,
}

impl<'a> Default for Symbol<'a> {
    fn default() -> Self {
        Symbol::Default
    }
}

impl<'a> Symbol<'a> {
    pub fn new_var(name: &'a str) -> Self {
        Symbol::Var(VarSymbol { name, type_: Types::Unknown })
    }

    pub fn new_fn(name: &'a str, rtn_type: Types) -> Self {
        Symbol::Fn(FnSymbol { name, rtn_type, args: Vec::new() })
    }

    pub fn set_type(&mut self, type_: &Types) {
        match self {
            Symbol::Var(v) => v.type_ = type_.clone(),
            Symbol::Fn(f) => f.rtn_type = type_.clone(),
            _ => {}
        }
    }

    pub fn set_args(&mut self, args: Vec<VarSymbol<'a>>) {
        if let Symbol::Fn(f) = self {
            f.args = args;
        }
    }
}