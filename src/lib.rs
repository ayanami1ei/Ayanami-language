use crate::hir::HIR;
use crate::parser::Parser;
use crate::semantic_analysiser::SemanticAnalysiser;
use crate::symbol_table::SymbolTable;
use crate::tokenlizer::Tokenlizer;
use crate::types::Token;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod error_type;
pub mod hir;
pub mod parser;
pub mod semantic_analysiser;
pub mod symbol_table;
pub mod tokenlizer;
pub mod types;

pub fn run() {
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
        if token.len() != 0 {
            tokens.push(token);
        }
    }

    let symbol_table = SymbolTable::new();

    let mut parser = Parser::new(tokens, symbol_table.clone());
    let mut stmts = match parser.parser() {
        Ok(c) => c,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let mut semantic_analysiser = SemanticAnalysiser::new(stmts, symbol_table.clone());
    stmts = semantic_analysiser.semantic_analysise().unwrap();

    let mut hir = HIR::new(stmts, symbol_table).unwrap();
    let ins = hir.gen_ir().unwrap();
    for i in ins {
        println!("{}", i);
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
