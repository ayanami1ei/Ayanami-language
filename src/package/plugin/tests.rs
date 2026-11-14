use super::*;

#[test]
fn test_parse_valid_manifest() {
    let body = r#"
[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
abi=8
min-compiler="0.6.8"
built-with="0.6.8"
"#;
    let manifest = parse_manifest(body, "test").unwrap().unwrap();
    assert_eq!(manifest.schema, 1);
    assert_eq!(manifest.name, "my_plugin");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.abi, 8);
    assert_eq!(manifest.min_compiler, "0.6.8");
    assert_eq!(manifest.built_with, "0.6.8");
}

#[test]
fn test_no_plugin_section() {
    let body = r#"
[symbols]
"#;
    let manifest = parse_manifest(body, "test").unwrap();
    assert!(manifest.is_none());
}

#[test]
fn test_missing_abi() {
    let body = r#"
[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
min-compiler="0.6.8"
built-with="0.6.8"
"#;
    assert!(parse_manifest(body, "test").is_err());
}

#[test]
fn test_validate_wrong_abi() {
    let manifest = PluginManifest {
        schema: 1,
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        abi: 7,
        min_compiler: "0.6.8".to_string(),
        built_with: "0.6.8".to_string(),
    };
    assert!(validate_manifest(&manifest, "test").is_err());
}

#[test]
fn test_validate_too_new_min_compiler() {
    let manifest = PluginManifest {
        schema: 1,
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        abi: 8,
        min_compiler: "999.0.0".to_string(),
        built_with: "0.6.8".to_string(),
    };
    assert!(validate_manifest(&manifest, "test").is_err());
}

#[test]
fn test_validate_invalid_min_compiler() {
    let manifest = PluginManifest {
        schema: 1,
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        abi: 8,
        min_compiler: "not-a-version".to_string(),
        built_with: "0.6.8".to_string(),
    };
    assert!(validate_manifest(&manifest, "test").is_err());
}

#[test]
fn test_version_at_least() {
    assert!(version_at_least("0.6.8", "0.6.7"));
    assert!(!version_at_least("0.6.8", "0.6.9"));
    assert!(version_at_least("1", "0.9.9"));
}

#[test]
fn test_plugin_section_before_later_sections() {
    let body = r#"
[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
abi=8
min-compiler="0.6.8"
built-with="0.6.8"

[symbols]
attr="identity"
"#;
    let manifest = parse_manifest(body, "test").unwrap().unwrap();
    assert_eq!(manifest.abi, 8);
}

#[test]
fn test_parse_manifest_ignores_symbols() {
    let body = r#"
[symbols]
[plugin]
manifest=1
name="my_plugin"
version="0.1.0"
abi=8
min-compiler="0.6.8"
built-with="0.6.8"
"#;
    let manifest = parse_manifest(body, "test").unwrap().unwrap();
    assert_eq!(manifest.name, "my_plugin");
}
