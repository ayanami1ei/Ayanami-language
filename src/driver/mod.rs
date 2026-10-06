/// Compilation driver: converts LLVM IR text → object → executable.
///
/// Pipeline:
/// 1. `llc` compiles `.ll` → `.o` (LLVM static compiler)
/// 2. `gcc` links `.o` with `src/runtime.c` → executable (`-no-pie`)
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

mod runtime;
pub(crate) use runtime::resolve_runtime;

/// Find `llc` — check next to the ayanami binary first, then PATH.
pub(crate) fn find_llc() -> Result<(PathBuf, PathBuf)> {
    let exe = std::env::current_exe().ok();
    if let Some(exe_path) = exe {
        if let Some(exe_dir) = exe_path.parent() {
            let local = exe_dir.join("llc");
            if local.exists() {
                return Ok((local, exe_dir.to_path_buf()));
            }
        }
    }
    // Fall back to system PATH
    Ok((PathBuf::from("llc"), PathBuf::new()))
}

/// Find `opt` — check next to the ayanami binary first, then PATH.
///
/// `local_only` 用于 bundled llc 场景：系统 `opt` 可能与 bundled LLVM 版本不一致
/// （例如 opt 22 产出的属性 bundled llc 21 不认识），此时只用同目录 `opt`。
/// Optimization is optional: when `opt` is missing (or `AYANAMI_OPT=0`),
/// `ir_to_object` falls back to compiling the unoptimized IR with llc.
fn find_opt(local_only: bool) -> Option<(PathBuf, PathBuf)> {
    if std::env::var("AYANAMI_OPT").map(|v| v == "0").unwrap_or(false) {
        return None;
    }
    let exe = std::env::current_exe().ok();
    if let Some(exe_path) = exe {
        if let Some(exe_dir) = exe_path.parent() {
            let local = exe_dir.join("opt");
            if local.exists() {
                return Some((local, exe_dir.to_path_buf()));
            }
        }
    }
    if local_only {
        return None;
    }
    Some((PathBuf::from("opt"), PathBuf::new()))
}

/// Compile LLVM IR text → object file (.o) via `opt -O2` (optional) + `llc`.
pub fn ir_to_object(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()> {
    let obj = obj_path.as_ref();

    // Write LLVM IR to temp file
    let mut ll_path = obj.to_path_buf();
    ll_path.set_extension("ll");
    std::fs::write(&ll_path, llvm_ir)
        .map_err(|e| Error::Driver(format!("failed to write .ll file: {}", e)))?;

    let (llc_path, llc_dir) = find_llc()?;
    let llc_is_local = !llc_dir.as_os_str().is_empty();

    // M-opt.1：模式档位——release 跑 opt -O3（标注/所有权属性才能跨调用生效）；
    // debug 跳过中端优化（编译快、便于调试；非法 IR 由 llc 校验兜底）。
    // bundled llc 场景只用同目录 opt（避免版本不匹配）；缺失/失败回退未优化 IR。
    let release = crate::hir::contracts::is_release();
    let mut llc_input = ll_path.clone();
    if release {
        if let Some((opt_path, opt_dir)) = find_opt(llc_is_local) {
            let mut opt_ll = obj.to_path_buf();
            opt_ll.set_extension("opt.ll");
            let mut cmd = Command::new(&opt_path);
            cmd.arg("-O3").arg("-S").arg(&ll_path).arg("-o").arg(&opt_ll);
            if !opt_dir.as_os_str().is_empty() {
                cmd.env("LD_LIBRARY_PATH", opt_dir.to_string_lossy().as_ref());
            }
            if let Ok(status) = cmd.status() {
                if status.success() {
                    llc_input = opt_ll;
                }
            }
        }
    }

    let mut cmd = Command::new(&llc_path);
    cmd.arg("-filetype=obj");
    if release {
        cmd.arg("-O3");
    }
    cmd.arg("-o").arg(obj)
        .arg(&llc_input);

    // If llc is bundled, set LD_LIBRARY_PATH so it can find its .so
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }

    let status = cmd.status()
        .map_err(|e| Error::Driver(format!("failed to run llc: {} (install llvm or place llc next to ayanami)", e)))?;

    if !status.success() {
        return Err(Error::Driver("llc failed".into()));
    }

    // Keep .ll file for debugging
    // let _ = std::fs::remove_file(&ll_path);
    Ok(())
}

/// Link multiple object files + runtime → executable via `gcc`.
pub fn objects_to_exe(obj_paths: &[PathBuf], exe_path: impl AsRef<Path>) -> Result<()> {
    objects_to_exe_with_flags(obj_paths, &[], exe_path)
}

/// Link with extra flags (for dynamic lib linking).
pub fn objects_to_exe_with_flags(obj_paths: &[PathBuf], extra_flags: &[String], exe_path: impl AsRef<Path>) -> Result<()> {
    objects_to_exe_with_runtime(obj_paths, extra_flags, exe_path, None)
}

