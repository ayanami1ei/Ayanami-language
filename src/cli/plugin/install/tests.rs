use super::add_dependency;

#[test]
fn adds_dependency_section() {
    let out = add_dependency("[package]\nname = \"demo\"\n", "demo", "plugins/demo.lcl").unwrap();
    assert!(out.contains("[dependencies]\ndemo = \"plugins/demo.lcl\""));
}

#[test]
fn inserts_into_existing_section() {
    let src = "[package]\nname = \"demo\"\n\n[dependencies]\nfoo = \"foo.lcl\"\n\n[build]\ntarget = \"exe\"\n";
    let out = add_dependency(src, "bar", "plugins/bar.lcl").unwrap();
    assert!(out.contains("[dependencies]\nfoo = \"foo.lcl\"\nbar = \"plugins/bar.lcl\"\n\n[build]"));
}

#[test]
fn keeps_identical_and_rejects_conflict() {
    let src = "[dependencies]\ndemo = \"plugins/demo.lcl\"\n";
    assert_eq!(add_dependency(src, "demo", "plugins/demo.lcl").unwrap(), src);
    assert!(add_dependency(src, "demo", "plugins/other.lcl").is_err());
}
