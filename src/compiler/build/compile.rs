use super::*;
use super::deps::{merge_dep_struct_defs, resolve_dependencies};
use super::lir::{build_target_artifact, lower_to_lir};
use super::package::emit_lcl_package;

pub fn compile_file(
    src_path: &Path,
    base_dir: &Path,
    out_dir: &Path,
    compiling: &mut HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, CompiledFile>,
    target_override: Option<&str>,
    runtime: Option<&Path>,
) -> Result<CompiledFile> {
    let canonical = src_path
        .canonicalize()
        .map_err(|e| Error::Import(format!("cannot resolve '{}': {}", src_path.display(), e)))?;

    if compiling.contains(&canonical) {
        return Err(Error::CircularImport { path: canonical.clone() });
    }

    if let Some(cached) = cache.get(&canonical) {
        return Ok(CompiledFile {
            program: Program::new(Vec::new()),
            lir_program: crate::lir::ir::LirProgram {
                specialized_fns: Default::default(),
                strings: Vec::new(),
                globals: Vec::new(),
                extern_globals: Vec::new(),
                fn_names: std::collections::HashMap::new(),
                functions: Vec::new(),
                vtables: Vec::new(),
                struct_defs: std::collections::HashMap::new(),
                generic_struct_params: std::collections::HashMap::new(),
                extern_decls: Vec::new(),
                effect_summaries: std::collections::HashMap::new(),
            },
            llvm_ir: String::new(),
            obj_paths: cached.obj_paths.clone(),
            link_flags: cached.link_flags.clone(),
            own_obj: cached.own_obj.clone(),
            lcl_path: cached.lcl_path.clone(),
            dep_lcl_paths: cached.dep_lcl_paths.clone(),
            consts: Vec::new(),
            statics: Vec::new(),
        });
    }

    compiling.insert(canonical.clone());

    let code = fs::read_to_string(src_path)
        .map_err(|e| Error::Compile(format!("failed to read '{}': {}", src_path.display(), e)))?;

    let program = parse_and_check(&code, src_path)?;

    let (dep_obj_paths, dep_link_flags, dep_lcl_paths, new_stmts) =
        resolve_dependencies(src_path, base_dir, out_dir, compiling, cache, &program, runtime)?;

    let mut program = program;
    program.stmts = new_stmts;

    let (mut lir_program, hir_consts, hir_statics) = lower_to_lir(&program, src_path)?;

    merge_dep_struct_defs(&mut lir_program, &dep_lcl_paths);

    let llvm_ir = crate::lir::emit_program(&lir_program);

    let stem = src_path.file_stem().unwrap_or(std::ffi::OsStr::new("a"));
    let own_obj = out_dir.join(stem).with_extension("o");
    let lcl_path = out_dir.join(stem).with_extension("lcl");

    crate::driver::ir_to_object(&llvm_ir, &own_obj)
        .map_err(|e| Error::Compile(format!("llc failed for {}: {}", src_path.display(), e)))?;

    build_target_artifact(
        target_override,
        &dep_obj_paths,
        &dep_link_flags,
        &own_obj,
        out_dir,
        stem,
        runtime,
    )?;

    emit_lcl_package(
        &program,
        &lir_program,
        &hir_consts,
        &hir_statics,
        &lcl_path,
        &dep_lcl_paths,
        stem,
    )?;

    let mut all_objs = vec![own_obj.clone()];
    all_objs.extend(dep_obj_paths);

    let result = CompiledFile {
        program,
        lir_program,
        llvm_ir,
        obj_paths: all_objs,
        link_flags: dep_link_flags,
        own_obj,
        lcl_path,
        dep_lcl_paths,
        consts: hir_consts,
        statics: hir_statics,
    };

    cache.insert(
        canonical.clone(),
        CompiledFile {
            program: Program::new(Vec::new()),
            lir_program: crate::lir::ir::LirProgram {
                specialized_fns: Default::default(),
                strings: Vec::new(),
                globals: Vec::new(),
                extern_globals: Vec::new(),
                fn_names: std::collections::HashMap::new(),
                functions: Vec::new(),
                vtables: Vec::new(),
                struct_defs: std::collections::HashMap::new(),
                generic_struct_params: std::collections::HashMap::new(),
                extern_decls: Vec::new(),
                effect_summaries: std::collections::HashMap::new(),
            },
            llvm_ir: String::new(),
            obj_paths: result.obj_paths.clone(),
            link_flags: result.link_flags.clone(),
            own_obj: result.own_obj.clone(),
            lcl_path: result.lcl_path.clone(),
            dep_lcl_paths: result.dep_lcl_paths.clone(),
            consts: Vec::new(),
            statics: Vec::new(),
        },
    );

    compiling.remove(&canonical);
    Ok(result)
}

pub(super) fn parse_and_check(code: &str, src_path: &Path) -> Result<Program> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    parser
        .parse_program()
        .map_err(|e| Error::Compile(format!("{}: {}", src_path.display(), e)))
}
