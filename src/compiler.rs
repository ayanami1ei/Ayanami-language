use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::ast::{
    BinaryOp, Block, Expr, Literal, Program, Stmt, Type, UnaryOp,
};

/// Result of compiling a single file (including its recursively-resolved imports).
pub struct CompiledFile {
    pub program: Program,
    pub lir_program: crate::lir::ir::LirProgram,
    pub llvm_ir: String,
    /// All .o files this file depends on (including its own)
    pub obj_paths: Vec<PathBuf>,
    /// Link flags for dynamic libraries: -L and -l
    pub link_flags: Vec<String>,
    /// Path to this file's own .o
    pub own_obj: PathBuf,
    /// Path to this file's .lcl
    pub lcl_path: PathBuf,
}

/// Compile a .aya source file with recursive import resolution.
/// If `target_override` is set, also produce the target artifact (.so / .a / exe).
pub fn compile_file(
    src_path: &Path,
    base_dir: &Path,
    out_dir: &Path,
    compiling: &mut HashSet<PathBuf>,
    cache: &mut std::collections::HashMap<PathBuf, CompiledFile>,
    target_override: Option<&str>,
) -> Result<CompiledFile, String> {
    let canonical = src_path.canonicalize()
        .map_err(|e| format!("cannot resolve '{}': {}", src_path.display(), e))?;

    // Cycle detection
    if compiling.contains(&canonical) {
        return Err(format!("circular import detected: {}", src_path.display()));
    }

    // Cache hit
    if let Some(cached) = cache.get(&canonical) {
        return Ok(CompiledFile {
            program: Program::new(Vec::new()),
            lir_program: crate::lir::ir::LirProgram {
                strings: Vec::new(), fn_names: std::collections::HashMap::new(),
                functions: Vec::new(), vtables: Vec::new(),
                struct_defs: std::collections::HashMap::new(),
                imported_fn_ids: HashSet::new(),
            },
            llvm_ir: String::new(),
            obj_paths: cached.obj_paths.clone(),
            link_flags: cached.link_flags.clone(),
            own_obj: cached.own_obj.clone(),
            lcl_path: cached.lcl_path.clone(),
        });
    }

    compiling.insert(canonical.clone());

    let code = fs::read_to_string(src_path)
        .map_err(|e| format!("failed to read '{}': {}", src_path.display(), e))?;

    let mut lexer = crate::lexer::Lexer::new(&code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    let mut program = parser.parse_program()
        .map_err(|e| format!("Parse error in {}: {}", src_path.display(), e))?;

    // Resolve imports: for each .aya import, compile the dependency
    let mut dep_obj_paths = Vec::new();
    let mut dep_link_flags = Vec::new();
    let mut new_stmts = Vec::new();
    for stmt in &program.stmts {
        if let Stmt::Import { path, .. } = stmt {
            if path.ends_with(".aya") {
                let dep_path = base_dir.join(path);
                let dep_target = load_config_for_file(&dep_path, out_dir);
                let dep = compile_file(&dep_path, base_dir, out_dir, compiling, cache, dep_target.as_deref())?;
                // If dep is a dynamic lib, add link flags instead of its .o
                if dep_target.as_deref() == Some("dynamic-lib") {
                    let dep_stem = dep_path.file_stem().unwrap_or_default().to_string_lossy();
                    dep_link_flags.push(format!("-L{}", out_dir.canonicalize().unwrap_or_else(|_| out_dir.to_path_buf()).display()));
                    dep_link_flags.push(format!("-l{}", dep_stem));
                    dep_link_flags.push(format!("-Wl,-rpath,{}", out_dir.canonicalize().unwrap_or_else(|_| out_dir.to_path_buf()).display()));
                } else {
                    dep_obj_paths.extend(dep.obj_paths.clone());
                }
                dep_link_flags.extend(dep.link_flags.clone());
                let lcl_name = dep.lcl_path.to_string_lossy().into_owned();
                new_stmts.push(Stmt::Import { path: lcl_name, span: crate::span::Span::default() });
            } else {
                new_stmts.push(stmt.clone());
            }
        } else {
            new_stmts.push(stmt.clone());
        }
    }
    program.stmts = new_stmts;

    // HIR → MIR → LIR
    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error in {}: {}", src_path.display(), e))?;
    let mir_program = crate::mir::lower_program(&hir_program);
    let lir_program = crate::lir::lower_program(&mir_program);
    let llvm_ir = crate::lir::emit_program(&lir_program);

    // Compute output paths
    let stem = src_path.file_stem().unwrap_or(std::ffi::OsStr::new("a"));
    let own_obj = out_dir.join(stem).with_extension("o");
    let lcl_path = out_dir.join(stem).with_extension("lcl");

    // Emit .o
    crate::driver::ir_to_object(&llvm_ir, &own_obj)
        .map_err(|e| format!("llc failed for {}: {}", src_path.display(), e))?;

    // If a target type is specified and this file has no imports (standalone),
    // produce the target artifact.  Files with deps will be linked by the root.
    // Also skip if there are link flags (dynamic lib linking handled by root).
    let stem_str = stem.to_string_lossy();
    if dep_obj_paths.is_empty() && dep_link_flags.is_empty() {
        let target = target_override.unwrap_or("static-lib");
        match target {
            "executable" => {
                let exe_path = out_dir.join(&*stem_str);
                crate::driver::objects_to_exe(&[own_obj.clone()], &exe_path)?;
            }
            "static-lib" => {
                let lib_path = out_dir.join(format!("lib{}.a", stem_str));
                crate::driver::object_to_static_lib(&own_obj, &lib_path)?;
            }
            "dynamic-lib" => {
                let so_path = out_dir.join(format!("lib{}.so", stem_str));
                crate::driver::object_to_shared_lib(&own_obj, &so_path)?;
            }
            _ => {}
        }
    }

    // Emit .lcl (include all symbols for .aya imports)
    let mut pkg = crate::package::Package::new(stem.to_string_lossy().into_owned(), "0.1.0".into());
    pkg.collect_all_symbols(&program.stmts);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&lir_program);
    pkg.write_to_file(&lcl_path.to_string_lossy())
        .map_err(|e| format!("package write failed for {}: {}", src_path.display(), e))?;

    let mut all_objs = dep_obj_paths;
    all_objs.push(own_obj.clone());

    let result = CompiledFile {
        program,
        lir_program,
        llvm_ir,
        obj_paths: all_objs,
        link_flags: dep_link_flags,
        own_obj,
        lcl_path,
    };

    cache.insert(canonical.clone(), CompiledFile {
        program: Program::new(Vec::new()),
        lir_program: crate::lir::ir::LirProgram {
            strings: Vec::new(), fn_names: std::collections::HashMap::new(),
            functions: Vec::new(), vtables: Vec::new(),
            struct_defs: std::collections::HashMap::new(),
            imported_fn_ids: HashSet::new(),
        },
        llvm_ir: String::new(),
        obj_paths: result.obj_paths.clone(),
        link_flags: result.link_flags.clone(),
        own_obj: result.own_obj.clone(),
        lcl_path: result.lcl_path.clone(),
    });

    compiling.remove(&canonical);
    Ok(result)
}

