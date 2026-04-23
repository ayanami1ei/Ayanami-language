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

pub fn tokenlize(path: &str) -> Result<Vec<Vec<Token>>, error_type::Error> {
    let input = File::open(path).unwrap();
    let buffered = BufReader::new(input);

    let mut tokens = Vec::<Vec<Token>>::new();

    for line in buffered.lines() {
        let s = line.unwrap();
        let mut tl = Tokenlizer::new(s.to_string());
        let token = match tl.tokenlize() {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };
        if token.len() != 0 {
            tokens.push(token);
        }
    }
    return Ok(tokens);
}

pub fn parse(tokens: Vec<Vec<Token>>) -> Result<(Vec<Stmt>, SymbolTable), error_type::Error> {
    let symbol_table = SymbolTable::new();

    let mut parser = Parser::new(tokens, symbol_table.clone());
    let mut stmts = match parser.parser() {
        Ok(c) => c,
        Err(e) => {
            return Err(e);
        }
    };

    Ok((stmts, symbol_table))
}
pub fn type_infer(
    mut stmts: Vec<Stmt>,
    symbol_table: SymbolTable,
) -> Result<Vec<Stmt>, error_type::Error> {
    let mut type_inferrer = TypeInferrer::new(stmts, symbol_table.clone());
    stmts = match type_inferrer.semantic_analysise() {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    Ok(stmts)
}
pub fn pakage(
    stmts: Vec<Stmt>,
    symbol_table: SymbolTable,
    out_dir: &str,
    name: String,
    version: String,
    use_mode: UseMode,
    target: Target,
    arch: Arch,
    system: System,
) {
    let out_dir = std::path::Path::new(out_dir);
    let _ = fs::create_dir_all(out_dir);
    let mut hir_generator = HirGenerator::new(stmts, symbol_table.clone());
    let hirs = hir_generator.gen_hir();

    #[cfg(debug_assertions)]
    {
        let hir_path = out_dir.join("hir.txt");
        if let Some(parent) = hir_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = File::create(hir_path.clone());
        fs::write(&hir_path, "").unwrap();
        let mut hir_file = OpenOptions::new().append(true).open(&hir_path).unwrap();
        for i in hirs.clone() {
            hir_file.write(&i.to_string().as_bytes()).unwrap();
            hir_file.write("\n".as_bytes()).unwrap();
        }
    }

    // create an LLVM context and pass it to LIR generator
    let context = inkwell::context::Context::create();
    let mut lir_generator = LirGenerator::new(&context, &hirs, hir_generator.func_registry.clone());
    lir_generator.gen_lir();
    let base_name = std::path::Path::new(&name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("ayanami")
        .to_string();
    let bc_path = out_dir.join(format!("{}.bc", base_name));
    lir_generator.to_llvm_bc(bc_path.to_string_lossy().as_ref());

    let pakager = Pakage::new(
        name,
        version,
        use_mode,
        target,
        arch,
        system,
        symbol_table.export_root_symbols().clone(),
        bc_path.to_string_lossy().to_string(),
    );

    let pak_path = out_dir.join(format!("{}.lcl", base_name));
    pakager.gen_pak(pak_path.to_string_lossy().as_ref());
}

pub fn compile(pak_path: &str, output_path: &str) {
    compile::compile(pak_path, output_path);
}