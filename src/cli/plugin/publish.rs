//! A5j-2b：把校验过的 `.lcl` 发布到本地 registry 并更新 `index.toml`。

use std::env;
use std::fs;
use std::path::PathBuf;

use ayanami::package::config::ProjectConfig;
use ayanami::package::load_checked_manifest;
use ayanami::plugin_registry::{default_registry_root, normalize_version, RegistryIndex};

use super::install::fail;

pub(super) fn plugin_publish(args: &[String]) {
    let mut force = false;
    let mut registry_arg = None;
    let mut lcl = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--force" => force = true,
            "--registry" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    fail("usage: ayanami plugin publish <plugin.lcl> [--registry <dir>] [--force]");
                };
                registry_arg = Some(path.clone());
            }
            other => {
                if lcl.is_some() {
                    fail("plugin publish accepts exactly one .lcl file");
                }
                lcl = Some(other.to_string());
            }
        }
        i += 1;
    }
    let Some(lcl) = lcl else {
        fail("usage: ayanami plugin publish <plugin.lcl> [--registry <dir>] [--force]");
    };
    let src = PathBuf::from(&lcl);
    if src.extension().and_then(|e| e.to_str()) != Some("lcl") || !src.is_file() {
        fail("plugin publish requires an existing .lcl file");
    }
    let manifest = match load_checked_manifest(&src.to_string_lossy()) {
        Ok(Some(m)) => m,
        Ok(None) => fail(format!("'{}' has no [plugin] manifest", src.display())),
        Err(e) => fail(e),
    };
    let name = manifest.name.clone();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        fail(format!("invalid plugin name '{}'", name));
    }
    let Some(version) = normalize_version(&manifest.version) else {
        fail(format!("invalid plugin version '{}'", manifest.version));
    };
    let file_name = format!("{}-{}.lcl", name, version);
    let root = registry_root(registry_arg.as_deref());
    if let Err(e) = fs::create_dir_all(&root) {
        fail(format!("failed to create registry '{}': {}", root.display(), e));
    }
    let dst = root.join(&file_name);
    let bytes = match fs::read(&src) {
        Ok(b) => b,
        Err(e) => fail(format!("failed to read '{}': {}", src.display(), e)),
    };
    if dst.exists() {
        let existing = fs::read(&dst).unwrap_or_default();
        if existing != bytes && !force {
            fail(format!(
                "'{}' already exists with different content (use --force)",
                dst.display()
            ));
        }
    }
    if !dst.exists() || fs::read(&dst).unwrap_or_default() != bytes {
        if let Err(e) = fs::write(&dst, &bytes) {
            fail(format!("failed to write '{}': {}", dst.display(), e));
        }
    }

    let index_path = root.join("index.toml");
    let mut index = if index_path.is_file() {
        match RegistryIndex::load(&root) {
            Ok(i) => i,
            Err(e) => fail(e),
        }
    } else {
        RegistryIndex::default()
    };
    if let Err(e) = index.insert(&name, &version, &file_name, force) {
        fail(e);
    }
    if let Err(e) = index.save(&root) {
        fail(e);
    }
    println!("published plugin: {} {} -> {}", name, version, dst.display());
}

fn registry_root(explicit: Option<&str>) -> PathBuf {
    if let Some(path) = explicit {
        return PathBuf::from(path);
    }
    if let Some(path) = env::var_os("AYANAMI_PLUGIN_REGISTRY") {
        return PathBuf::from(path);
    }
    if let Ok(cwd) = env::current_dir() {
        if let Some((project, toml)) = crate::find_project(&cwd) {
            let config = ProjectConfig::load(&toml);
            if let Some(path) = config.plugin_registry {
                return project.join(path);
            }
        }
    }
    default_registry_root().unwrap_or_else(|e| fail(e))
}
