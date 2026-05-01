use crate::intern::Symbol;
use crate::parser::ast::Type;
use crate::parser::symbol::fn_symbol::FnSymbol;
use crate::parser::symbol::var_symbol::VarSymbol;

#[derive(Clone)]
pub enum SemanticSymbol {
    Fn(FnSymbol),
    Var(VarSymbol),
}

impl Default for SemanticSymbol {
    fn default() -> Self {
        SemanticSymbol::Var(VarSymbol {
            name: Symbol::default(),
            type_: Type::Void(Default::default()),
        })
    }
}

impl SemanticSymbol {
    pub fn new_var(name: Symbol) -> Self {
        SemanticSymbol::Var(VarSymbol {
            name,
            type_: Type::Void(Default::default()),
        })
    }

    pub fn new_fn(name: Symbol, rtn_type: Type) -> Self {
        SemanticSymbol::Fn(FnSymbol {
            name,
            rtn_type,
            args: Vec::new(),
        })
    }

    pub fn set_type(&mut self, type_: Type) {
        match self {
            SemanticSymbol::Var(v) => v.type_ = type_,
            SemanticSymbol::Fn(f) => f.rtn_type = type_,
        }
    }

    pub fn set_args(&mut self, args: Vec<VarSymbol>) {
        if let SemanticSymbol::Fn(f) = self {
            f.args = args;
        }
    }
}
