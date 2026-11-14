use super::*;
use std::path::PathBuf;

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ayanami-lock-{}-{}-{}",
        std::process::id(),
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

#[test]
fn missing_file_is_empty() {
    let path = temp_path("missing");
    assert!(Lockfile::load(&path).unwrap().entries.is_empty());
}

#[test]
fn roundtrip_two_entries() {
    let mut lockfile = Lockfile::default();
    lockfile.insert(
        "my_plugin".into(),
        LockEntry {
            version: "0.1.0".into(),
            path: "plugins/my_plugin.lcl".into(),
            sha256: "A1B2C3D4E5F67890123456789012345678901234567890123456789012345678".into(),
        },
    );
    lockfile.insert(
        "other_plugin".into(),
        LockEntry {
            version: "0.2.0".into(),
            path: "plugins/other.lcl".into(),
            sha256: "FEDCBA9876543210012345678901234567890123456789012345678901234567".into(),
        },
    );
    let path = temp_path("roundtrip");
    lockfile.save(&path).unwrap();
    let loaded = Lockfile::load(&path).unwrap();
    assert_eq!(lockfile, loaded);
    let _ = fs::remove_file(path);
}

#[test]
fn wrong_header_rejected() {
    let path = temp_path("header");
    fs::write(&path, "version=2\n").unwrap();
    assert!(Lockfile::load(&path).is_err());
    let _ = fs::remove_file(path);
}

#[test]
fn leading_comments_before_header_allowed() {
    let path = temp_path("comments");
    fs::write(&path, "# lockfile\n\nversion=1\n").unwrap();
    assert!(Lockfile::load(&path).unwrap().entries.is_empty());
    let _ = fs::remove_file(path);
}

#[test]
fn missing_sha256_rejected() {
    let path = temp_path("missing_sha");
    fs::write(
        &path,
        "version=1\n\n[plugin.test]\nversion=\"0.1.0\"\npath=\"test.lcl\"\n",
    )
    .unwrap();
    assert!(Lockfile::load(&path).is_err());
    let _ = fs::remove_file(path);
}

#[test]
fn invalid_sha256_rejected() {
    let path = temp_path("bad_sha");
    fs::write(
        &path,
        "version=1\n\n[plugin.test]\nversion=\"0.1.0\"\npath=\"test.lcl\"\nsha256=\"invalid\"\n",
    )
    .unwrap();
    assert!(Lockfile::load(&path).is_err());
    let _ = fs::remove_file(path);
}

#[test]
fn duplicate_section_rejected() {
    let path = temp_path("duplicate");
    fs::write(
        &path,
        "version=1\n\n[plugin.test]\nversion=\"0.1.0\"\npath=\"a.lcl\"\nsha256=\"a1b2c3d4e5f67890123456789012345678901234567890123456789012345678\"\n\n[plugin.test]\nversion=\"0.2.0\"\npath=\"b.lcl\"\nsha256=\"a1b2c3d4e5f67890123456789012345678901234567890123456789012345678\"\n",
    )
    .unwrap();
    assert!(Lockfile::load(&path).is_err());
    let _ = fs::remove_file(path);
}

#[test]
fn path_traversal_rejected() {
    let path = temp_path("traversal");
    fs::write(
        &path,
        "version=1\n\n[plugin.test]\nversion=\"0.1.0\"\npath=\"../outside.lcl\"\nsha256=\"a1b2c3d4e5f67890123456789012345678901234567890123456789012345678\"\n",
    )
    .unwrap();
    assert!(Lockfile::load(&path).is_err());
    let _ = fs::remove_file(path);
}

#[test]
fn escaping_roundtrip() {
    let mut lockfile = Lockfile::default();
    lockfile.insert(
        "escaped".into(),
        LockEntry {
            version: "0.1.0".into(),
            path: "C:\\Users\\Test\\plugin.lcl".into(),
            sha256: "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678".into(),
        },
    );
    let path = temp_path("escape");
    lockfile.save(&path).unwrap();
    assert_eq!(lockfile, Lockfile::load(&path).unwrap());
    let _ = fs::remove_file(path);
}