/// Try to load project config and resolve target for a file path.
fn load_config_for_file(file_path: &Path, _out_dir: &Path) -> Option<String> {
    // Walk up from file to find ayanami.toml
    let mut dir = file_path.parent()?;
    loop {
        let toml = dir.join("ayanami.toml");
        if toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&toml) {
                let cfg = crate::package::config::ProjectConfig::load(&content);
                let rel_path = file_path.strip_prefix(dir).ok()?;
                return Some(cfg.resolve_target(rel_path).to_string());
            }
        }
        if let Some(parent) = dir.parent() { dir = parent; } else { break; }
    }
    None
}

/// Simple compile from source text (no import resolution).
pub struct CompileResult {
    pub llvm_ir: String,
    pub program: Program,
    pub lir_program: crate::lir::ir::LirProgram,
}

pub fn compile_source(code: &str) -> Result<CompileResult, String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;
    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error: {}", e))?;
    let mir_program = crate::mir::lower_program(&hir_program);
    let lir_program = crate::lir::lower_program(&mir_program);
    let llvm_ir = crate::lir::emit_program(&lir_program);
    Ok(CompileResult { llvm_ir, program, lir_program })
}

/// Run frontend checks (lex, parse, HIR), write stage outputs to a directory.
pub fn check_source(code: &str, out_dir: &str) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("failed to create output dir '{}': {}", out_dir, e))?;

    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();

    let mut tokens_out = String::new();
    for t in tokens.iter().filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF)) {
        writeln!(tokens_out, "  {}", t).unwrap();
    }
    fs::write(format!("{}/tokens.txt", out_dir), &tokens_out)
        .map_err(|e| format!("write tokens failed: {}", e))?;

    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    let ast_out = format_program(&program);
    fs::write(format!("{}/ast.txt", out_dir), &ast_out)
        .map_err(|e| format!("write ast failed: {}", e))?;

    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error: {}", e))?;
    let hir_out = crate::hir::hir_program_to_string(&hir_program);
    fs::write(format!("{}/hir.txt", out_dir), &hir_out)
        .map_err(|e| format!("write hir failed: {}", e))?;

    let mir_program = crate::mir::lower_program(&hir_program);
    let mir_out = crate::mir::mir_program_to_string(&mir_program);
    fs::write(format!("{}/mir.txt", out_dir), &mir_out)
        .map_err(|e| format!("write mir failed: {}", e))?;

    let lir_program = crate::lir::lower_program(&mir_program);
    let lir_out = crate::lir::lir_program_to_string(&lir_program);
    fs::write(format!("{}/lir.txt", out_dir), &lir_out)
        .map_err(|e| format!("write lir failed: {}", e))?;

    let llvm_ir = crate::lir::emit_program(&lir_program);
    fs::write(format!("{}/llvm_ir.ll", out_dir), &llvm_ir)
        .map_err(|e| format!("write llvm_ir failed: {}", e))?;

    println!("check passed, stage output in {}/", out_dir);
    Ok(())
}

