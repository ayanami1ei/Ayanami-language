use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::ast::*;

use super::check::check_hir_returns;
use super::import::{load_config_for_file, resolve_import_path};
use super::CompiledFile;

/// Compile a .aya source file with recursive import resolution.
/// If `target_override` is set, also produce the target artifact (.so / .a / exe).
pub fn compile_file(
    src_path: &Path,
    base_dir: &Path,
    out_dir: &Path,
    compiling: &mut HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, CompiledFile>,
    target_override: Option<&str>,
) -> Result<CompiledFile, String> {
    let canonical = src_path
        .canonicalize()
        .map_err(|e| format!("cannot resolve '{}': {}", src_path.display(), e))?;

    if compiling.contains(&canonical) {
        return Err(format!("circular import detected: {}", src_path.display()));
    }

    if let Some(cached) = cache.get(&canonical) {
        return Ok(CompiledFile {
            program: Program::new(Vec::new()),
            lir_program: crate::lir::ir::LirProgram {
                strings: Vec::new(),
                fn_names: std::collections::HashMap::new(),
                functions: Vec::new(),
                vtables: Vec::new(),
                struct_defs: std::collections::HashMap::new(),
                imported_fn_ids: HashSet::new(),
            },
            llvm_ir: String::new(),
            obj_paths: cached.obj_paths.clone(),
            link_flags: cached.link_flags.clone(),
            own_obj: cached.own_obj.clone(),
            lcl_path: cached.lcl_path.clone(),
            dep_lcl_paths: cached.dep_lcl_paths.clone(),
        });
    }

    compiling.insert(canonical.clone());

    let code = fs::read_to_string(src_path)
        .map_err(|e| format!("failed to read '{}': {}", src_path.display(), e))?;

    let program = parse_and_check(&code, src_path)?;

    let (dep_obj_paths, dep_link_flags, dep_lcl_paths, new_stmts) =
        resolve_dependencies(src_path, base_dir, out_dir, compiling, cache, &program)?;

    let mut program = program;
    program.stmts = new_stmts;

    let mut lir_program = lower_to_lir(&program, src_path)?;

    merge_dep_struct_defs(&mut lir_program, &dep_lcl_paths);

    let llvm_ir = crate::lir::emit_program(&lir_program);

    let stem = src_path.file_stem().unwrap_or(std::ffi::OsStr::new("a"));
    let own_obj = out_dir.join(stem).with_extension("o");
    let lcl_path = out_dir.join(stem).with_extension("lcl");

    crate::driver::ir_to_object(&llvm_ir, &own_obj)
        .map_err(|e| format!("llc failed for {}: {}", src_path.display(), e))?;

    build_target_artifact(
        target_override,
        &dep_obj_paths,
        &dep_link_flags,
        &own_obj,
        out_dir,
        stem,
    )?;

    emit_lcl_package(
        &program,
        &lir_program,
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
    };

    cache.insert(
        canonical.clone(),
        CompiledFile {
            program: Program::new(Vec::new()),
            lir_program: crate::lir::ir::LirProgram {
                strings: Vec::new(),
                fn_names: std::collections::HashMap::new(),
                functions: Vec::new(),
                vtables: Vec::new(),
                struct_defs: std::collections::HashMap::new(),
                imported_fn_ids: HashSet::new(),
            },
            llvm_ir: String::new(),
            obj_paths: result.obj_paths.clone(),
            link_flags: result.link_flags.clone(),
            own_obj: result.own_obj.clone(),
            lcl_path: result.lcl_path.clone(),
            dep_lcl_paths: result.dep_lcl_paths.clone(),
        },
    );

    compiling.remove(&canonical);
    Ok(result)
}

fn parse_and_check(code: &str, src_path: &Path) -> Result<Program, String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    parser
        .parse_program()
        .map_err(|e| format!("{}: error: {}", src_path.display(), e))
}

