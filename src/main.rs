use crate::lexer::{Lexer, TokenKind};

pub mod lexer;
pub mod parser;

pub fn main() {
    let code = include_str!("../test.aya");
    let mut lexer = Lexer::new(code);
    let mut token = lexer.next_token();
    loop {
        if matches!(&token.kind, TokenKind::EOF) {
            break;
        }
        println!("{}", token);
        token = lexer.next_token();
    }
}

#[cfg(test)]
mod test {
    
    #[test]
    fn test() {
        
    }
}
