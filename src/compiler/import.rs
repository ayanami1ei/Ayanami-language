/// Find the standard library directory (next to the compiler binary).
pub fn find_std_dir() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    // Production: exe_dir/std/  (e.g. install/std/)
    let prod = exe_dir.join("std");
    if prod.join("io.lcl").exists() {
        return Some(prod);
    }
    // Dev: exe_dir/../std/  (e.g. target/debug/../std/ = ./std/)
    let dev = exe_dir.join("../std");
    if dev.join("io.lcl").exists() {
        return Some(dev);
    }
    None
}

/// Resolve an import path using config [dependencies] aliases and std library.
/// Returns Some(path) if found, None if the path doesn't exist.
pub fn resolve_import_path(
    import_path: &str,
    base_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    // Direct file check first
    let direct = base_dir.join(import_path);
    if direct.exists() {
        return Some(direct);
    }
    // Try with .aya extension
    let with_aya = base_dir.join(format!("{}.aya", import_path));
    if with_aya.exists() {
        return Some(with_aya);
    }

    // If it's a .lcl path and doesn't exist, fail (not in std dir)
    if import_path.ends_with(".lcl") {
        return None;
    }

    // Strip extensions for std dir lookup
    let stem = import_path
        .strip_suffix(".aya")
        .or_else(|| import_path.strip_suffix(".lcl"))
        .unwrap_or(import_path);

    // No extension: try config alias. Walk up from base_dir to find ayanami.toml
    let mut dir = Some(base_dir);
    while let Some(d) = dir {
        let toml_path = d.join("ayanami.toml");
        if toml_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&toml_path) {
                let config = crate::package::config::ProjectConfig::load(&content);
                if let Some(alias_path) = config.dependencies.get(import_path) {
                    let full = d.join(alias_path);
                    if full.exists() {
                        return Some(full);
                    }
                    let alt = base_dir.join(alias_path);
                    if alt.exists() {
                        return Some(alt);
                    }
                }
            }
            break;
        }
        dir = d.parent();
    }

    // Finally, try the standard library directory, preferring .aya over .lcl
    if let Some(std_dir) = find_std_dir() {
        let std_aya = std_dir.join(format!("{}.aya", stem));
        if std_aya.exists() {
            return Some(std_aya);
        }
        let std_lcl = std_dir.join(format!("{}.lcl", stem));
        if std_lcl.exists() {
            return Some(std_lcl);
        }
    }
    None
}

/// Try to load project config and resolve target for a file path.
pub fn load_config_for_file(file_path: &std::path::Path, _out_dir: &std::path::Path) -> Option<String> {
    let mut dir = file_path.parent()?;
    loop {
        let toml = dir.join("ayanami.toml");
        if toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&toml) {
                let cfg = crate::package::config::ProjectConfig::load(&content);
                let rel_path = file_path.strip_prefix(dir).ok()?;
                return Some(cfg.resolve_target(rel_path).to_string());
            }
        }
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }
    None
}
