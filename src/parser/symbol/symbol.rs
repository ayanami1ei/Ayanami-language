use crate::parser::symbol::{symbol_type::SymBolType, types::Types};

#[derive(Default, Clone)]
pub struct Symbol<'a>{
    name:&'a str,
    type_:Types,
    symbol_type:SymBolType<'a>,
}   

impl<'a> Symbol<'a>{
    pub fn new_var(name:&'a str)->Self{
        Self { name, type_:Types::Unknown, symbol_type: SymBolType::Var }
    }

    pub fn new_fn(name:&'a str, rtn_type:Types)->Self{
        Self { name, type_:rtn_type, symbol_type: SymBolType::Fn(Vec::new()) }
    }

    pub fn set_type(&mut self, type_:&Types){
        self.type_=type_.clone();
    }

    pub fn set_args(&mut self, args:Vec<Symbol<'a>>){
        self.symbol_type=SymBolType::Fn(args);
    }
}