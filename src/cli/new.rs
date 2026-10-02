use super::*;

pub(crate) fn cmd_new(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami new <project-name>");
        std::process::exit(1);
    }
    let name = &args[0];
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("error: directory '{}' already exists", name);
        std::process::exit(1);
    }
    fs::create_dir(dir).unwrap_or_else(|e| {
        eprintln!("error: failed to create '{}': {}", name, e);
        std::process::exit(1);
    });
    let src_dir = dir.join("src");
    fs::create_dir(&src_dir).unwrap_or_else(|e| {
        eprintln!("error: failed to create '{}/src': {}", name, e);
        std::process::exit(1);
    });
    let main_aya = src_dir.join("main.aya");
    fs::write(&main_aya, "fn main() -> int {\n    return 0;\n}\n").unwrap_or_else(|e| {
        eprintln!("error: failed to write main.aya: {}", e);
        std::process::exit(1);
    });
    let toml = dir.join("ayanami.toml");
    fs::write(&toml, format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\n\n[build]\ntarget = \"executable\"\n\n[build.targets]\n# \"src/utils.aya\" = \"static-lib\"\n# \"src/plugin.aya\" = \"dynamic-lib\"\n",
        name,
        env!("CARGO_PKG_VERSION")
    )).unwrap_or_else(|e| {
        eprintln!("error: failed to write ayanami.toml: {}", e);
        std::process::exit(1);
    });
    println!("created project '{}'", name);
    println!("  {}/", name);
    println!("  ├── ayanami.toml");
    println!("  └── src/");
    println!("       └── main.aya");
}
