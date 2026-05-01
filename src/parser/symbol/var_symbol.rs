use crate::intern::Symbol;
use crate::parser::ast::Type;

#[derive(Clone)]
pub struct VarSymbol {
    pub name: Symbol,
    pub type_: Type,
}
