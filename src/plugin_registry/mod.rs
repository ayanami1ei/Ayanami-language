//! A5j-2：插件本地 registry、lockfile 与 SHA-256 完整性校验。
//!
//! 当前只支持本地目录 registry（`index.toml` + `.lcl`），不做网络下载；
//! 版本选择支持精确版本与最高版本，范围求解留给后续。

pub mod lockfile;
mod registry;
pub mod sha256;

pub use lockfile::{LockEntry, Lockfile};
pub use registry::{RegistryIndex, RegistryVersion, ResolvedPlugin};
pub use sha256::sha256_hex;

use crate::error::{Error, Result};
use crate::package::parse_version;
use std::path::PathBuf;

/// 插件 home：`$AYANAMI_PLUGIN_HOME`，否则 `~/.ayanami`。
pub fn plugin_home() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os("AYANAMI_PLUGIN_HOME") {
        return Ok(PathBuf::from(p));
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| Error::Package("cannot determine plugin home; set AYANAMI_PLUGIN_HOME".into()))?;
    Ok(PathBuf::from(home).join(".ayanami"))
}

/// 默认本地 registry：`<plugin-home>/registry`。
pub fn default_registry_root() -> Result<PathBuf> {
    Ok(plugin_home()?.join("registry"))
}

/// 缓存路径：`<plugin-home>/cache/plugins/<name>/<version>.lcl`。
pub fn cache_path(name: &str, version: &str) -> Result<PathBuf> {
    if !safe_component(name) || !safe_component(version) {
        return Err(Error::Package(format!(
            "invalid plugin cache component '{}@{}'",
            name, version
        )));
    }
    Ok(plugin_home()?
        .join("cache")
        .join("plugins")
        .join(name)
        .join(format!("{}.lcl", version)))
}

/// 规范化 semver 为 `major.minor.patch`；非法返回 None。
pub fn normalize_version(value: &str) -> Option<String> {
    parse_version(value).map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
}

/// 两个版本是否 semver 数值相等（`1.2` == `1.2.0`）。
pub fn versions_equal(a: &str, b: &str) -> bool {
    match (parse_version(a), parse_version(b)) {
        (Some(a), Some(b)) => a == b,
        _ => a == b,
    }
}

fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        && value != "."
        && value != ".."
}

/// 从 registry 或 cache 复制插件到 `dst`；父目录自动创建。
pub fn copy_plugin(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Package(format!("failed to create '{}': {}", parent.display(), e)))?;
    }
    std::fs::copy(src, dst)
        .map_err(|e| Error::Package(format!("failed to copy '{}' to '{}': {}", src.display(), dst.display(), e)))?;
    Ok(())
}