/// Link with an explicit runtime input（#84 ②）。
///
/// `runtime = None` 时使用内置 `runtime.c`；`Some(path)` 支持 `.c`（gcc 现场编译）、
/// `.o` / `.a`（静态库，直接作为输入）——自定义 runtime 取代内置 runtime.c。
pub fn objects_to_exe_with_runtime(
    obj_paths: &[PathBuf],
    extra_flags: &[String],
    exe_path: impl AsRef<Path>,
    runtime: Option<&Path>,
) -> Result<()> {
    let runtime_path: PathBuf = match runtime {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(find_runtime_c()?),
    };
    if !runtime_path.exists() {
        return Err(Error::Driver(format!(
            "runtime '{}' not found",
            runtime_path.display()
        )));
    }
    let mut cmd = Command::new("gcc");
    cmd.arg("-no-pie");
    for o in obj_paths { cmd.arg(o); }
    for f in extra_flags { cmd.arg(f); }
    cmd.arg(&runtime_path).arg("-lm").arg("-o").arg(exe_path.as_ref());
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run gcc: {}", e)))?;
    if !status.success() { return Err(Error::Driver("gcc link failed".into())); }
    Ok(())
}

/// Single object file version (backward compat).
pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<()> {
    objects_to_exe(&[obj_path.as_ref().to_path_buf()], exe_path)
}

/// Locate the runtime C file.
/// Searches: exe dir → Cargo.toml parent → cwd parent.
pub(crate) fn find_runtime_c() -> Result<String> {
    let exe = std::env::current_exe().ok();
    // 开发态（cargo 的 target/ 下）：优先仓库 src/runtime.c，避免 target/debug 里的陈旧副本
    if let Some(exe_path) = &exe {
        let in_target = exe_path.components().any(|c| c.as_os_str() == "target");
        if in_target {
            if let Some(exe_dir) = exe_path.parent() {
                let mut dir = Some(exe_dir);
                while let Some(d) = dir {
                    if d.join("Cargo.toml").exists() && d.join("src").join("runtime.c").exists() {
                        return Ok(d.join("src").join("runtime.c").to_string_lossy().into_owned());
                    }
                    dir = d.parent();
                }
            }
        }
    }
    // 安装态：同目录 runtime.c
    if let Some(exe_path) = exe {
        if let Some(exe_dir) = exe_path.parent() {
            let rt = exe_dir.join("runtime.c");
            if rt.exists() {
                return Ok(rt.to_string_lossy().into_owned());
            }
        }
    }
    // Second, search upward for Cargo.toml (for development)
    let cwd = std::env::current_dir().map_err(|e| Error::Driver(format!("failed to get cwd: {}", e)))?;
    let mut dir = Some(cwd.as_path());
    while let Some(d) = dir {
        if d.join("Cargo.toml").exists() {
            let rt = d.join("src").join("runtime.c");
            if rt.exists() {
                return Ok(rt.to_string_lossy().into_owned());
            }
            return Err(Error::Driver("src/runtime.c not found next to Cargo.toml".into()));
        }
        dir = d.parent();
    }
    Err(Error::Driver("could not locate runtime.c (place next to the ayanami binary)".into()))
}

/// Link object file → static library (.a) via `ar`.
pub fn object_to_static_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()> {
    let status = Command::new("ar")
        .arg("rcs")
        .arg(lib_path.as_ref())
        .arg(obj_path.as_ref())
        .status()
        .map_err(|e| Error::Driver(format!("failed to run ar: {}", e)))?;

    if !status.success() {
        return Err(Error::Driver("ar failed".into()));
    }
    Ok(())
}

/// Link multiple object files → shared library (.so) via `gcc`.
pub fn objects_to_shared_lib(obj_paths: &[PathBuf], lib_path: impl AsRef<Path>) -> Result<()> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-shared").arg("-fPIC");
    for o in obj_paths { cmd.arg(o); }
    cmd.arg("-o").arg(lib_path.as_ref());
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run gcc: {}", e)))?;
    if !status.success() { return Err(Error::Driver("gcc -shared failed".into())); }
    Ok(())
}

/// Link single object file → shared library (.so) via `gcc`.
pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()> {
    let status = Command::new("gcc")
        .arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(lib_path.as_ref())
        .arg(obj_path.as_ref())
        .status()
        .map_err(|e| Error::Driver(format!("failed to run gcc: {}", e)))?;

    if !status.success() {
        return Err(Error::Driver("gcc -shared failed".into()));
    }
    Ok(())
}

/// Compile LLVM IR → library (.a or .so) in one step.
pub fn ir_to_library(llvm_ir: &str, lib_path: impl AsRef<Path>, lib_type: &str) -> Result<()> {
    let obj_path = {
        let mut p = lib_path.as_ref().to_path_buf();
        p.set_extension("o");
        p
    };

    ir_to_object(llvm_ir, &obj_path)?;

    match lib_type {
        "static-lib" => object_to_static_lib(&obj_path, lib_path)?,
        "dynamic-lib" => object_to_shared_lib(&obj_path, lib_path)?,
        _ => return Err(Error::Driver(format!("unknown library type: {}", lib_type))),
    }

    let _ = std::fs::remove_file(&obj_path);
    Ok(())
}

/// Compile LLVM IR → .o (keeps the .o file for later linking).
pub fn ir_to_object_keep(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<()> {
    ir_to_object(llvm_ir, &obj_path)
}

/// Compile LLVM IR → executable in one step, keeping .o.
pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<()> {
    let obj_path = {
        let mut p = exe_path.as_ref().to_path_buf();
        p.set_extension("o");
        p
    };

    ir_to_object(llvm_ir, &obj_path)?;
    objects_to_exe(&[obj_path], exe_path)?;

    Ok(())
}
