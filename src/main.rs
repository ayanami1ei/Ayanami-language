use crate::lexer::{Lexer, TokenKind};

pub mod intern;
pub mod lexer;
pub mod parser;
pub mod span;

pub fn main() {
    let code = include_str!("../test.aya");
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize_all();
    for t in tokens.iter().filter(|t| !matches!(t.kind, TokenKind::EOF)) {
        println!("{}", t);
    }
}

#[cfg(test)]
mod test {
    
    #[test]
    fn test() {
        
    }
}
