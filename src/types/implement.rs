use core::fmt;

use crate::types::{Argc, Expr, Token, VarType};
use std::collections::HashSet;

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

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarType::Int => write!(f, "int"),
            VarType::Float => write!(f, "float"),
            VarType::Char => write!(f, "char"),
            VarType::Bool => write!(f, "bool"),
            VarType::Unknown => write!(f, "unknown"),
        }
    }
}

impl Expr {
    pub fn get_type_set(&self) -> &HashSet<VarType> {
        match self {
            Expr::ConstNum(_, ty) => ty,
            Expr::ConstChar(_, ty) => ty,
            Expr::Var(_, ty) => ty,
            Expr::FuncCall(_, _, ty) => ty,
            Expr::Add(_, _, ty) => ty,
            Expr::Sub(_, _, ty) => ty,
            Expr::Mul(_, _, ty) => ty,
            Expr::Div(_, _, ty) => ty,
            Expr::Equal(_, _, ty) => ty,
            Expr::Greater(_, _, ty) => ty,
            Expr::Less(_, _, ty) => ty,
            Expr::GreaterEqual(_, _, ty) => ty,
            Expr::LessEqual(_, _, ty) => ty,
            Expr::Not(_, ty) => ty,
        }
    }
}
