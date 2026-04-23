use crate::tokenlizer::{Lexer, TokenKind};

pub mod tokenlizer;

pub fn main(){
    let code=include_str!("../test.aya");
    let mut tokenlizer=Lexer::new(code);
    let mut token = tokenlizer.next_token();
    loop {
        if matches!(&token.kind, TokenKind::EOF) {
            break;
        }
        println!("{}", token);
        token = tokenlizer.next_token();
    }
}