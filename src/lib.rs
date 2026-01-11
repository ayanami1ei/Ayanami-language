use crate::tokenlizer::Tokenlizer;
use crate::types::Token;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod error_type;
pub mod tokenlizer;
pub mod types;

#[test]
fn test() {
    let path = "test.aya";

    let input = File::open(path).unwrap();
    let buffered = BufReader::new(input);

    let mut tokens = Vec::<Vec<Token>>::new();

    for line in buffered.lines() {
        let s = line.unwrap();
        let mut tl = Tokenlizer::new(s.to_string());
        let token = match tl.tokenlize() {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        tokens.push(token);
    }

    for i in 0..tokens.len() {
        for j in 0..tokens[i].len() {
            println!("{}", tokens[i][j]);
        }
    }
}

pub fn tokenlize() {
    let s = "s = 1+\"str\"";
    let mut tl = Tokenlizer::new(s.to_string());
    let tokens = match tl.tokenlize() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    for i in 0..tokens.len() {
        println!("{}", tokens[i]);
    }
}
