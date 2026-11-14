
use crate::error::{Error, Result};
use crate::package::parse_version;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryVersion {
    pub version: String,
    pub file: String,
}

#[derive(Debug, Default, Clone)]
pub struct RegistryIndex {
    pub entries: BTreeMap<String, Vec<RegistryVersion>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPlugin {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

impl RegistryIndex {
    /// Read `<root>/index.toml`; missing/invalid file => Error::Package.
    pub fn load(root: &Path) -> Result<Self> {
        let index_path = root.join("index.toml");
        let content = fs::read_to_string(&index_path)
            .map_err(|_| Error::Package(format!("registry file '{}' not found", index_path.display())))?;
        
        let mut index = RegistryIndex::default();
        let mut current_plugin: Option<String> = None;
        let mut seen_versions: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            
            if line.starts_with('[') && line.ends_with(']') {
                let name = &line[1..line.len()-1];
                if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    return Err(Error::Package(format!("invalid plugin name '{}' in registry '{}'", name, index_path.display())));
                }
                if name.is_empty() {
                    return Err(Error::Package(format!("empty plugin name in registry '{}'", index_path.display())));
                }
                if index.entries.contains_key(name) {
                    return Err(Error::Package(format!("duplicate plugin section '{}' in registry '{}'", name, index_path.display())));
                }
                current_plugin = Some(name.to_string());
                seen_versions.clear();
                continue;
            }
            
            if let Some(plugin_name) = &current_plugin {
                if let Some((version_str, file)) = line.split_once('=') {
                    let version_str = version_str.trim();
                    let file = file.trim().trim_matches('"');
                    
                    if !file.ends_with(".lcl") {
                        return Err(Error::Package(format!("plugin '{}' version '{}' file '{}' must end with .lcl", plugin_name, version_str, file)));
                    }
                    
                    if Path::new(file).is_absolute() || file.starts_with('/') || file.starts_with('\\') || file.contains(':') {
                        return Err(Error::Package(format!("plugin '{}' version '{}' file '{}' must be relative", plugin_name, version_str, file)));
                    }
                    
                    if file.contains("..") {
                        return Err(Error::Package(format!("plugin '{}' version '{}' file '{}' must not contain ..", plugin_name, version_str, file)));
                    }
                    
                    let version_tuple = parse_version(version_str)
                        .ok_or_else(|| Error::Package(format!("invalid version '{}' in registry '{}'", version_str, index_path.display())))?;
                    
                    if seen_versions.contains_key(version_str) {
                        return Err(Error::Package(format!("duplicate version '{}' in plugin '{}' in registry '{}'", version_str, plugin_name, index_path.display())));
                    }
                    
                    for (other_version, other_tuple) in &seen_versions {
                        if *other_tuple == version_tuple {
                            return Err(Error::Package(format!("duplicate semver {} and {} in plugin '{}' in registry '{}'", other_version, version_str, plugin_name, index_path.display())));
                        }
                    }
                    
                    seen_versions.insert(version_str.to_string(), version_tuple);
                    index.entries.entry(plugin_name.clone())
                        .or_default()
                        .push(RegistryVersion {
                            version: version_str.to_string(),
                            file: file.to_string(),
                        });
                }
            }
        }
        
        Ok(index)
    }
    
    /// Resolve name + optional version request.
    /// None or Some("*") => highest parsed semver.
    /// Otherwise exact semver equality (1.2 == 1.2.0).
    /// On missing plugin/version, error lists available versions.
    pub fn resolve(&self, root: &Path, name: &str, requested: Option<&str>) -> Result<ResolvedPlugin> {
        let entries = self.entries.get(name)
            .ok_or_else(|| Error::Package(format!("plugin '{}' not found in registry '{}'", name, root.join("index.toml").display())))?;
        
        let requested_version = match requested {
            None | Some("*") => {
                let latest = entries.iter()
                    .max_by_key(|v| parse_version(&v.version).unwrap())
                    .ok_or_else(|| Error::Package(format!("plugin '{}' has no versions (available: )", name)))?;
                latest.version.clone()
            },
            Some(version) => version.to_string(),
        };
        
        let requested_tuple = parse_version(&requested_version)
            .ok_or_else(|| Error::Package(format!("invalid version '{}' in registry '{}'", requested_version, root.join("index.toml").display())))?;
        
        // Check for exact match
        if let Some(entry) = entries.iter().find(|v| {
            parse_version(&v.version).map_or(false, |tuple| tuple == requested_tuple)
        }) {
            let path = root.join(&entry.file);
            if !path.is_file() {
                return Err(Error::Package(format!("registry entry '{}' points to missing file '{}'", name, path.display())));
            }
            return Ok(ResolvedPlugin {
                name: name.to_string(),
                version: entry.version.clone(),
                path,
            });
        }
        
        // No exact match found
        let mut available_versions: Vec<_> = entries.iter()
            .map(|v| v.version.clone())
            .collect();
        available_versions.sort_by_key(|v| parse_version(v).unwrap());
        let available = available_versions.join(", ");
        Err(Error::Package(format!("plugin '{}' has no version matching '{}' (available: {})", name, requested_version, available)))
    }
    
    pub fn versions(&self, name: &str) -> Vec<&str> {
        if let Some(entries) = self.entries.get(name) {
            let mut versions: Vec<_> = entries.iter()
                .map(|v| v.version.as_str())
                .collect();
            versions.sort_by(|a, b| {
                let a_tuple = parse_version(a).unwrap();
                let b_tuple = parse_version(b).unwrap();
                a_tuple.cmp(&b_tuple)
            });
            versions
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests;
