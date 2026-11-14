//! registry 模式安装：index 解析 + cache/lockfile 选择 + 复制。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ayanami::package::config::ProjectConfig;
use ayanami::package::load_checked_manifest;
use ayanami::plugin_registry::{
    cache_path, copy_plugin, default_registry_root, normalize_version, versions_equal,
    LockEntry, Lockfile, RegistryIndex,
};

use super::{current_project, ensure_dependency, fail, sha256_file, validate_alias};

pub(super) fn registry_install(target: &str, alias_arg: Option<&str>, update: bool) {
    let (name, requested) = split_target(target);
    validate_alias(name);
    let alias = alias_arg.unwrap_or(name);
    validate_alias(alias);

    let project = current_project();
    let config_path = project.join("ayanami.toml");
    let toml = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => fail(format!("failed to read '{}': {}", config_path.display(), e)),
    };
    let config = ProjectConfig::load(&toml);
    let lock_path = project.join("ayanami.lock");
    let mut lock = match Lockfile::load(&lock_path) {
        Ok(l) => l,
        Err(e) => fail(e),
    };
    let expected_rel = format!("plugins/{}.lcl", alias);
    let locked = lock.get(name).cloned();

    let mut locked_version = None;
    let mut locked_sha = None;
    if let Some(entry) = &locked {
        if !update {
            if let Some(req) = requested {
                if !version_matches(&entry.version, req) {
                    fail(format!(
                        "plugin '{}' is locked to {}; requested {} (use --update)",
                        name, entry.version, req
                    ));
                }
            }
            if entry.path != expected_rel {
                fail(format!(
                    "plugin '{}' is locked at '{}'; use --update to install as '{}'",
                    name, entry.path, expected_rel
                ));
            }
            let dst = project.join(&entry.path);
            if dst.is_file() {
                if let Ok(sha) = sha256_file(&dst) {
                    if sha == entry.sha256 && manifest_matches(&dst, name, &entry.version) {
                        ensure_dependency(&project, alias, &expected_rel);
                        println!("using locked plugin: {} {} ({})", name, entry.version, entry.path);
                        return;
                    }
                }
            }
            if let Some(src) = cache_for(name, &entry.version) {
                if cache_valid(&src, Some(&entry.sha256)) {
                    if let Err(e) = copy_plugin(&src, &dst) {
                        fail(e);
                    }
                    if !manifest_matches(&dst, name, &entry.version) {
                        fail(format!("cached plugin '{}' {} has an invalid manifest", name, entry.version));
                    }
                    ensure_dependency(&project, alias, &expected_rel);
                    println!("restored locked plugin from cache: {} {}", name, entry.version);
                    return;
                }
            }
            locked_version = Some(entry.version.clone());
            locked_sha = Some(entry.sha256.clone());
        }
    }

    let want = locked_version.as_deref().or(requested);
    let mut source: Option<(PathBuf, String)> = None;
    if let Some(version) = want {
        if let Some(cache) = cache_for(name, version) {
            if cache_valid(&cache, locked_sha.as_deref()) {
                let canonical = normalize_version(version).unwrap_or_else(|| version.to_string());
                source = Some((cache, canonical));
            }
        }
    }
    if source.is_none() {
        let root = registry_root(&project, &config);
        let index = match RegistryIndex::load(&root) {
            Ok(i) => i,
            Err(e) => fail(format!("{} (registry: {})", e, root.display())),
        };
        let resolved = match index.resolve(&root, name, want) {
            Ok(r) => r,
            Err(e) => fail(e),
        };
        source = Some((resolved.path, resolved.version));
    }
    let (src, resolved_version) = source.unwrap_or_else(|| fail("no plugin source resolved"));

    let src_s = src.to_string_lossy();
    let manifest = match load_checked_manifest(&src_s) {
        Ok(Some(m)) => m,
        Ok(None) => fail(format!("'{}' is not a plugin (missing [plugin] manifest)", src.display())),
        Err(e) => fail(e),
    };
    if manifest.name != name {
        fail(format!(
            "registry plugin '{}' contains manifest name '{}'",
            name, manifest.name
        ));
    }
    if !versions_equal(&manifest.version, &resolved_version) {
        fail(format!(
            "registry plugin '{}' index version {} does not match manifest version {}",
            name, resolved_version, manifest.version
        ));
    }
    if let Some(expected) = &locked_sha {
        let actual = sha256_file(&src).unwrap_or_else(|e| fail(e));
        if actual != *expected {
            fail(format!(
                "plugin '{}' artifact checksum changed since lockfile; use --update",
                name
            ));
        }
    }

    let dst = project.join(&expected_rel);
    if let Err(e) = copy_plugin(&src, &dst) {
        fail(e);
    }
    let actual_sha = sha256_file(&dst).unwrap_or_else(|e| fail(e));
    let actual_version = manifest.version.clone();
    let cache_version = normalize_version(&actual_version).unwrap_or_else(|| actual_version.clone());
    if let Ok(cache) = cache_path(name, &cache_version) {
        if src != cache {
            match copy_plugin(&src, &cache) {
                Ok(()) => {
                    let cache_sha = sha256_file(&cache).unwrap_or_else(|e| fail(e));
                    if let Err(e) = fs::write(cache.with_extension("sha256"), format!("{}\n", cache_sha)) {
                        eprintln!("warning: failed to write plugin cache checksum: {}", e);
                    }
                }
                Err(e) => eprintln!("warning: failed to populate plugin cache: {}", e),
            }
        }
    }

    ensure_dependency(&project, alias, &expected_rel);
    lock.insert(
        name.to_string(),
        LockEntry {
            version: actual_version.clone(),
            path: expected_rel.clone(),
            sha256: actual_sha,
        },
    );
    if let Err(e) = lock.save(&lock_path) {
        fail(e);
    }
    println!("installed plugin from registry: {} {} -> {}", name, actual_version, expected_rel);
}

fn split_target(target: &str) -> (&str, Option<&str>) {
    let (name, requested) = match target.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (target, None),
    };
    if name.is_empty() {
        fail("plugin name must not be empty");
    }
    if requested == Some("") {
        fail("plugin version after '@' must not be empty");
    }
    (name, requested)
}

fn version_matches(version: &str, requested: &str) -> bool {
    requested == "*" || versions_equal(version, requested)
}

fn cache_for(name: &str, version: &str) -> Option<PathBuf> {
    let canonical = normalize_version(version)?;
    cache_path(name, &canonical).ok().filter(|p| p.is_file())
}

fn cache_valid(cache: &Path, expected: Option<&str>) -> bool {
    let Ok(actual) = sha256_file(cache) else { return false };
    if let Some(expected) = expected {
        return actual == expected;
    }
    let meta = cache.with_extension("sha256");
    match fs::read_to_string(meta) {
        Ok(expected) => expected.trim() == actual,
        Err(_) => false,
    }
}

fn manifest_matches(path: &Path, name: &str, version: &str) -> bool {
    let Ok(Some(manifest)) = load_checked_manifest(&path.to_string_lossy()) else { return false };
    manifest.name == name && versions_equal(&manifest.version, version)
}

fn registry_root(project: &Path, config: &ProjectConfig) -> PathBuf {
    if let Some(root) = env::var_os("AYANAMI_PLUGIN_REGISTRY") {
        return PathBuf::from(root);
    }
    if let Some(root) = &config.plugin_registry {
        return project.join(root);
    }
    default_registry_root().unwrap_or_else(|e| fail(e))
}
