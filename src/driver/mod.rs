/// Compilation driver: converts LLVM IR text → object → executable.
///
/// Pipeline:
/// 1. `llc` compiles `.ll` → `.o` (LLVM static compiler)
/// 2. `gcc` links `.o` with `src/runtime.c` → executable (`-no-pie`)
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

mod link;
mod lto;
mod runtime;
pub(crate) use runtime::resolve_runtime;
pub use link::*;
pub(crate) use link::find_runtime_c;
pub use lto::{is_lto, link_modules_lto, set_lto};
pub use crate::target::{is_bare_metal_triple, is_cross_target, set_target_triple, target_triple};

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
pub(super) fn find_opt(local_only: bool) -> Option<(PathBuf, PathBuf)> {
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

/// 依赖 .o 缓存键：编译模式参与哈希，避免 debug/release/LTO 产物互相复用。
pub fn build_mode_key() -> String {
    format!(
        "release={};lto={};opt={};fp={}",
        crate::hir::contracts::is_release(),
        is_lto(),
        std::env::var("AYANAMI_OPT").unwrap_or_default(),
        env!("AYANAMI_COMPILER_FP"),
    )
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
    let triple = crate::target::target_triple();
    if let Some(t) = &triple {
        cmd.arg(format!("-mtriple={}", t));
    }
    if crate::target::freestanding()
        || triple.as_deref().map_or(false, crate::target::is_bare_metal_triple)
    {
        cmd.arg("-relocation-model=static");
    }
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

    // freestanding 只保留 .o（LTO 需要 .ll 时保留）
    if crate::target::freestanding() && !is_lto() {
        let _ = std::fs::remove_file(&ll_path);
        let mut opt_ll = obj.to_path_buf();
        opt_ll.set_extension("opt.ll");
        let _ = std::fs::remove_file(&opt_ll);
    }

    // Keep .ll file for debugging
    // let _ = std::fs::remove_file(&ll_path);
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
