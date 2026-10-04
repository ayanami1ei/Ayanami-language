use super::*;
use super::deps::merge_symbols;

pub fn build_source_with_target(
    src_path: &str,
    _code: &str,
    out_dir: &str,
    target_override: Option<&str>,
) -> Result<()> {
    let out_path = Path::new(out_dir);
    std::fs::create_dir_all(out_path)
        .map_err(|e| Error::Compile(format!("failed to create output dir '{}': {}", out_dir, e)))?;

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

    let _output_path = match target {
        "executable" => {
            let exe_path = out_path.join(&*name);
            crate::driver::objects_to_exe_with_flags(
                &compiled.obj_paths,
                &compiled.link_flags,
                &exe_path,
            )
            .map_err(|e| Error::Compile(format!("link failed: {}", e)))?;
            exe_path
        }
        "static-lib" => {
            let lib_path = out_path.join(format!("lib{}.a", name));
            if compiled.obj_paths.len() == 1 {
                crate::driver::object_to_static_lib(&compiled.obj_paths[0], &lib_path)?;
            } else {
                let mut cmd = std::process::Command::new("ar");
                cmd.arg("rcs").arg(&lib_path);
                for o in &compiled.obj_paths {
                    cmd.arg(o);
                }
                let status = cmd.status().map_err(|e| Error::Compile(format!("failed to run ar: {}", e)))?;
                if !status.success() {
                    return Err(Error::Compile("ar failed".into()));
                }
            }
            lib_path
        }
        "dynamic-lib" => {
            let so_path = out_path.join(format!("lib{}.so", name));
            if compiled.obj_paths.len() == 1 {
                crate::driver::object_to_shared_lib(&compiled.obj_paths[0], &so_path)?;
            } else {
                crate::driver::objects_to_shared_lib(&compiled.obj_paths, &so_path)?;
            }
            so_path
        }
        _ => return Err(Error::Compile(format!("unknown target type: {}", target))),
    };


    let lcl_path = out_path.join(format!("{}.lcl", name));
    let mut pkg = crate::package::Package::new(name.to_string(), env!("CARGO_PKG_VERSION").into());
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
        .map_err(|e| Error::Compile(format!("package write failed: {}", e)))?;

    Ok(())
}

/// Run a built executable.  Looks in `build/` first, then cwd.
pub fn run_executable(exe_name: &str) -> Result<i32> {
    let build_path = format!("build/{}", exe_name);
    let exe_path = if std::path::Path::new(&build_path).exists() {
        build_path
    } else {
        exe_name.to_string()
    };
    let status = std::process::Command::new(&exe_path)
        .status()
        .map_err(|e| Error::Compile(format!("failed to run '{}': {}", exe_path, e)))?;
    Ok(status.code().unwrap_or(-1))
}
