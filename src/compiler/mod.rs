pub mod build;
pub mod check;
pub mod debug;
pub mod import;
pub mod symdef;

pub use build::{
    build_source, build_source_to, build_source_with_target, compile_file,
    install_package, package_source, run_executable,
};
pub use check::check_hir_returns;
pub use debug::format_program;
pub use import::{find_std_dir, resolve_import_path};
pub use symdef::{collect_defs_from_stmts, defs_to_json, SymDef};

use crate::lir::ir::LirProgram;
use crate::parser::ast::Program;

/// Result of compiling a single file (including its recursively-resolved imports).
pub struct CompiledFile {
    pub program: Program,
    pub lir_program: LirProgram,
    pub llvm_ir: String,
    pub obj_paths: Vec<std::path::PathBuf>,
    pub link_flags: Vec<String>,
    pub own_obj: std::path::PathBuf,
    pub lcl_path: std::path::PathBuf,
    pub dep_lcl_paths: Vec<std::path::PathBuf>,
}

/// Simple compile from source text (no import resolution).
pub struct CompileResult {
    pub llvm_ir: String,
    pub program: Program,
    pub lir_program: LirProgram,
}

pub fn compile_source(code: &str) -> Result<CompileResult, String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;
    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error: {}", e))?;
    let mir_program = crate::mir::lower_program(&hir_program);
    let lir_program = crate::lir::lower_program(&mir_program);
    let llvm_ir = crate::lir::emit_program(&lir_program);
    Ok(CompileResult {
        llvm_ir,
        program,
        lir_program,
    })
}

pub fn check_source(code: &str, _out_dir: &str) -> Result<(), String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;
    let hir_program =
        crate::hir::lower_program(&program).map_err(|e| format!("HIR error: {}", e))?;
    let mir_program = crate::mir::lower_program(&hir_program);
    for item in &mir_program.items {
        if let crate::mir::ir::MirItem::Fn(f) = item {
            crate::mir::borrow::check_borrows(f)
                .map_err(|e| format!("borrow error: {}", e))?;
        }
    }
    println!("check passed");
    Ok(())
}
