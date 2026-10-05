//! Runtime 选择（#84 ②）：`AYANAMI_RUNTIME` 环境变量 / `ayanami.toml [runtime] path`。
//!
//! 优先级：环境变量 > 项目配置 > 内置 `runtime.c`（返回 None）。
//! 路径支持 `.c`（gcc 现场编译）/ `.o` / `.a`（静态库）——统一作为 gcc 输入文件。
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// 解析自定义 runtime 路径；未指定时返回 `None`（使用内置 runtime.c）。
///
/// `src_path`：当前编译的源文件，用于向上查找 `ayanami.toml`。
/// `[runtime] path` 相对项目根目录（含 ayanami.toml 的目录）解析。
pub(crate) fn resolve_runtime(src_path: &Path) -> Result<Option<PathBuf>> {
    if let Ok(env_path) = std::env::var("AYANAMI_RUNTIME") {
        let env_path = env_path.trim();
        if !env_path.is_empty() {
            let p = PathBuf::from(env_path);
            if !p.exists() {
                return Err(Error::Driver(format!(
                    "AYANAMI_RUNTIME='{}' not found",
                    env_path
                )));
            }
            return Ok(Some(p));
        }
    }

    let start = src_path.parent().unwrap_or(Path::new("."));
    let mut dir = Some(start);
    while let Some(d) = dir {
        let cfg_path = d.join("ayanami.toml");
        if cfg_path.exists() {
            let content = std::fs::read_to_string(&cfg_path)
                .map_err(|e| Error::Driver(format!("failed to read '{}': {}", cfg_path.display(), e)))?;
            let cfg = crate::package::config::ProjectConfig::load(&content);
            if let Some(rel) = cfg.runtime {
                let p = d.join(&rel);
                if !p.exists() {
                    return Err(Error::Driver(format!(
                        "[runtime] path '{}' not found (from '{}')",
                        rel,
                        cfg_path.display()
                    )));
                }
                return Ok(Some(p));
            }
            return Ok(None);
        }
        dir = d.parent();
    }
    Ok(None)
}
