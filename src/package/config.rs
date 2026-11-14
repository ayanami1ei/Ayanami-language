use std::collections::HashMap;
use std::path::Path;

/// Project configuration from ayanami.toml.
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub default_target: Option<String>,
    /// 交叉/裸机目标三元组（`[build] arch`，如 aarch64-none-elf）
    pub arch: Option<String>,
    /// freestanding：只产出 .o，不链接 runtime.c/libc，不生成 PIE
    pub freestanding: bool,
    pub file_targets: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
    /// #84 ②：`[runtime] path`——自定义 runtime（.c/.a/.o），相对项目根
    pub runtime: Option<String>,
    /// M8：启用 feature 列表（`features = ["a", "b"]`，供 `#[cfg(feature = "a")]`）
    pub features: Vec<String>,
    /// A5j-2：本地插件 registry 目录（`[plugins] registry`），相对项目根
    pub plugin_registry: Option<String>,
}

impl ProjectConfig {
    pub fn load(toml_content: &str) -> Self {
        let mut name = String::new();
        let mut version = String::new();
        let mut default_target: Option<String> = None;
        let mut arch: Option<String> = None;
        let mut freestanding = false;
        let mut file_targets = HashMap::new();
        let mut in_build = false;
        let mut in_targets = false;
        let mut in_deps = false;
        let mut in_runtime = false;
        let mut in_plugins = false;
        let mut runtime: Option<String> = None;
        let mut plugin_registry: Option<String> = None;
        let mut dependencies = HashMap::new();
        let mut features: Vec<String> = Vec::new();

        for line in toml_content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_build = line.starts_with("[build]") || line.starts_with("[build.targets]");
                in_targets = line.starts_with("[build.targets]");
                in_deps = line.starts_with("[dependencies]");
                in_runtime = line.starts_with("[runtime]");
                in_plugins = line.starts_with("[plugins]");
                continue;
            }
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"');
                if in_targets {
                    file_targets.insert(k.trim_matches('"').to_string(), v.to_string());
                } else if in_deps {
                    dependencies.insert(k.trim_matches('"').to_string(), v.to_string());
                } else if in_runtime {
                    if k == "path" { runtime = Some(v.to_string()); }
                } else if in_plugins {
                    if k == "registry" { plugin_registry = Some(v.to_string()); }
                } else if in_build {
                    match k {
                        "target" => default_target = Some(v.to_string()),
                        "arch" => arch = Some(v.to_string()),
                        "freestanding" => freestanding = v == "true" || v == "1",
                        _ => {}
                    }
                } else {
                    match k {
                        "name" => name = v.to_string(),
                        "version" => version = v.to_string(),
                        "features" => {
                            features = v.trim_start_matches('[').trim_end_matches(']')
                                .split(',').map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty()).collect();
                        }
                        _ => {}
                    }
                }
            }
        }
        ProjectConfig { name, version, default_target, arch, freestanding, file_targets, dependencies, runtime, features, plugin_registry }
    }

    /// Resolve an import path using the [dependencies] aliases.
    /// If the path looks like a direct file path (has extension), returns it as-is.
    /// Otherwise, looks up the alias in [dependencies].
    pub fn resolve_import<'a>(&'a self, import_path: &str, base_dir: &Path) -> Option<String> {
        // If it looks like a file path (has extension), don't resolve
        if import_path.contains('.') {
            let full = base_dir.join(import_path);
            if full.exists() { return Some(full.to_string_lossy().into_owned()); }
            return None;
        }
        // Otherwise, look up in dependencies
        self.dependencies.get(import_path)
            .map(|p| {
                let full = base_dir.join(p);
                full.to_string_lossy().into_owned()
            })
    }

    /// Resolve the target type for a file.
    /// Priority: per-file config → project default → heuristic
    pub fn resolve_target(&self, file_path: &Path) -> &str {
        let file_str = file_path.to_string_lossy();
        if let Some(t) = self.file_targets.get(file_str.as_ref()) {
            return t;
        }
        if let Some(ref t) = self.default_target {
            return t;
        }
        // Heuristic
        if file_str.ends_with("main.aya") { "executable" } else { "static-lib" }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectConfig;

    #[test]
    fn parses_target_arch() {
        let cfg = ProjectConfig::load("[package]\nname = \"p\"\n[build]\ntarget = \"object\"\narch = \"aarch64-none-elf\"\nfreestanding = true\n");
        assert_eq!(cfg.default_target.as_deref(), Some("object"));
        assert_eq!(cfg.arch.as_deref(), Some("aarch64-none-elf"));
        assert!(cfg.freestanding);
    }
}
