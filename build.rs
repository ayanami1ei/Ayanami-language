//! 构建指纹（#177 后续）：把 git 提交短哈希（+ dirty 标记）编入二进制，
//! 供 `ayanami --version` 展示与 `ayanami update --check` 比较
//! ——同版本号的 respin（修复构建）也能被检测到并触发更新。

use std::process::Command;

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
}
