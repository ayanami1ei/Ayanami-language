use crate::parser::symbol::symbol::Symbol;

#[derive(Default, Clone)]
pub enum SymBolType<'a>{
    Fn(Vec<Symbol<'a>>),
    Var,

    #[default]
    Default,
}