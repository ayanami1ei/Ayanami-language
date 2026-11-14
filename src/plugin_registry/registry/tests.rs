use super::*;
use std::fs;
use std::path::PathBuf;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ayanami-registry-{}-{}-{}",
        std::process::id(),
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn touch_lcl(root: &std::path::Path, name: &str) {
    fs::write(root.join(name), b"LCL1").unwrap();
}

#[test]
fn resolve_exact_by_numeric_version() {
    let root = temp_dir("exact");
    touch_lcl(&root, "demo-1.2.0.lcl");
    fs::write(root.join("index.toml"), "[demo]\n1.2.0 = \"demo-1.2.0.lcl\"\n").unwrap();
    let index = RegistryIndex::load(&root).unwrap();
    let resolved = index.resolve(&root, "demo", Some("1.2")).unwrap();
    assert_eq!(resolved.version, "1.2.0");
    assert!(resolved.path.is_file());
}

#[test]
fn resolve_latest() {
    let root = temp_dir("latest");
    for v in ["1.1.0", "1.2.0", "1.1.1"] {
        touch_lcl(&root, &format!("demo-{}.lcl", v));
    }
    fs::write(
        root.join("index.toml"),
        "[demo]\n1.1.0 = \"demo-1.1.0.lcl\"\n1.2.0 = \"demo-1.2.0.lcl\"\n1.1.1 = \"demo-1.1.1.lcl\"\n",
    )
    .unwrap();
    let index = RegistryIndex::load(&root).unwrap();
    let resolved = index.resolve(&root, "demo", None).unwrap();
    assert_eq!(resolved.version, "1.2.0");
}

#[test]
fn missing_version_lists_available() {
    let root = temp_dir("missing_version");
    for v in ["1.1.0", "1.2.0"] {
        touch_lcl(&root, &format!("demo-{}.lcl", v));
    }
    fs::write(
        root.join("index.toml"),
        "[demo]\n1.1.0 = \"demo-1.1.0.lcl\"\n1.2.0 = \"demo-1.2.0.lcl\"\n",
    )
    .unwrap();
    let index = RegistryIndex::load(&root).unwrap();
    let err = index.resolve(&root, "demo", Some("9.9.9")).unwrap_err().to_string();
    assert!(err.contains("available: 1.1.0, 1.2.0"), "{}", err);
}

#[test]
fn duplicate_semver_tuple_rejected() {
    let root = temp_dir("duplicate_version");
    fs::write(
        root.join("index.toml"),
        "[demo]\n1.2.0 = \"demo-1.2.0.lcl\"\n1.2 = \"demo-1.2.lcl\"\n",
    )
    .unwrap();
    assert!(RegistryIndex::load(&root).is_err());
}

#[test]
fn duplicate_section_rejected() {
    let root = temp_dir("duplicate_section");
    fs::write(
        root.join("index.toml"),
        "[demo]\n1.0.0 = \"a.lcl\"\n\n[demo]\n2.0.0 = \"b.lcl\"\n",
    )
    .unwrap();
    assert!(RegistryIndex::load(&root).is_err());
}

#[test]
fn path_traversal_rejected() {
    let root = temp_dir("traversal");
    fs::write(root.join("index.toml"), "[demo]\n1.0.0 = \"../outside.lcl\"\n").unwrap();
    assert!(RegistryIndex::load(&root).is_err());
}

#[test]
fn absolute_path_rejected() {
    let root = temp_dir("absolute");
    fs::write(root.join("index.toml"), "[demo]\n1.0.0 = \"/absolute.lcl\"\n").unwrap();
    assert!(RegistryIndex::load(&root).is_err());
}

#[test]
fn insert_save_roundtrip() {
    let root = temp_dir("save");
    touch_lcl(&root, "demo-0.1.0.lcl");
    let mut index = RegistryIndex::default();
    index.insert("demo", "0.1.0", "demo-0.1.0.lcl", false).unwrap();
    index.save(&root).unwrap();
    let loaded = RegistryIndex::load(&root).unwrap();
    assert_eq!(loaded.resolve(&root, "demo", Some("0.1.0")).unwrap().version, "0.1.0");
    index.insert("demo", "0.1", "other.lcl", false).unwrap_err();
    index.insert("demo", "0.1", "other.lcl", true).unwrap();
    assert_eq!(index.entries["demo"][0].file, "other.lcl");
}

#[test]
fn missing_target_file_error() {
    let root = temp_dir("missing_file");
    fs::write(root.join("index.toml"), "[demo]\n1.0.0 = \"missing.lcl\"\n").unwrap();
    let index = RegistryIndex::load(&root).unwrap();
    assert!(index.resolve(&root, "demo", Some("1.0.0")).is_err());
}
