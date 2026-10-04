use super::*;
use super::deps::merge_symbols;

pub(super) fn emit_lcl_package(
    program: &Program,
    lir_program: &crate::lir::ir::LirProgram,
    lcl_path: &Path,
    dep_lcl_paths: &[PathBuf],
    stem: &std::ffi::OsStr,
) -> Result<()> {
    let mut pkg = crate::package::Package::new(stem.to_string_lossy().into_owned(), env!("CARGO_PKG_VERSION").into());
    pkg.set_effect_summaries(lir_program.effect_summaries.clone());
    pkg.collect_all_symbols(&program.stmts);
    merge_symbols(&mut pkg, dep_lcl_paths);
    pkg.deps = dep_stems(dep_lcl_paths);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(lir_program);
    pkg.write_to_file(&lcl_path.to_string_lossy())
        .map_err(|e| Error::Compile(format!("package write failed for {}: {}", lcl_path.display(), e)))?;
    Ok(())
}

pub fn package_source(src_path: &str, _code: &str) -> Result<()> {
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

    let mut pkg = crate::package::Package::new(exe_name, env!("CARGO_PKG_VERSION").into());
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
    pkg.set_effect_summaries(compiled.lir_program.effect_summaries.clone());
    pkg.collect_all_symbols(&compiled.program.stmts);
    merge_symbols(&mut pkg, &compiled.dep_lcl_paths);
    pkg.deps = dep_stems(&compiled.dep_lcl_paths);
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&compiled.lir_program);

    let lcl_name = format!("{}.lcl", src_path.strip_suffix(".aya").unwrap_or(src_path));
    pkg.write_to_file(&lcl_name)
        .map_err(|e| Error::Compile(format!("package write failed: {}", e)))?;

    std::fs::remove_dir_all(&tmp_dir).ok();
    Ok(())
}

/// 依赖 lcl 路径 → 包 stem（去重排序；A5b-3 宏插件递归链接用）
fn dep_stems(paths: &[PathBuf]) -> Vec<String> {
    let mut out: Vec<String> = paths.iter()
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Install a .lcl package: read LIR, emit LLVM IR, build executable or library.
pub fn install_package(lcl_path: &str, target_type: Option<&str>) -> Result<()> {
    let (_, _, lir_binary, target_types) =
        crate::package::load_package(lcl_path).map_err(|e| Error::Compile(format!("failed to load package: {}", e)))?;

    let lir_program = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| Error::Compile(format!("failed to deserialize LIR: {}", e)))?;

    let llvm_ir = crate::lir::emit_program(&lir_program);

    let base_name = std::path::Path::new(lcl_path)
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy()
        .into_owned();

    let tt = target_type
        .and_then(|s| crate::package::TargetType::from_str(s))
        .or_else(|| target_types.first().copied())
        .ok_or_else(|| Error::Compile("no target type specified and none in package".into()))?;

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