/// Package a source file into a .lcl (LIR + symbols + metadata), no executable.
pub fn package_source(src_path: &str, code: &str) -> Result<(), String> {
    let result = compile_source(code)?;

    let exe_name = {
        let p = std::path::Path::new(src_path);
        p.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned()
    };

    println!("packaging {} -> {}.lcl", src_path, exe_name);

    let mut pkg = crate::package::Package::new(exe_name, "0.1.0".into());
    let has_main = result.program.stmts.iter().any(|s| matches!(s,
        crate::parser::ast::Stmt::FnDecl { name, .. } if name.as_str() == "main"
    ));
    pkg.target_types = if has_main {
        vec![crate::package::TargetType::Executable,
             crate::package::TargetType::StaticLib,
             crate::package::TargetType::DynamicLib]
    } else {
        vec![crate::package::TargetType::StaticLib,
             crate::package::TargetType::DynamicLib]
    };
    pkg.collect_symbols(&result.program.stmts);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&result.lir_program);

    let lcl_name = format!("{}.lcl", src_path.strip_suffix(".aya").unwrap_or(src_path));
    pkg.write_to_file(&lcl_name).map_err(|e| format!("package write failed: {}", e))?;
    println!("package: {}", lcl_name);

    Ok(())
}

/// Install a .lcl package: read LIR, emit LLVM IR, build executable or library.
pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<(), String> {
    let (_, _, lir_binary, target_types) = crate::package::load_package(lcl_path)
        .map_err(|e| format!("failed to load package: {}", e))?;

    let lir_program = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| format!("failed to deserialize LIR: {}", e))?;

    let llvm_ir = crate::lir::emit_program(&lir_program);

    let base_name = std::path::Path::new(lcl_path)
        .file_stem().unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy().into_owned();

    // Determine which target type to build
    let tt = target_type.and_then(|s| crate::package::TargetType::from_str(s))
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
pub fn build_source_with_target(src_path: &str, _code: &str, out_dir: &str, target_override: Option<&str>) -> Result<(), String> {
    let out_path = Path::new(out_dir);
    std::fs::create_dir_all(out_path)
        .map_err(|e| format!("failed to create output dir '{}': {}", out_dir, e))?;

    let src_path = Path::new(src_path);
    let base_dir = src_path.parent().unwrap_or(Path::new("."));
    let mut compiling = HashSet::new();
    let mut cache = HashMap::new();

    let compiled = compile_file(src_path, base_dir, out_path, &mut compiling, &mut cache, target_override)?;

    let name = src_path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy();

    // Determine target: override → heuristic
    let target = target_override.unwrap_or_else(|| {
        if &*name == "main" { "executable" } else { "static-lib" }
    });

    // Build the target artifact
    let output_path = match target {
        "executable" => {
            let exe_path = out_path.join(&*name);
            println!("building {} -> {}", src_path.display(), exe_path.display());
            crate::driver::objects_to_exe_with_flags(&compiled.obj_paths, &compiled.link_flags, &exe_path)
                .map_err(|e| format!("link failed: {}", e))?;
            exe_path
        }
        "static-lib" => {
            let lib_path = out_path.join(format!("lib{}.a", name));
            println!("building {} -> {}", src_path.display(), lib_path.display());
            // For static lib, just archive all .o files
            if compiled.obj_paths.len() == 1 {
                crate::driver::object_to_static_lib(&compiled.obj_paths[0], &lib_path)?;
            } else {
                // Multiple .o files: link into single .o first, then archive
                crate::driver::objects_to_exe(&compiled.obj_paths, &out_path.join("_temp_exe"))?;
                crate::driver::object_to_static_lib(&out_path.join("_temp_exe.o"), &lib_path)?;
                let _ = std::fs::remove_file(&out_path.join("_temp_exe"));
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

    // Generate root .lcl
    let lcl_path = out_path.join(format!("{}.lcl", name));
    let mut pkg = crate::package::Package::new(name.to_string(), "0.1.0".into());
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&compiled.lir_program);
    pkg.collect_symbols(&compiled.program.stmts);
    let has_main = compiled.program.stmts.iter().any(|s| matches!(s,
        crate::parser::ast::Stmt::FnDecl { name, .. } if name.as_str() == "main"
    ));
    pkg.target_types = if has_main {
        vec![crate::package::TargetType::Executable]
    } else {
        vec![crate::package::TargetType::StaticLib, crate::package::TargetType::DynamicLib]
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

// ====================================================================
//  Pretty printer (for diagnostics / check output)
// ====================================================================

fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::Int(_) => "Int".into(),
        Type::Float(_) => "Float".into(),
        Type::Char(_) => "Char".into(),
        Type::Bool(_) => "Bool".into(),
        Type::Void(_) => "Void".into(),
        Type::Named(s, _) => format!("Named({})", s),
        Type::Array(inner, _) => format!("[{}]", format_type(inner)),
        Type::Unique(inner, _) => format!("unique {}", format_type(inner)),
        Type::Shared(inner, _) => format!("shared {}", format_type(inner)),
        Type::Weak(inner, _) => format!("weak {}", format_type(inner)),
        Type::Self_(_) => "Self".into(),
    }
}

fn format_op(op: &BinaryOp) -> &str {
    match op {
        BinaryOp::Add => "Add",
        BinaryOp::Sub => "Sub",
        BinaryOp::Mul => "Mul",
        BinaryOp::Div => "Div",
        BinaryOp::Mod => "Mod",
        BinaryOp::Eq => "Eq",
        BinaryOp::Neq => "Neq",
        BinaryOp::Lt => "Lt",
        BinaryOp::Gt => "Gt",
        BinaryOp::Le => "Le",
        BinaryOp::Ge => "Ge",
        BinaryOp::And => "And",
        BinaryOp::Or => "Or",
    }
}

fn format_unary(op: &UnaryOp) -> &str {
    match op {
        UnaryOp::Neg => "Neg",
        UnaryOp::Not => "Not",
    }
}

fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => format!("Int({})", n),
        Literal::Float(n, _) => format!("Float({})", n),
        Literal::Char(c, _) => format!("Char('{}')", c),
        Literal::String(s, _) => format!("String(\"{}\")", s),
        Literal::Bool(b, _) => format!("Bool({})", b),
    }
}

