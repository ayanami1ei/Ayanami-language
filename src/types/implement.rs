use core::fmt;

use crate::types::Token;

impl fmt::Display for Token{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Token::Identifier(s)=self{
            write!(f,"identifier: {}", s)
        } else if let Token::Operator(s)=self{
            write!(f,"operator: {}", s)
        } else if let Token::Keyword(s)=self{
            write!(f,"keyword: {}", s)
        } else if let Token::Num(s)=self{
            write!(f,"const number: {}", s)
        } else {
            write!(f,"print error")
        }
    }
}