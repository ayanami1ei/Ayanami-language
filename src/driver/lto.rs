//! M-opt.9：跨对象内联（LTO）——llvm-link 合并模块 IR → opt -O3 → llc -O3。
use super::*;

// ── M-opt.9：跨对象内联（LTO）────────────────────────────────────
/// LTO 开关（进程级；CLI `--lto` 或 `AYANAMI_LTO=1`）
static LTO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn set_lto(v: bool) { LTO.store(v, std::sync::atomic::Ordering::Relaxed); }
pub fn is_lto() -> bool { LTO.load(std::sync::atomic::Ordering::Relaxed) }

/// 查找 LLVM 工具（优先 ayanami 同目录，其次 PATH）
fn find_tool(name: &str, local_only: bool) -> Option<(PathBuf, PathBuf)> {
    let exe = std::env::current_exe().ok();
    if let Some(exe_path) = exe {
        if let Some(exe_dir) = exe_path.parent() {
            let local = exe_dir.join(name);
            if local.exists() {
                return Some((local, exe_dir.to_path_buf()));
            }
        }
    }
    if local_only {
        return None;
    }
    Some((PathBuf::from(name), PathBuf::new()))
}

/// M-opt.9：跨对象内联 —— `llvm-link` 合并各模块 .ll → `opt -O3` → `llc -O3`（单个 .o）。
/// 合并后 pub 函数与 std 热路径可跨模块内联；private/internal 符号由 llvm-link 去重/重命名。
pub fn link_modules_lto(modules: &[PathBuf], out_obj: &Path) -> Result<()> {
    let (llc_path, llc_dir) = find_llc()?;
    let llc_is_local = !llc_dir.as_os_str().is_empty();
    let (link, link_dir) = find_tool("llvm-link", llc_is_local)
        .ok_or_else(|| Error::Driver("llvm-link not found（LTO 需要 llvm-link）".into()))?;
    let (opt, _) = find_opt(llc_is_local)
        .ok_or_else(|| Error::Driver("opt not found（LTO 需要 opt）".into()))?;

    let merged = out_obj.with_extension("lto.ll");
    let merged_opt = out_obj.with_extension("lto.opt.ll");

    let mut cmd = Command::new(&link);
    cmd.arg("-S");
    for m in modules { cmd.arg(m); }
    cmd.arg("-o").arg(&merged);
    if !link_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", link_dir.to_string_lossy().as_ref());
    }
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run llvm-link: {}", e)))?;
    if !status.success() {
        return Err(Error::Driver("llvm-link failed".into()));
    }

    let mut cmd = Command::new(&opt);
    cmd.arg("-O3").arg("-S").arg(&merged).arg("-o").arg(&merged_opt);
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run opt: {}", e)))?;
    if !status.success() {
        return Err(Error::Driver("opt failed (LTO)".into()));
    }

    let mut cmd = Command::new(&llc_path);
    cmd.arg("-filetype=obj").arg("-O3").arg("-o").arg(out_obj).arg(&merged_opt);
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }
    let status = cmd.status().map_err(|e| Error::Driver(format!("failed to run llc: {}", e)))?;
    if !status.success() {
        return Err(Error::Driver("llc failed (LTO)".into()));
    }
    Ok(())
}

