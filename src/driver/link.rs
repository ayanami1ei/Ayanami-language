//! 链接：可执行文件 / 静态库 / 共享库（含 runtime.c 查找）。
use super::*;

/// Link multiple object files → executable via `gcc`.
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
    if cfg!(target_os = "linux") { cmd.arg("-Wl,--allow-multiple-definition"); }
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
    if cfg!(target_os = "linux") { cmd.arg("-Wl,--allow-multiple-definition"); }
    for o in obj_paths { cmd.arg(o); }
    cmd.arg("-o").arg(lib_path.as_ref());
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run gcc: {}", e)))?;
    if !status.success() { return Err(Error::Driver("gcc -shared failed".into())); }
    Ok(())
}

/// Link single object file → shared library (.so) via `gcc`.
pub fn object_to_shared_lib(obj_path: impl AsRef<Path>, lib_path: impl AsRef<Path>) -> Result<()> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-shared").arg("-fPIC");
    if cfg!(target_os = "linux") { cmd.arg("-Wl,--allow-multiple-definition"); }
    let status = cmd
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