pub fn format_program(program: &Program) -> String {
    let mut s = String::new();
    writeln!(s, "Program").unwrap();
    for stmt in &program.stmts {
        write_stmt(stmt, 1, &mut s);
    }
    s
}

fn write_expr(expr: &Expr, level: usize, w: &mut impl Write) {
    match expr {
        Expr::Literal(lit) => {
            writeln!(w, "{}{}", pad(level), format_literal(lit)).unwrap();
        }
        Expr::Ident(name, _) => {
            writeln!(w, "{}Ident({})", pad(level), name).unwrap();
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            writeln!(w, "{}Binary {{ op: {} }}", pad(level), format_op(op)).unwrap();
            writeln!(w, "{}  lhs:", pad(level)).unwrap();
            write_expr(lhs, level + 1, w);
            writeln!(w, "{}  rhs:", pad(level)).unwrap();
            write_expr(rhs, level + 1, w);
        }
        Expr::Unary { op, arg, .. } => {
            writeln!(w, "{}Unary {{ op: {} }}", pad(level), format_unary(op)).unwrap();
            write_expr(arg, level + 1, w);
        }
        Expr::FnCall { name, args, .. } => {
            writeln!(w, "{}FnCall {{ name: {} }}", pad(level), name).unwrap();
            writeln!(w, "{}  args:", pad(level)).unwrap();
            for arg in args {
                write_expr(arg, level + 1, w);
            }
        }
        Expr::Move(expr, _) => { writeln!(w, "{}Move", pad(level)).unwrap(); write_expr(expr, level + 1, w); }
        Expr::Clone(expr, _) => { writeln!(w, "{}Clone", pad(level)).unwrap(); write_expr(expr, level + 1, w); }
        Expr::ToUnique(expr, _) => { writeln!(w, "{}ToUnique", pad(level)).unwrap(); write_expr(expr, level + 1, w); }
        Expr::ToShared(expr, _) => { writeln!(w, "{}ToShared", pad(level)).unwrap(); write_expr(expr, level + 1, w); }
        Expr::ToWeak(expr, _) => { writeln!(w, "{}ToWeak", pad(level)).unwrap(); write_expr(expr, level + 1, w); }
        Expr::MethodCall { object, method, args, .. } => {
            writeln!(w, "{}MethodCall {{ method: {} }}", pad(level), method).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            for arg in args { write_expr(arg, level + 1, w); }
        }
        Expr::FieldAccess { object, field, .. } => {
            writeln!(w, "{}FieldAccess {{ field: {} }}", pad(level), field).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
        }
        Expr::StructLiteral { type_name, fields, .. } => {
            writeln!(w, "{}StructLiteral {{ type: {} }}", pad(level), type_name).unwrap();
            for (name, val) in fields {
                writeln!(w, "{}  {} =", pad(level), name).unwrap();
                write_expr(val, level + 1, w);
            }
        }
        Expr::ArrayLiteral(elems, _) => {
            writeln!(w, "{}ArrayLiteral", pad(level)).unwrap();
            for e in elems { write_expr(e, level + 1, w); }
        }
        Expr::Index { object, index, .. } => {
            writeln!(w, "{}Index", pad(level)).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  index:", pad(level)).unwrap();
            write_expr(index, level + 1, w);
        }
    }
}

