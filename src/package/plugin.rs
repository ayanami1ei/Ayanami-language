//! A5j-1：`.lcl` 插件清单解析、ABI/最低编译器兼容性校验与旧插件告警。


use crate::error::{Error, Result};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub(crate) const PLUGIN_ABI_VERSION: u32 = 8;
pub(crate) const PLUGIN_MANIFEST_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub schema: u32,
    pub name: String,
    pub version: String,
    pub abi: u32,
    pub min_compiler: String,
    pub built_with: String,
}

impl PluginManifest {
    pub fn new(name: String, version: String, min_compiler: String, built_with: String) -> Self {
        Self {
            schema: PLUGIN_MANIFEST_SCHEMA,
            name,
            version,
            abi: PLUGIN_ABI_VERSION,
            min_compiler,
            built_with,
        }
    }
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.split(&['-', '+']);
    let core = parts.next()?;
    let mut nums = core.split('.');
    let major = nums.next()?.parse().ok()?;
    let minor = nums.next().unwrap_or("0").parse().ok()?;
    let patch = nums.next().unwrap_or("0").parse().ok()?;
    if nums.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn version_at_least(current: &str, required: &str) -> bool {
    let Some((c_major, c_minor, c_patch)) = parse_version(current) else { return false; };
    let Some((r_major, r_minor, r_patch)) = parse_version(required) else { return false; };
    c_major > r_major || (c_major == r_major && (c_minor > r_minor || (c_minor == r_minor && c_patch >= r_patch)))
}

pub(crate) fn parse_manifest(body: &str, path: &str) -> Result<Option<PluginManifest>> {
    let mut schema = None;
    let mut name = None;
    let mut version = None;
    let mut abi = None;
    let mut min_compiler = None;
    let mut built_with = None;

    let mut in_plugin_section = false;
    let mut found_plugin = false;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line == "[plugin]" {
            in_plugin_section = true;
            found_plugin = true;
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_plugin_section = false;
            continue;
        }
        if !in_plugin_section {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(Error::Package(format!("plugin manifest in '{}': missing '=' in key-value pair", path)));
        };

        let key = key.trim();
        let value = value.trim();
        let value = if value.starts_with('"') && value.ends_with('"') {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        };

        match key {
            "manifest" => schema = Some(value.parse().map_err(|_| Error::Package(format!("plugin manifest in '{}': invalid manifest version", path)))?),
            "name" => name = Some(value),
            "version" => version = Some(value),
            "abi" => abi = Some(value.parse().map_err(|_| Error::Package(format!("plugin manifest in '{}': invalid ABI version", path)))?),
            "min-compiler" => min_compiler = Some(value),
            "built-with" => built_with = Some(value),
            _ => continue,
        }
    }

    if !found_plugin {
        return Ok(None);
    }

    let schema = schema.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'manifest' key", path)))?;
    let name = name.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'name' key", path)))?;
    let version = version.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'version' key", path)))?;
    let abi = abi.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'abi' key", path)))?;
    let min_compiler = min_compiler.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'min-compiler' key", path)))?;
    let built_with = built_with.ok_or_else(|| Error::Package(format!("plugin manifest in '{}': missing 'built-with' key", path)))?;

    if schema != PLUGIN_MANIFEST_SCHEMA {
        return Err(Error::Package(format!("plugin '{}' version {}: uses manifest schema {}, compiler supports {}", name, version, schema, PLUGIN_MANIFEST_SCHEMA)));
    }

    Ok(Some(PluginManifest {
        schema,
        name,
        version,
        abi,
        min_compiler,
        built_with,
    }))
}

pub(crate) fn load_manifest(path: &str) -> Result<Option<PluginManifest>> {
    let data = std::fs::read(path)
        .map_err(|e| Error::Package(format!("failed to read package '{}': {}", path, e)))?;
    if data.len() < 12 {
        return Ok(None);
    }
    let marker = b"===LIR===\n";
    let pos = data[12..]
        .windows(marker.len())
        .position(|w| w == marker)
        .map(|p| p + 12)
        .unwrap_or(data.len());
    let body = std::str::from_utf8(&data[12..pos])
        .map_err(|e| Error::Package(format!("invalid UTF-8 in package INI: {}", e)))?;
    parse_manifest(body, path)
}

pub(crate) fn validate_manifest(manifest: &PluginManifest, path: &str) -> Result<()> {
    if manifest.schema != PLUGIN_MANIFEST_SCHEMA {
        return Err(Error::Package(format!(
            "plugin '{}' {} uses manifest schema {}, compiler supports {} (at '{}')",
            manifest.name, manifest.version, manifest.schema, PLUGIN_MANIFEST_SCHEMA, path
        )));
    }
    if manifest.abi != PLUGIN_ABI_VERSION {
        return Err(Error::Package(format!(
            "plugin '{}' {} requires plugin ABI {}, compiler provides {}; rebuild the plugin with this compiler or install a compatible version (at '{}')",
            manifest.name,
            manifest.version,
            manifest.abi,
            PLUGIN_ABI_VERSION,
            path
        )));
    }

    if parse_version(&manifest.min_compiler).is_none() {
        return Err(Error::Package(format!(
            "plugin '{}' {} has invalid min-compiler '{}' (at '{}')",
            manifest.name, manifest.version, manifest.min_compiler, path
        )));
    }
    let current_version = env!("CARGO_PKG_VERSION");
    if !version_at_least(current_version, &manifest.min_compiler) {
        return Err(Error::Package(format!(
            "plugin '{}' {} requires ayanami >= {}, current {} (at '{}')",
            manifest.name,
            manifest.version,
            manifest.min_compiler,
            current_version,
            path
        )));
    }

    Ok(())
}

pub fn load_checked_manifest(path: &str) -> Result<Option<PluginManifest>> {
    let manifest = load_manifest(path)?;
    if let Some(ref m) = manifest {
        validate_manifest(m, path)?;
    }
    Ok(manifest)
}

static WARNED_PATHS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub(crate) fn warn_missing_manifest(path: &str) {
    let warned_paths = WARNED_PATHS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut paths = warned_paths.lock().unwrap();
    if paths.insert(path.to_string()) {
        let stem = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path);
        eprintln!("warning: plugin package '{}' has no [plugin] manifest (built before A5j-1); rebuild it with 'ayanami plugin package' for ABI/compiler checks", stem);
    }
}

#[cfg(test)]
mod tests;
