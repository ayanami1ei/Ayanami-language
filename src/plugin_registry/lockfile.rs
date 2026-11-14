//! A5j-2：项目 `ayanami.lock` 解析/序列化。

use crate::error::{Error, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntry {
    pub version: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub entries: BTreeMap<String, LockEntry>,
}

impl Lockfile {
    /// 缺失或空文件 → 空 lockfile；存在但格式错误 → 明确诊断。
    pub fn load(path: &Path) -> Result<Self> {
        let contents = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => {
                return Err(Error::Package(format!(
                    "failed to read lockfile '{}': {}",
                    path.display(),
                    e
                )))
            }
        };
        if contents.trim().is_empty() {
            return Ok(Self::default());
        }

        let mut entries = BTreeMap::new();
        let mut header_seen = false;
        let mut current: Option<(String, LockEntry)> = None;

        for raw in contents.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if !header_seen {
                if line != "version=1" {
                    return Err(Error::Package(format!(
                        "invalid lockfile '{}': expected 'version=1', found '{}'",
                        path.display(),
                        line
                    )));
                }
                header_seen = true;
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                close_entry(&mut entries, current.take(), path)?;
                let inner = &line[1..line.len() - 1];
                if let Some(name) = inner.strip_prefix("plugin.") {
                    validate_name(name, path)?;
                    if entries.contains_key(name) {
                        return Err(Error::Package(format!(
                            "invalid lockfile '{}': duplicate section [plugin.{}]",
                            path.display(),
                            name
                        )));
                    }
                    current = Some((name.to_string(), empty_entry()));
                }
                // 非 plugin section 忽略，同时结束上一个条目。
                continue;
            }

            let Some((name, entry)) = current.as_mut() else {
                continue;
            };
            let Some((key, value)) = line.split_once('=') else {
                return Err(Error::Package(format!(
                    "invalid lockfile '{}': expected key=value in [plugin.{}]",
                    path.display(),
                    name
                )));
            };
            let value = parse_value(value.trim());
            match key.trim() {
                "version" => entry.version = value,
                "path" => {
                    entry.path = unescape(&value);
                }
                "sha256" => {
                    if !is_sha256(&value) {
                        return Err(Error::Package(format!(
                            "invalid lockfile '{}': invalid sha256 for plugin '{}'",
                            path.display(),
                            name
                        )));
                    }
                    entry.sha256 = value.to_ascii_lowercase();
                }
                _ => {}
            }
        }

        if !header_seen {
            return Err(Error::Package(format!(
                "invalid lockfile '{}': missing version header",
                path.display()
            )));
        }
        close_entry(&mut entries, current.take(), path)?;
        Ok(Self { entries })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let mut content = String::from("version=1\n");
        for (name, entry) in &self.entries {
            content.push_str(&format!(
                "\n[plugin.{}]\nversion=\"{}\"\npath=\"{}\"\nsha256={}\n",
                name,
                entry.version,
                escape(&entry.path),
                entry.sha256
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Package(format!("failed to create '{}': {}", parent.display(), e)))?;
        }
        fs::write(path, content)
            .map_err(|e| Error::Package(format!("failed to write lockfile '{}': {}", path.display(), e)))
    }

    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        self.entries.get(name)
    }

    pub fn insert(&mut self, name: String, mut entry: LockEntry) {
        entry.sha256 = entry.sha256.to_ascii_lowercase();
        self.entries.insert(name, entry);
    }
}

fn empty_entry() -> LockEntry {
    LockEntry { version: String::new(), path: String::new(), sha256: String::new() }
}

fn close_entry(
    entries: &mut BTreeMap<String, LockEntry>,
    current: Option<(String, LockEntry)>,
    path: &Path,
) -> Result<()> {
    let Some((name, entry)) = current else { return Ok(()) };
    if entry.version.is_empty() {
        return Err(Error::Package(format!(
            "invalid lockfile '{}': [plugin.{}] missing version",
            path.display(),
            name
        )));
    }
    if entry.path.is_empty() {
        return Err(Error::Package(format!(
            "invalid lockfile '{}': [plugin.{}] missing path",
            path.display(),
            name
        )));
    }
    if Path::new(&entry.path).is_absolute()
        || entry.path.starts_with('/')
        || entry.path.starts_with('\\')
        || entry.path.contains("..")
    {
        return Err(Error::Package(format!(
            "invalid lockfile '{}': [plugin.{}] path must be project-relative",
            path.display(),
            name
        )));
    }
    if !is_sha256(&entry.sha256) {
        return Err(Error::Package(format!(
            "invalid lockfile '{}': [plugin.{}] missing or invalid sha256",
            path.display(),
            name
        )));
    }
    entries.insert(name, entry);
    Ok(())
}

fn validate_name(name: &str, path: &Path) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::Package(format!(
            "invalid lockfile '{}': invalid plugin name '{}'",
            path.display(),
            name
        )));
    }
    Ok(())
}

fn parse_value(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn unescape(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.peek() {
            Some('n') => {
                chars.next();
                out.push('\n');
            }
            Some('"') => {
                chars.next();
                out.push('"');
            }
            Some('\\') => {
                chars.next();
                out.push('\\');
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests;