fn resolve_dependencies(
    _src_path: &Path,
    base_dir: &Path,
    out_dir: &Path,
    compiling: &mut HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, CompiledFile>,
    program: &Program,
) -> Result<(Vec<PathBuf>, Vec<String>, Vec<PathBuf>, Vec<Stmt>), String> {
    let mut dep_obj_paths = Vec::new();
    let mut dep_link_flags = Vec::new();
    let mut dep_lcl_paths = Vec::new();
    let mut new_stmts = Vec::new();

    for stmt in &program.stmts {
        if let Stmt::Import { path, .. } = stmt {
            let resolved = resolve_import_path(path, base_dir);
            let dep_path = match resolved {
                Some(p) => p,
                None => {
                    let direct = base_dir.join(path);
                    if direct.exists() {
                        direct
                    } else {
                        return Err(format!(
                            "cannot resolve import '{}' from {}: not found",
                            path,
                            base_dir.display()
                        ));
                    }
                }
            };
            let dep_target = load_config_for_file(&dep_path, out_dir);
            if dep_path.to_string_lossy().ends_with(".aya") {
                let dep_target_or_static = if dep_target.as_deref() == Some("dynamic-lib") {
                    "dynamic-lib"
                } else {
                    "static-lib"
                };
                let dep = compile_file(
                    &dep_path,
                    base_dir,
                    out_dir,
                    compiling,
                    cache,
                    Some(dep_target_or_static),
                )?;
                if dep_target_or_static == "dynamic-lib" {
                    let dep_stem = dep_path.file_stem().unwrap_or_default().to_string_lossy();
                    let canon = out_dir.canonicalize().unwrap_or_else(|_| out_dir.to_path_buf());
                    dep_link_flags.push(format!("-L{}", canon.display()));
                    dep_link_flags.push(format!("-l{}", dep_stem));
                    dep_link_flags.push(format!("-Wl,-rpath,{}", canon.display()));
                } else {
                    for p in &dep.obj_paths {
                        if !dep_obj_paths.contains(p) {
                            dep_obj_paths.push(p.clone());
                        }
                    }
                }
                dep_link_flags.extend(dep.link_flags.clone());
                let lcl_name = dep.lcl_path.to_string_lossy().into_owned();
                if !dep_lcl_paths.contains(&dep.lcl_path) {
                    dep_lcl_paths.push(dep.lcl_path.clone());
                }
                new_stmts.push(Stmt::Import {
                    path: lcl_name,
                    span: crate::span::Span::default(),
                });
            } else {
                let lcl_str = dep_path.to_string_lossy().into_owned();
                new_stmts.push(Stmt::Import {
                    path: lcl_str,
                    span: crate::span::Span::default(),
                });

                let o_name = dep_path.file_stem().unwrap_or(std::ffi::OsStr::new("a"));
                let o_path = out_dir.join(o_name).with_extension("o");
                if !o_path.exists() {
                    if let Ok((_, _, lir_binary, _)) =
                        crate::package::load_package(&dep_path.to_string_lossy())
                    {
                        if let Ok(lir_prog) =
                            crate::lir::serialize::program_from_bytes(&lir_binary)
                        {
                            let llvm_ir = crate::lir::emit_program(&lir_prog);
                            crate::driver::ir_to_object(&llvm_ir, &o_path)
                                .map_err(|e| format!("llc failed for {}: {}", dep_path.display(), e))?;
                        }
                    }
                }
                if o_path.exists() && !dep_obj_paths.contains(&o_path) {
                    dep_obj_paths.push(o_path);
                }
            }
        } else {
            new_stmts.push(stmt.clone());
        }
    }

    Ok((dep_obj_paths, dep_link_flags, dep_lcl_paths, new_stmts))
}

