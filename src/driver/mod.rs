/// Compilation driver: converts LLVM IR text → object → executable.
///
/// Pipeline:
/// 1. `llc` compiles `.ll` → `.o` (LLVM static compiler)
/// 2. `gcc` links `.o` with `src/runtime.c` → executable (`-no-pie`)
use std::path::{Path, PathBuf};
use std::process::Command;

/// Find `llc` — check next to the ayanami binary first, then PATH.
fn find_llc() -> Result<(PathBuf, PathBuf), String> {
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

/// Compile LLVM IR text → object file (.o) via `llc`.
pub fn ir_to_object(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<(), String> {
    let obj = obj_path.as_ref();

    // Write LLVM IR to temp file
    let mut ll_path = obj.to_path_buf();
    ll_path.set_extension("ll");
    std::fs::write(&ll_path, llvm_ir)
        .map_err(|e| format!("failed to write .ll file: {}", e))?;

    let (llc_path, llc_dir) = find_llc()?;
    let mut cmd = Command::new(&llc_path);
    cmd.arg("-filetype=obj")
        .arg("-o").arg(obj)
        .arg(&ll_path);

    // If llc is bundled, set LD_LIBRARY_PATH so it can find its .so
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }

    let status = cmd.status()
        .map_err(|e| format!("failed to run llc: {} (install llvm or place llc next to ayanami)", e))?;

    if !status.success() {
        return Err("llc failed".into());
    }

    // Clean up temp .ll file
    let _ = std::fs::remove_file(&ll_path);
    Ok(())
}

/// Link object file + runtime → executable via `gcc`.
/// Looks for `src/runtime.c` relative to the crate root.
pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<(), String> {
    // Find runtime.c relative to the project root
    let runtime_c = find_runtime_c()?;

    let status = Command::new("gcc")
        .arg("-no-pie")
        .arg(obj_path.as_ref())
        .arg(&runtime_c)
        .arg("-o")
        .arg(exe_path.as_ref())
        .status()
        .map_err(|e| format!("failed to run gcc: {}", e))?;

    if !status.success() {
        return Err("gcc link failed".into());
    }
    Ok(())
}

/// Locate the runtime C file.
/// Searches: exe dir → Cargo.toml parent → cwd parent.
fn find_runtime_c() -> Result<String, String> {
    // First, try next to the executable (for release builds in build/)
    let exe = std::env::current_exe().ok();
    if let Some(exe_path) = exe {
        if let Some(exe_dir) = exe_path.parent() {
            let rt = exe_dir.join("runtime.c");
            if rt.exists() {
                return Ok(rt.to_string_lossy().into_owned());
            }
        }
    }
    // Second, search upward for Cargo.toml (for development)
    let cwd = std::env::current_dir().map_err(|e| format!("failed to get cwd: {}", e))?;
    let mut dir = Some(cwd.as_path());
    while let Some(d) = dir {
        if d.join("Cargo.toml").exists() {
            let rt = d.join("src").join("runtime.c");
            if rt.exists() {
                return Ok(rt.to_string_lossy().into_owned());
            }
            return Err("src/runtime.c not found next to Cargo.toml".into());
        }
        dir = d.parent();
    }
    Err("could not locate runtime.c (place next to the ayanami binary)".into())
}

/// Compile LLVM IR → executable in one step.
pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<(), String> {
    let obj_path = {
        let mut p = exe_path.as_ref().to_path_buf();
        p.set_extension("o");
        p
    };

    ir_to_object(llvm_ir, &obj_path)?;
    object_to_exe(&obj_path, exe_path)?;

    // Clean up .o
    let _ = std::fs::remove_file(&obj_path);
    Ok(())
}
