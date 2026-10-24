use super::*;
use super::deps::merge_symbols;

/// 预扫描源码 import 图：存在源码级依赖（可解析到 .aya 的模块）→ 多模块构建。
/// 多模块下每个模块编译为独立对象，非 pub 定义须 internal（同名私有函数强符号冲突）。
fn scan_source_deps(src_path: &Path) -> bool {
    use crate::compiler::import::resolve_import_path;
    let mut seen: HashSet<std::path::PathBuf> = HashSet::new();
    let mut stack: Vec<std::path::PathBuf> = vec![src_path.to_path_buf()];
    while let Some(p) = stack.pop() {
        if !seen.insert(p.clone()) { continue; }
        let Ok(code) = std::fs::read_to_string(&p) else { continue };
        let base = p.parent().unwrap_or(Path::new("."));
        for line in code.lines() {
            let t = line.trim();
            let Some(rest) = t.strip_prefix("import ") else { continue };
            let name = rest.trim().trim_end_matches(';').trim().trim_matches('"');
            if name.is_empty() { continue; }
            if let Some(cand) = resolve_import_path(name, base) {
                if cand.extension().map(|e| e == "aya").unwrap_or(false) {
                    return true;
                }
                stack.push(cand);
            }
        }
    }
    false
}

/// #35 粗粒度增量：可执行目标且 build/<stem> 与 stamp 匹配时直接复用。
pub fn build_source_with_target(
    src_path: &str,
    code: &str,
    out_dir: &str,
    target: Option<&str>,
) -> Result<()> {
    // 默认关闭：std 测试脚本每次重装 .lcl，需按安装目录内容建键后再默认开启
    let cacheable = target == Some("executable") && std::env::var("AYANAMI_NO_INCREMENTAL").is_err();
    let stem = std::path::Path::new(src_path).file_stem()
        .map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let exe = std::path::Path::new(out_dir).join(&stem);
    let stamp = std::path::Path::new(out_dir).join(format!("{}.stamp", stem));
    let key = crate::compiler::build::cache_key(src_path, code);
    if cacheable && exe.exists()
        && std::fs::read_to_string(&stamp).ok().as_deref() == Some(key.as_str())
    {
        return Ok(());
    }
    let r = build_source_with_target_inner(src_path, code, out_dir, target);
    if r.is_ok() && cacheable { let _ = std::fs::write(&stamp, &key); }
    r
}

fn build_source_with_target_inner(
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

    // #84 ②：runtime 选择（AYANAMI_RUNTIME > ayanami.toml [runtime] path > 内置 runtime.c）
    let runtime = crate::driver::resolve_runtime(src_path)?;

    // 多模块构建检测（源码级 import）：非 pub 定义须 internal，避免跨对象同名私有符号冲突
    crate::hir::contracts::set_multi_module(scan_source_deps(src_path));

    let compiled = compile_file(
        src_path,
        base_dir,
        out_path,
        &mut compiling,
        &mut cache,
        target_override,
        runtime.as_deref(),
    )?;

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

    // M-opt.9：LTO —— llvm-link 合并全部模块 IR，opt -O3 一次，跨对象内联
    let mut obj_paths = compiled.obj_paths.clone();
    if crate::driver::is_lto() {
        let modules: Vec<std::path::PathBuf> = obj_paths.iter()
            .map(|o| o.with_extension("ll"))
            .filter(|p| p.exists())
            .collect();
        if modules.len() > 1 {
            let lto_obj = out_path.join(format!("{}.lto.o", name));
            crate::driver::link_modules_lto(&modules, &lto_obj)
                .map_err(|e| Error::Compile(format!("LTO failed: {}", e)))?;
            obj_paths = vec![lto_obj];
        }
    }

    let _output_path = match target {
        "executable" => {
            let exe_path = out_path.join(&*name);
            crate::driver::objects_to_exe_with_runtime(
                &obj_paths,
                &compiled.link_flags,
                &exe_path,
                runtime.as_deref(),
            )
            .map_err(|e| Error::Compile(format!("link failed: {}", e)))?;
            exe_path
        }
        "static-lib" => {
            let lib_path = out_path.join(format!("lib{}.a", name));
            if obj_paths.len() == 1 {
                crate::driver::object_to_static_lib(&obj_paths[0], &lib_path)?;
            } else {
                let mut cmd = std::process::Command::new("ar");
                cmd.arg("rcs").arg(&lib_path);
                for o in &obj_paths {
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
            if obj_paths.len() == 1 {
                crate::driver::object_to_shared_lib(&obj_paths[0], &so_path)?;
            } else {
                crate::driver::objects_to_shared_lib(&obj_paths, &so_path)?;
            }
            so_path
        }
        _ => return Err(Error::Compile(format!("unknown target type: {}", target))),
    };


    let lcl_path = out_path.join(format!("{}.lcl", name));
    let mut pkg = crate::package::Package::new(name.to_string(), env!("CARGO_PKG_VERSION").into());
    pkg.lir_data = crate::lir::serialize::program_to_bytes(&compiled.lir_program);
    pkg.collect_symbols(&compiled.program.stmts);
    pkg.set_consts(&compiled.consts);
    pkg.set_statics(&compiled.statics, false);
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