fn lower_to_lir(program: &Program, src_path: &Path) -> Result<crate::lir::ir::LirProgram, String> {
    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("{}: error: {}", src_path.display(), e))?;

    check_hir_returns(&hir_program, src_path)?;

    let mir_program = crate::mir::lower_program(&hir_program);
    for item in &mir_program.items {
        if let crate::mir::ir::MirItem::Fn(f) = item {
            crate::mir::borrow::check_borrows(f)
                .map_err(|e| format!("{}: borrow error: {}", src_path.display(), e))?;
        }
    }
    Ok(crate::lir::lower_program(&mir_program))
}

fn merge_dep_struct_defs(
    lir_program: &mut crate::lir::ir::LirProgram,
    dep_lcl_paths: &[PathBuf],
) {
    for lcl_path in dep_lcl_paths {
        if let Ok((_, _, lir_binary, _)) =
            crate::package::load_package(&lcl_path.to_string_lossy())
        {
            if let Ok(dep_lir) = crate::lir::serialize::program_from_bytes(&lir_binary) {
                for (name, fields) in dep_lir.struct_defs {
                    lir_program.struct_defs.entry(name).or_insert(fields);
                }
            }
        }
    }
}

fn build_target_artifact(
    target_override: Option<&str>,
    dep_obj_paths: &[PathBuf],
    dep_link_flags: &[String],
    own_obj: &Path,
    out_dir: &Path,
    stem: &std::ffi::OsStr,
) -> Result<(), String> {
    if dep_obj_paths.is_empty() && dep_link_flags.is_empty() {
        let target = target_override.unwrap_or("static-lib");
        let stem_str = stem.to_string_lossy();
        match target {
            "executable" => {
                let exe_path = out_dir.join(&*stem_str);
                crate::driver::objects_to_exe(&[own_obj.to_path_buf()], &exe_path)?;
            }
            "static-lib" => {
                let lib_path = out_dir.join(format!("lib{}.a", stem_str));
                crate::driver::object_to_static_lib(own_obj, &lib_path)?;
            }
            "dynamic-lib" => {
                let so_path = out_dir.join(format!("lib{}.so", stem_str));
                crate::driver::object_to_shared_lib(own_obj, &so_path)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn merge_symbols(
    pkg: &mut crate::package::Package,
    dep_lcl_paths: &[PathBuf],
) {
    use crate::package::PackageSymbol;
    for dep_lcl in dep_lcl_paths {
        if let Ok((syms, sources, _, _)) = crate::package::load_package(&dep_lcl.to_string_lossy())
        {
            for sym in syms {
                let pkg_sym = match sym {
                    crate::package::ImportedSymbol::Fn { name, sig } => {
                        PackageSymbol::Fn {
                            name,
                            signature: sig,
                        }
                    }
                    crate::package::ImportedSymbol::Struct { name } => {
                        PackageSymbol::Struct { name }
                    }
                    crate::package::ImportedSymbol::Namespace { name } => {
                        PackageSymbol::Namespace { name }
                    }
                    crate::package::ImportedSymbol::Interface { name } => {
                        PackageSymbol::Interface { name }
                    }
                };
                if !pkg.symbols.contains(&pkg_sym) {
                    pkg.symbols.push(pkg_sym);
                }
            }
            for src in &sources {
                if !pkg.generic_sources.contains(src) {
                    pkg.generic_sources.push(src.clone());
                }
            }
        }
    }
}

fn emit_lcl_package(
    program: &Program,
    lir_program: &crate::lir::ir::LirProgram,
    lcl_path: &Path,
    dep_lcl_paths: &[PathBuf],
    stem: &std::ffi::OsStr,
) -> Result<(), String> {
    let mut pkg = crate::package::Package::new(stem.to_string_lossy().into_owned(), "0.1.0".into());
    pkg.collect_all_symbols(&program.stmts);
    merge_symbols(&mut pkg, dep_lcl_paths);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(lir_program);
    pkg.write_to_file(&lcl_path.to_string_lossy())
        .map_err(|e| format!("package write failed for {}: {}", lcl_path.display(), e))?;
    Ok(())
}

/// Package a source file into a .lcl (LIR + symbols + metadata), no executable.
pub fn package_source(src_path: &str, _code: &str) -> Result<(), String> {
    let path = std::path::Path::new(src_path);
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let tmp_dir = std::env::temp_dir().join(format!("ayanami_pkg_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).ok();

    let mut compiling = HashSet::new();
    let mut cache = HashMap::new();
    let compiled =
        compile_file(path, base_dir, &tmp_dir, &mut compiling, &mut cache, Some("static-lib"))?;

    let exe_name = path
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy()
        .into_owned();

    println!("packaging {} -> {}.lcl", src_path, exe_name);

    let has_main = compiled.program.stmts.iter().any(|s| {
        matches!(s, crate::parser::ast::Stmt::FnDecl { name, .. } if name.as_str() == "main")
    });

    let mut pkg = crate::package::Package::new(exe_name, "0.1.0".into());
    pkg.target_types = if has_main {
        vec![
            crate::package::TargetType::Executable,
            crate::package::TargetType::StaticLib,
            crate::package::TargetType::DynamicLib,
        ]
    } else {
        vec![
            crate::package::TargetType::StaticLib,
            crate::package::TargetType::DynamicLib,
        ]
    };
    pkg.collect_all_symbols(&compiled.program.stmts);
    merge_symbols(&mut pkg, &compiled.dep_lcl_paths);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&compiled.lir_program);

    let lcl_name = format!("{}.lcl", src_path.strip_suffix(".aya").unwrap_or(src_path));
    pkg.write_to_file(&lcl_name)
        .map_err(|e| format!("package write failed: {}", e))?;
    println!("package: {}", lcl_name);

    std::fs::remove_dir_all(&tmp_dir).ok();
    Ok(())
}

/// Install a .lcl package: read LIR, emit LLVM IR, build executable or library.
pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<(), String> {
    let (_, _, lir_binary, target_types) =
        crate::package::load_package(lcl_path).map_err(|e| format!("failed to load package: {}", e))?;

    let lir_program = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| format!("failed to deserialize LIR: {}", e))?;

    let llvm_ir = crate::lir::emit_program(&lir_program);

    let base_name = std::path::Path::new(lcl_path)
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy()
        .into_owned();

    let tt = target_type
        .and_then(|s| crate::package::TargetType::from_str(s))
        .or_else(|| target_types.first().copied())
        .ok_or_else(|| "no target type specified and none in package".to_string())?;

    match tt {
        crate::package::TargetType::Executable => {
            let exe_path = format!("./{}", base_name);
            println!("installing {} -> executable {}", lcl_path, exe_path);
            crate::driver::ir_to_executable(&llvm_ir, &exe_path)?;
            println!("installed: {}", exe_path);
        }
        crate::package::TargetType::StaticLib => {
            let lib_path = format!("./lib{}.a", base_name);
            println!("installing {} -> static lib {}", lcl_path, lib_path);
            crate::driver::ir_to_library(&llvm_ir, &lib_path, "static-lib")?;
            println!("installed: {}", lib_path);
        }
        crate::package::TargetType::DynamicLib => {
            let lib_path = format!("./lib{}.so", base_name);
            println!("installing {} -> dynamic lib {}", lcl_path, lib_path);
            crate::driver::ir_to_library(&llvm_ir, &lib_path, "dynamic-lib")?;
            println!("installed: {}", lib_path);
        }
    }

    Ok(())
}

/// Build a source file into an executable, generating a .lcl package alongside.
pub fn build_source(src_path: &str, code: &str) -> Result<(), String> {
    build_source_to(src_path, code, "build")
}

/// Build with explicit output directory, using recursive import compilation.
pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<(), String> {
    build_source_with_target(src_path, _code, out_dir, None)
}

/// Build with explicit output directory and target type override.
pub fn build_source_with_target(
    src_path: &str,
    _code: &str,
    out_dir: &str,
    target_override: Option<&str>,
) -> Result<(), String> {
    let out_path = Path::new(out_dir);
    std::fs::create_dir_all(out_path)
        .map_err(|e| format!("failed to create output dir '{}': {}", out_dir, e))?;

    let src_path = Path::new(src_path);
    let base_dir = src_path.parent().unwrap_or(Path::new("."));
    let mut compiling = HashSet::new();
    let mut cache = HashMap::new();

    let compiled =
        compile_file(src_path, base_dir, out_path, &mut compiling, &mut cache, target_override)?;

    let name = src_path
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy();

    let target = target_override.unwrap_or_else(|| {
        if &*name == "main" {
            "executable"
        } else {
            "static-lib"
        }
    });

    let output_path = match target {
        "executable" => {
            let exe_path = out_path.join(&*name);
            println!("building {} -> {}", src_path.display(), exe_path.display());
            crate::driver::objects_to_exe_with_flags(
                &compiled.obj_paths,
                &compiled.link_flags,
                &exe_path,
            )
            .map_err(|e| format!("link failed: {}", e))?;
            exe_path
        }
        "static-lib" => {
            let lib_path = out_path.join(format!("lib{}.a", name));
            println!("building {} -> {}", src_path.display(), lib_path.display());
            if compiled.obj_paths.len() == 1 {
                crate::driver::object_to_static_lib(&compiled.obj_paths[0], &lib_path)?;
            } else {
                let mut cmd = std::process::Command::new("ar");
                cmd.arg("rcs").arg(&lib_path);
                for o in &compiled.obj_paths {
                    cmd.arg(o);
                }
                let status = cmd.status().map_err(|e| format!("failed to run ar: {}", e))?;
                if !status.success() {
                    return Err("ar failed".into());
                }
            }
            lib_path
        }
        "dynamic-lib" => {
            let so_path = out_path.join(format!("lib{}.so", name));
            println!("building {} -> {}", src_path.display(), so_path.display());
            if compiled.obj_paths.len() == 1 {
                crate::driver::object_to_shared_lib(&compiled.obj_paths[0], &so_path)?;
            } else {
                crate::driver::objects_to_shared_lib(&compiled.obj_paths, &so_path)?;
            }
            so_path
        }
        _ => return Err(format!("unknown target type: {}", target)),
    };

    println!("build ok: {}", output_path.display());

    let lcl_path = out_path.join(format!("{}.lcl", name));
    let mut pkg = crate::package::Package::new(name.to_string(), "0.1.0".into());
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&compiled.lir_program);
    pkg.collect_symbols(&compiled.program.stmts);
    merge_symbols(&mut pkg, &compiled.dep_lcl_paths);

    let has_main = compiled.program.stmts.iter().any(|s| {
        matches!(s, crate::parser::ast::Stmt::FnDecl { name, .. } if name.as_str() == "main")
    });
    pkg.target_types = if has_main {
        vec![crate::package::TargetType::Executable]
    } else {
        vec![
            crate::package::TargetType::StaticLib,
            crate::package::TargetType::DynamicLib,
        ]
    };
    pkg.write_to_file(&lcl_path.to_string_lossy())
        .map_err(|e| format!("package write failed: {}", e))?;
    println!("package: {}", lcl_path.display());

    Ok(())
}

/// Run a built executable.  Looks in `build/` first, then cwd.
pub fn run_executable(exe_name: &str) -> Result<i32, String> {
    let build_path = format!("build/{}", exe_name);
    let exe_path = if std::path::Path::new(&build_path).exists() {
        build_path
    } else {
        exe_name.to_string()
    };
    println!("running: {}", exe_path);
    let status = std::process::Command::new(&exe_path)
        .status()
        .map_err(|e| format!("failed to run '{}': {}", exe_path, e))?;
    Ok(status.code().unwrap_or(-1))
}
