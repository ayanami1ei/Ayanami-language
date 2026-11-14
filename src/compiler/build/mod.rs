use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::ast::*;

use super::CompiledFile;
use super::check::check_hir_returns;
use super::import::{load_config_for_file, resolve_import_path};

/// Compile a .aya source file with recursive import resolution.
mod compile;
mod deps;
mod lir;
mod package;
mod passes;
mod target;

pub use compile::compile_file;
pub use package::{install_package, package_source, package_source_with_manifest};
pub use target::build_source_with_target;
pub use target::run_executable;

pub fn build_source(src_path: &str, code: &str) -> Result<()> {
    build_source_to(src_path, code, "build")
}

/// Build with explicit output directory, using recursive import compilation.
pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<()> {
    build_source_with_target(src_path, _code, out_dir, None)
}

/// 从源码 import 行提取模块名（支持 `import "x"` / `import "x" { ... }`）。
fn import_names(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in code.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("import ") else {
            continue;
        };
        let Some(q) = rest.find('"') else { continue };
        let after = &rest[q + 1..];
        let Some(end) = after.find('"') else { continue };
        let name = &after[..end];
        if !name.is_empty() {
            out.push(name.to_string());
        }
    }
    out
}

/// 递归哈希根源文件与 import 依赖（.aya 继续展开，.lcl/.c 只哈希内容）。
fn hash_import_closure(
    src: &std::path::Path,
    h: &mut std::collections::hash_map::DefaultHasher,
    seen: &mut HashSet<PathBuf>,
) {
    use std::hash::{Hash, Hasher};
    let canonical = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    if !seen.insert(canonical.clone()) {
        return;
    }
    canonical.to_string_lossy().hash(h);
    let Ok(code) = fs::read_to_string(&canonical) else {
        return;
    };
    code.hash(h);
    if canonical.extension().and_then(|e| e.to_str()) != Some("aya") {
        return;
    }
    let base = canonical.parent().unwrap_or(Path::new("."));
    for name in import_names(&code) {
        if let Some(dep) = resolve_import_path(&name, base) {
            hash_import_closure(&dep, h, seen);
        }
    }
    h.write_u8(0);
}

/// #35 增量键：源码内容 + 编译器指纹 + 构建模式 + 编译期环境 + 递归 import 依赖内容。
pub(crate) fn cache_key(src_path: &str, code: &str, target: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut h);
    env!("AYANAMI_COMPILER_FP").hash(&mut h);
    crate::driver::build_mode_key().hash(&mut h);
    // 目标类型/三元组/裸机模式都会改变产物语义，必须进入缓存键。
    target.hash(&mut h);
    crate::target::target_triple().hash(&mut h);
    crate::target::freestanding().hash(&mut h);
    // 影响代码生成的编译期环境/工作目录（cfg feature、自定义 runtime 等）
    for k in [
        "AYANAMI_FEATURES",
        "AYANAMI_RUNTIME",
        "AYANAMI_CHECKS",
        "AYANAMI_LTO",
        "AYANAMI_BUILD_ID",
        "AYANAMI_ASYNC_CFG",
    ] {
        std::env::var(k).unwrap_or_default().hash(&mut h);
    }
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
        .hash(&mut h);
    // 安装目录 std/*.lcl 的内容纳入 key（std 测试脚本每次重装但内容可能不变）
    if let Some(std_dir) = crate::compiler::import::find_std_dir() {
        let mut files: Vec<_> = std::fs::read_dir(&std_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |x| x == "lcl"))
            .collect();
        files.sort();
        for p in files {
            p.to_string_lossy().hash(&mut h);
            if let Ok(data) = std::fs::read(&p) {
                data.hash(&mut h);
            }
        }
    }
    // 只纳入根文件与递归 import 依赖，避免扫描整个祖先目录（/tmp 下无关 .aya 会导致误失效）。
    let mut seen = HashSet::new();
    hash_import_closure(std::path::Path::new(src_path), &mut h, &mut seen);
    format!("{:x}", h.finish())
}
