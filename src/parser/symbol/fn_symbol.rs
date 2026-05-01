use crate::intern::Symbol;
use crate::parser::ast::Type;
use crate::parser::symbol::var_symbol::VarSymbol;

#[derive(Clone)]
pub struct FnSymbol {
    pub name: Symbol,
    pub rtn_type: Type,
    pub args: Vec<VarSymbol>,
}