fn write_block(block: &Block, level: usize, w: &mut impl Write) {
    writeln!(w, "{}Block {{", pad(level)).unwrap();
    for stmt in &block.stmts { write_stmt(stmt, level + 1, w); }
    writeln!(w, "{}}}", pad(level)).unwrap();
}

fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write) {
    let p = pad(level);
    match stmt {
        Stmt::FnDecl { name, params, return_type, body, .. } => {
            writeln!(w, "{}FnDecl {{ name: {}, return: {} }}", p, name, format_type(return_type)).unwrap();
            for (n, t) in params { writeln!(w, "{}    {}: {}", p, n, format_type(t)).unwrap(); }
            writeln!(w, "{}  body:", p).unwrap();
            write_block(body, level + 1, w);
        }
        Stmt::Assign { name, value, .. } => {
            writeln!(w, "{}Assign {{ name: {} }}", p, name).unwrap();
            writeln!(w, "{}  value:", p).unwrap();
            write_expr(value, level + 1, w);
        }
        Stmt::Return { value, .. } => {
            writeln!(w, "{}Return", p).unwrap();
            if let Some(val) = value { write_expr(val, level + 1, w); }
        }
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            writeln!(w, "{}If", p).unwrap();
            writeln!(w, "{}  cond:", p).unwrap();
            write_expr(cond, level + 1, w);
            writeln!(w, "{}  then:", p).unwrap();
            write_block(then_block, level + 1, w);
            for (c, b) in elifs {
                writeln!(w, "{}  elif:", p).unwrap();
                write_expr(c, level + 1, w);
                write_block(b, level + 1, w);
            }
            if let Some(b) = else_block {
                writeln!(w, "{}  else:", p).unwrap();
                write_block(b, level + 1, w);
            }
        }
        Stmt::For { iterator, start, end, step, body, .. } => {
            writeln!(w, "{}For {{ iterator: {} }}", p, iterator).unwrap();
            write_expr(start, level + 1, w);
            write_expr(end, level + 1, w);
            if let Some(s) = step { write_expr(s, level + 1, w); }
            write_block(body, level + 1, w);
        }
        Stmt::While { cond, body, .. } => {
            writeln!(w, "{}While", p).unwrap();
            write_expr(cond, level + 1, w);
            write_block(body, level + 1, w);
        }
        Stmt::Namespace { name, items, .. } => {
            writeln!(w, "{}Namespace {{ name: {} }}", p, name).unwrap();
            for item in items { write_stmt(item, level + 1, w); }
        }
        Stmt::ExprStmt { expr, .. } => {
            writeln!(w, "{}ExprStmt", p).unwrap();
            write_expr(expr, level + 1, w);
        }
        Stmt::StructDef { name, fields, .. } => {
            writeln!(w, "{}StructDef {{ name: {} }}", p, name).unwrap();
            for (fname, fty) in fields { writeln!(w, "{}  {}: {}", p, fname, format_type(fty)).unwrap(); }
        }
        Stmt::InterfaceDef { name, methods, .. } => {
            writeln!(w, "{}InterfaceDef {{ name: {} }}", p, name).unwrap();
            for m in methods {
                let params: Vec<String> = m.params.iter().map(|(n, t)| format!("{}: {}", n, format_type(t))).collect();
                writeln!(w, "{}  fn {}({}) -> {}", p, m.name, params.join(", "), format_type(&m.return_type)).unwrap();
            }
        }
        Stmt::ImplBlock { type_name, methods, .. } => {
            writeln!(w, "{}ImplBlock {{ name: {} }}", p, type_name).unwrap();
            for m in methods { write_stmt(m, level + 1, w); }
        }
        Stmt::Import { path, .. } => {
            writeln!(w, "{}Import {{ path: {} }}", p, path).unwrap();
        }
    }
}

