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

/// Link multiple object files + runtime → executable via `gcc`.
pub fn objects_to_exe(obj_paths: &[PathBuf], exe_path: impl AsRef<Path>) -> Result<(), String> {
    let runtime_c = find_runtime_c()?;
    let mut cmd = Command::new("gcc");
    cmd.arg("-no-pie");
    for o in obj_paths { cmd.arg(o); }
    cmd.arg(&runtime_c).arg("-o").arg(exe_path.as_ref());
    let status = cmd.status().map_err(|e| format!("failed to run gcc: {}", e))?;
    if !status.success() { return Err("gcc link failed".into()); }
    Ok(())
}

/// Single object file version (backward compat).
pub fn object_to_exe(obj_path: impl AsRef<Path>, exe_path: impl AsRef<Path>) -> Result<(), String> {
    objects_to_exe(&[obj_path.as_ref().to_path_buf()], exe_path)
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

/// Link object file → static library (.a) via `ar`.
pub fn object_to_static_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<(), String> {
    let status = Command::new("ar")
        .arg("rcs")
        .arg(lib_path.as_ref())
        .arg(obj_path.as_ref())
        .status()
        .map_err(|e| format!("failed to run ar: {}", e))?;

    if !status.success() {
        return Err("ar failed".into());
    }
    Ok(())
}

/// Link multiple object files → shared library (.so) via `gcc`.
pub fn objects_to_shared_lib(obj_paths: &[PathBuf], lib_path: impl AsRef<Path>) -> Result<(), String> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-shared").arg("-fPIC");
    for o in obj_paths { cmd.arg(o); }
    cmd.arg("-o").arg(lib_path.as_ref());
    let status = cmd.status().map_err(|e| format!("failed to run gcc: {}", e))?;
    if !status.success() { return Err("gcc -shared failed".into()); }
    Ok(())
}

/// Link single object file → shared library (.so) via `gcc`.
pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<(), String> {
    let status = Command::new("gcc")
        .arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(lib_path.as_ref())
        .arg(obj_path.as_ref())
        .status()
        .map_err(|e| format!("failed to run gcc: {}", e))?;

    if !status.success() {
        return Err("gcc -shared failed".into());
    }
    Ok(())
}

/// Compile LLVM IR → library (.a or .so) in one step.
pub fn ir_to_library(llvm_ir: &str, lib_path: impl AsRef<Path>, lib_type: &str) -> Result<(), String> {
    let obj_path = {
        let mut p = lib_path.as_ref().to_path_buf();
        p.set_extension("o");
        p
    };

    ir_to_object(llvm_ir, &obj_path)?;

    match lib_type {
        "static-lib" => object_to_static_lib(&obj_path, lib_path)?,
        "dynamic-lib" => object_to_shared_lib(&obj_path, lib_path)?,
        _ => return Err(format!("unknown library type: {}", lib_type)),
    }

    let _ = std::fs::remove_file(&obj_path);
    Ok(())
}

/// Compile LLVM IR → .o (keeps the .o file for later linking).
pub fn ir_to_object_keep(llvm_ir: &str, obj_path: impl AsRef<Path>) -> Result<(), String> {
    ir_to_object(llvm_ir, &obj_path)
}

/// Compile LLVM IR → executable in one step, keeping .o.
pub fn ir_to_executable(llvm_ir: &str, exe_path: impl AsRef<Path>) -> Result<(), String> {
    let obj_path = {
        let mut p = exe_path.as_ref().to_path_buf();
        p.set_extension("o");
        p
    };

    ir_to_object(llvm_ir, &obj_path)?;
    objects_to_exe(&[obj_path], exe_path)?;

    Ok(())
}
