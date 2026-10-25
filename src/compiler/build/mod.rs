use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::ast::*;

use super::check::check_hir_returns;
use super::import::{load_config_for_file, resolve_import_path};
use super::CompiledFile;

/// Compile a .aya source file with recursive import resolution.

mod compile;
mod deps;
mod lir;
mod package;
mod passes;
mod target;

pub use target::build_source_with_target;
pub use compile::compile_file;
pub use package::{install_package, package_source};
pub use target::run_executable;

pub fn build_source(src_path: &str, code: &str) -> Result<()> {
    build_source_to(src_path, code, "build")
}

/// Build with explicit output directory, using recursive import compilation.
pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<()> {
    build_source_with_target(src_path, _code, out_dir, None)
}

/// #35 增量键：源码内容 + 编译器指纹 + 构建模式 + 编译期环境 + 源祖先目录 .aya/.lcl/.c 最新 mtime。
pub(crate) fn cache_key(src_path: &str, code: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut h);
    env!("AYANAMI_COMPILER_FP").hash(&mut h);
    crate::driver::build_mode_key().hash(&mut h);
    // 影响代码生成的编译期环境/工作目录（cfg feature、自定义 runtime 等）
    for k in ["AYANAMI_FEATURES", "AYANAMI_RUNTIME", "AYANAMI_CHECKS", "AYANAMI_LTO", "AYANAMI_BUILD_ID"] {
        std::env::var(k).unwrap_or_default().hash(&mut h);
    }
    std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned()).hash(&mut h);
    // 安装目录 std/*.lcl 的内容纳入 key（std 测试脚本每次重装但内容可能不变）
    if let Some(std_dir) = crate::compiler::import::find_std_dir() {
        let mut files: Vec<_> = std::fs::read_dir(&std_dir).ok().into_iter().flatten().flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |x| x == "lcl"))
            .collect();
        files.sort();
        for p in files {
            p.to_string_lossy().hash(&mut h);
            if let Ok(data) = std::fs::read(&p) { data.hash(&mut h); }
        }
    }
    // 源祖先目录中的 .aya/.lcl/.c 按内容纳入（mtime 会因 std 重装而变，内容不变应命中）
    let start_dir = std::path::Path::new(src_path).parent()
        .unwrap_or(std::path::Path::new("."));
    let mut dir = Some(start_dir.to_path_buf());
    let mut first_dir = true;
    while let Some(d) = dir {
        if let Ok(rd) = std::fs::read_dir(&d) {
            let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            entries.sort();
            for p in entries {
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                // 祖先目录的 .lcl 可能是 std 源树重建产物（非确定性字节/未实际使用），
                // 只对源目录本地的 .lcl 计内容；安装目录 std/*.lcl 已单独计哈希
                if ext == "aya" || ext == "c" || (ext == "lcl" && first_dir) {
                    p.to_string_lossy().hash(&mut h);
                    if let Ok(data) = std::fs::read(&p) { data.hash(&mut h); }
                }
            }
        }
        first_dir = false;
        dir = d.parent().map(|p| p.to_path_buf());
    }
    format!("{:x}", h.finish())
}
