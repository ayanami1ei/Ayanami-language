use core::fmt;
use std::collections::HashSet;

use crate::types::{Argc, Token, VarType};

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Token::Identifier(s) = self {
            write!(f, "identifier: {}", s)
        } else if let Token::Operator(s) = self {
            write!(f, "operator: {}", s)
        } else if let Token::Keyword(s) = self {
            write!(f, "keyword: {}", s)
        } else if let Token::Num(s) = self {
            write!(f, "const number: {}", s)
        } else {
            write!(f, "print error")
        }
    }
}

impl Argc {
    pub(crate) fn new() -> Argc {
        Argc {
            is_ref: false,
            arg_type: VarType::Unknown,
            var_name: String::new(),
        }
    }
}

impl fmt::Display for VarType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            VarType::Int=>write!(f,"int"),
            VarType::Float=>write!(f,"float"),
            VarType::Char=>write!(f,"char"),
            VarType::Bool=>write!(f,"bool"),
            VarType::Unknown=>write!(f,"unknown"),
        }
    }
}