// ====================================================================
//  Tests
// ====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::ir::*;
    use crate::lexer::Lexer;

    fn parse(code: &str) -> Program {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize_all();
        let filtered: Vec<_> = tokens.into_iter()
            .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
            .collect();
        let mut parser = crate::parser::Parser::new(filtered);
        parser.parse_program().expect("parse failed")
    }

    fn lower(program: &Program) -> HirProgram {
        crate::hir::lower_program(program).expect("lower failed")
    }

    #[test]
    fn test_simple_fn() {
        let program = parse("fn main()->int{ return 1; }");
        let hir = lower(&program);
        assert_eq!(hir.items.len(), 1);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.name.as_str(), "main");
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Literal(HirLiteral::Int(1), HirType::Int)) } => {}
                    _ => panic!("expected Return Int(1)"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_fn_with_params() {
        let program = parse("fn add(int a, int b)->int{ return a+b; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.params.len(), 2);
                assert_eq!(f.params[0].1, HirType::Int);
                assert_eq!(f.params[1].1, HirType::Int);
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_fn_with_unique_shared_weak_params() {
        let program = parse("fn foo(unique int x, shared float y, weak char z)->int{ return 0; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.params[0].1, HirType::Unique(Box::new(HirType::Int)));
                assert_eq!(f.params[1].1, HirType::Shared(Box::new(HirType::Float)));
                assert_eq!(f.params[2].1, HirType::Weak(Box::new(HirType::Char)));
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_assign() {
        let program = parse("fn main()->int{ a=1; return a; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.locals.len(), 1);
                match &f.body.stmts[0] {
                    HirStmt::Assign { target: HirExpr::Local(VarId(0), HirType::Int), value: HirExpr::Literal(HirLiteral::Int(1), HirType::Int) } => {}
                    _ => panic!("expected Assign"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_literals() {
        let program = parse(r#"fn main()->int{ a=1; b=2.0; c='x'; d="hello"; return 0; }"#);
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert!(matches!(&f.body.stmts[0], HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Int(1), HirType::Int), .. }));
                assert!(matches!(&f.body.stmts[1], HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Float(_), HirType::Float), .. }));
                assert!(matches!(&f.body.stmts[2], HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Char('x'), HirType::Char), .. }));
                match &f.body.stmts[3] {
                    HirStmt::Assign { value: HirExpr::Literal(HirLiteral::String(s), _), .. } => assert_eq!(s, "hello"),
                    _ => panic!("expected String"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_if_else() {
        let program = parse("fn main()->int{ if 1>0{ return 1; }else{ return 2; } }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::If { elifs, else_block, .. } => {
                    assert!(elifs.is_empty());
                    assert!(else_block.is_some());
                }
                _ => panic!("expected If"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_while() {
        let program = parse("fn main()->int{ a=0; while a<10{ a=a+1; } return a; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => match &f.body.stmts[1] {
                HirStmt::While { cond, .. } => {
                    assert!(matches!(cond, HirExpr::Binary { op: BinaryOp::Lt, .. }));
                }
                _ => panic!("expected While"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_for_loop() {
        let program = parse("fn main(int a)->int{ for i in (1, 10){ a = a + i; } return a; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Block(stmts) => assert_eq!(stmts.len(), 2),
                _ => panic!("expected Block"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_namespace() {
        let program = parse("namespace math{ fn add(int a, int b)->int{ return a+b; } fn sub(int a, int b)->int{ return a-b; } }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Namespace { name, items } => {
                assert_eq!(name.as_str(), "math");
                assert_eq!(items.len(), 2);
            }
            _ => panic!("expected Namespace"),
        }
    }

    #[test]
    fn test_fn_call() {
        let program = parse("fn add(int a, int b)->int{ return a+b; } fn main()->int{ return add(1,2); }");
        let hir = lower(&program);
        match &hir.items[1] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Call { fn_id, args, ty }) } => {
                    assert_eq!(fn_id.0, 0);
                    assert_eq!(args.len(), 2);
                    assert_eq!(*ty, HirType::Int);
                }
                _ => panic!("expected Return with Call"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_move_clone() {
        let program = parse("fn foo(unique int x)->unique int{ return move x; } fn bar(shared int y)->int{ return clone y; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Move(_, ty)) } => assert_eq!(*ty, HirType::Unique(Box::new(HirType::Int))),
                _ => panic!("expected Move"),
            },
            _ => panic!("expected Fn"),
        }
        match &hir.items[1] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Clone(_, ty)) } => assert_eq!(*ty, HirType::Shared(Box::new(HirType::Int))),
                _ => panic!("expected Clone"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_undefined_variable() {
        let result = crate::hir::lower_program(&parse("fn main()->int{ return x; }"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined variable"));
    }

    #[test]
    fn test_undefined_function() {
        let result = crate::hir::lower_program(&parse("fn main()->int{ foo(); return 0; }"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined function"));
    }

    #[test]
    fn test_fn_decl_inside_body() {
        let result = crate::hir::lower_program(&parse("fn main()->int{ fn inner()->int{ return 1; } return 0; }"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unexpected declaration"));
    }

    #[test]
    fn test_non_fn_top_level() {
        let result = crate::hir::lower_program(&parse("a=1;"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unexpected top-level statement"));
    }

    #[test]
    fn test_overload_by_param_types() {
        let program = parse("fn foo(int a)->int{ return a; } fn foo(float b)->float{ return b; } fn main()->int{ return foo(1); }");
        let hir = lower(&program);
        match &hir.items[2] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Call { fn_id, .. }) } => assert_eq!(fn_id.0, 0),
                _ => panic!("expected Call"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_for_loop_with_step() {
        let program = parse("fn main(int a)->int{ for i in (0, 100, 2){ a = a + i; } return a; }");
        let hir = lower(&program);
        match &hir.items[0] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Block(stmts) => { assert!(stmts.len() >= 2); }
                _ => panic!("expected Block"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_overload_resolve_float() {
        let program = parse("fn foo(int a)->int{ return a; } fn foo(float b)->float{ return b; } fn main()->float{ return foo(1.0); }");
        let hir = lower(&program);
        match &hir.items[2] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Call { fn_id, ty, .. }) } => {
                    assert_eq!(fn_id.0, 1);
                    assert_eq!(*ty, HirType::Float);
                }
                _ => panic!("expected Call"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_overload_three_overloads() {
        let program = parse("fn add(int a, int b)->int{ return a+b; } fn add(float a, float b)->float{ return a+b; } fn add(int a, int b, int c)->int{ return a+b+c; } fn main()->float{ return add(1.0, 2.0); }");
        let hir = lower(&program);
        match &hir.items[3] {
            HirItem::Fn(f) => match &f.body.stmts[0] {
                HirStmt::Return { value: Some(HirExpr::Call { fn_id, .. }) } => assert_eq!(fn_id.0, 1),
                _ => panic!("expected Call"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_overload_no_match_error() {
        let program = parse("fn foo(int a)->int{ return a; } fn foo(float b)->float{ return b; } fn main()->int{ return foo('x'); }");
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no matching overload"));
    }

    #[test]
    fn test_overload_duplicate_sig_error() {
        let result = crate::hir::lower_program(&parse(
            "fn foo(int a)->int{ return a; } fn foo(int b)->int{ return b; } fn main()->int{ return foo(1); }",
        ));
        assert!(result.is_err());
    }
}
