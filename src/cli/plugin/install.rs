//! 插件安装/校验：本地 `.lcl` 路径模式 + registry 模式。

use std::fs;
use std::path::{Path, PathBuf};

use ayanami::package::load_checked_manifest;
use ayanami::plugin_registry::{sha256_hex, Lockfile};

mod registry;

use registry::registry_install;

pub(super) fn plugin_install(args: &[String]) {
    let mut update = false;
    let mut positional = Vec::new();
    for arg in args {
        if arg == "--update" {
            update = true;
        } else {
            positional.push(arg.clone());
        }
    }
    let Some(target) = positional.first() else {
        eprintln!("usage: ayanami plugin install <plugin.lcl|name[@version]> [alias] [--update]");
        std::process::exit(1);
    };
    if target.ends_with(".lcl") {
        path_install(Path::new(target), positional.get(1).map(String::as_str));
    } else {
        registry_install(target, positional.get(1).map(String::as_str), update);
    }
}

pub(super) fn plugin_verify(_args: &[String]) {
    let project = current_project();
    let lock_path = project.join("ayanami.lock");
    let lock = match Lockfile::load(&lock_path) {
        Ok(l) => l,
        Err(e) => fail(e),
    };
    if lock.entries.is_empty() {
        println!("no locked plugins");
        return;
    }
    let mut failed = 0;
    for (name, entry) in &lock.entries {
        let path = project.join(&entry.path);
        match sha256_file(&path) {
            Ok(sha) if sha == entry.sha256 => {
                println!("ok {} {} {}", name, entry.version, entry.path);
            }
            Ok(sha) => {
                eprintln!(
                    "error: {} {} checksum mismatch (lock {}, actual {})",
                    name, entry.version, entry.sha256, sha
                );
                failed += 1;
            }
            Err(e) => {
                eprintln!("error: {} {}: {}", name, entry.version, e);
                failed += 1;
            }
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
}

fn path_install(src: &Path, alias_arg: Option<&str>) {
    if !src.is_file() {
        eprintln!("error: plugin install requires an existing .lcl file");
        std::process::exit(1);
    }
    let src_s = src.to_string_lossy();
    match load_checked_manifest(&src_s) {
        Ok(Some(m)) => println!("plugin: {} {} (abi {}, min compiler {})", m.name, m.version, m.abi, m.min_compiler),
        Ok(None) => {}
        Err(e) => fail(e),
    }
    let project = current_project();
    let alias = alias_arg.map(str::to_string).unwrap_or_else(|| {
        src.file_stem().unwrap_or_default().to_string_lossy().into_owned()
    });
    validate_alias(&alias);
    let dep_path = format!("plugins/{}.lcl", alias);
    let dst = project.join(&dep_path);
    if let Err(e) = ayanami::plugin_registry::copy_plugin(src, &dst) {
        fail(e);
    }
    ensure_dependency(&project, &alias, &dep_path);
    println!("installed plugin: {}", dst.display());
    println!("then: import \"{}\"", alias);
}

pub(super) fn current_project() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    match crate::find_project(&cwd) {
        Some((project, _)) => project,
        None => fail("plugin command must run inside a project (no ayanami.toml found)"),
    }
}

pub(super) fn ensure_dependency(project: &Path, alias: &str, dep_path: &str) {
    let config_path = project.join("ayanami.toml");
    let toml = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => fail(format!("failed to read '{}': {}", config_path.display(), e)),
    };
    let updated = match add_dependency(&toml, alias, dep_path) {
        Ok(s) => s,
        Err(e) => fail(e),
    };
    if updated != toml {
        if let Err(e) = fs::write(&config_path, updated) {
            fail(format!("failed to update '{}': {}", config_path.display(), e));
        }
        println!("dependency added: {} = \"{}\"", alias, dep_path);
    } else {
        println!("dependency already configured: {} = \"{}\"", alias, dep_path);
    }
}

pub(super) fn sha256_file(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| format!("failed to read '{}': {}", path.display(), e))?;
    Ok(sha256_hex(&data))
}

pub(super) fn validate_alias(alias: &str) {
    if alias.is_empty() || !alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        fail(format!("invalid plugin alias '{}'", alias));
    }
}

pub(super) fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {}", message);
    std::process::exit(1);
}

/// 在 `[dependencies]` 中新增/校验一条依赖，保留其余配置文本。
pub(super) fn add_dependency(toml: &str, alias: &str, path: &str) -> Result<String, String> {
    let mut lines: Vec<String> = toml.lines().map(str::to_string).collect();
    let mut in_deps = false;
    let mut header = None;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with('[') {
            in_deps = t.starts_with("[dependencies]");
            if in_deps { header = Some(i); }
            continue;
        }
        if !in_deps { continue; }
        if let Some((k, v)) = t.split_once('=') {
            if k.trim() == alias {
                let existing = v.trim().trim_matches('"');
                return if existing == path {
                    Ok(toml.to_string())
                } else {
                    Err(format!(
                        "dependency '{}' already exists in ayanami.toml as '{}'",
                        alias, existing
                    ))
                };
            }
        }
    }
    let entry = format!("{} = \"{}\"", alias, path);
    if let Some(i) = header {
        let mut end = lines[i + 1..]
            .iter()
            .position(|l| l.trim_start().starts_with('['))
            .map(|p| i + 1 + p)
            .unwrap_or(lines.len());
        while end > i + 1 && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        lines.insert(end, entry);
    } else {
        if lines.iter().any(|l| !l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push("[dependencies]".to_string());
        lines.push(entry);
    }
    let mut out = lines.join("\n");
    if toml.ends_with('\n') { out.push('\n'); }
    Ok(out)
}

#[cfg(test)]
mod tests;
