use crate::hir::HirGenerator;
use crate::lir::LirGenerator;
use crate::pakager::{Arch, Pakage, System, Target, UseMode};
use crate::parser::Parser;
use crate::symbol_table::SymbolTable;
use crate::tokenlizer::Tokenlizer;
use crate::type_inferrer::TypeInferrer;
use crate::types::{Stmt, Token};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

pub mod compile;
pub mod error_type;
pub mod hir;
pub mod lir;
pub mod pakager;
pub mod parser;
pub mod symbol_table;
pub mod tokenlizer;
pub mod type_inferrer;
pub mod types;

pub fn main() {
    let path = "./test.aya";

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

    let mut type_inferrer = TypeInferrer::new(stmts, symbol_table.clone());
    stmts = type_inferrer.semantic_analysise().unwrap();

    let mut hir_generator = HirGenerator::new(stmts, symbol_table);
    let hirs = hir_generator.gen_hir();

    let hir_path = std::path::Path::new("./build/hir.txt");
    if let Some(parent) = hir_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = File::create(hir_path);
    fs::write(hir_path, "").unwrap();
    let mut hir_file = OpenOptions::new().append(true).open(hir_path).unwrap();
    for i in hirs.clone() {
        hir_file.write(&i.to_string().as_bytes()).unwrap();
        hir_file.write("\n".as_bytes()).unwrap();
    }

    // create an LLVM context and pass it to LIR generator
    let context = inkwell::context::Context::create();
    let mut lir_generator = LirGenerator::new(&context, &hirs, hir_generator.func_registry);
    lir_generator.gen_lir();
    lir_generator.to_llvm_bc("./build/ayanami_test.bc");

    let pakager = Pakage::new(
        "./build/ayanami_test.bc".to_string(),
        "0.0.0".to_string(),
        UseMode::AsDeveloper,
        Target::Executable,
        Arch::X86X64,
        System::Default,
        "./build/ayanami_test.bc".to_string(),
    );

    pakager.gen_pak("./build/ayanami_test.lcl");

    compile::compile("./build/ayanami_test.lcl", "./bin");
}
