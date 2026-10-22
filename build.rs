//! 构建指纹（#177 后续）：把 git 提交短哈希（+ dirty 标记）编入二进制，
//! 供 `ayanami --version` 展示与 `ayanami update --check` 比较
//! ——同版本号的 respin（修复构建）也能被检测到并触发更新。

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 递归收集 `.rs` 源码（排序后哈希，保证确定性）。
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().map_or(false, |x| x == "rs") {
            out.push(p);
        }
    }
}

fn main() {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git").args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let head = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let id = if dirty { format!("{}-dirty", head) } else { head };
    println!("cargo:rustc-env=AYANAMI_BUILD_ID={}", id);

    // #35：编译器源码内容指纹 —— 参与构建缓存键（编译器语义变化时旧产物必须失效；
    // git 哈希 + dirty 标记在连续 dirty 构建间不变，不足以作缓存键）。
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    let mut files = Vec::new();
    collect_rs(Path::new("src"), &mut files);
    files.push(PathBuf::from("Cargo.toml"));
    files.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for f in &files {
        f.to_string_lossy().hash(&mut hasher);
        std::fs::read(f).unwrap_or_default().hash(&mut hasher);
    }
    println!("cargo:rustc-env=AYANAMI_COMPILER_FP={:x}", hasher.finish());
}
