//! 第三方插件作者工作流：`ayanami plugin new/package`。
//!
//! 插件项目以 `src/lib.aya` 为入口，使用 `#[macro]` / `#[pass]` / `#[check]` /
//! `#[attr]` 声明导出注解；`plugin package` 校验至少有一个声明后打包为 `.lcl`。

use super::*;

pub(crate) fn cmd_plugin(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => plugin_new(&args[1..]),
        Some("package") => plugin_package(&args[1..]),
        _ => {
            eprintln!("usage: ayanami plugin <new|package> <name-or-project>");
            eprintln!("  new <name>        创建第三方插件项目");
            eprintln!("  package <project> 校验注解声明并打包 .lcl");
            std::process::exit(1);
        }
    }
}

fn plugin_new(args: &[String]) {
    let Some(name) = args.first() else {
        eprintln!("usage: ayanami plugin new <name>");
        std::process::exit(1);
    };
    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("error: directory '{}' already exists", name);
        std::process::exit(1);
    }
    let pkg_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or(name);
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap_or_else(|e| {
        eprintln!("error: failed to create '{}': {}", src_dir.display(), e);
        std::process::exit(1);
    });
    let toml = format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\n\n[build]\ntarget = \"static-lib\"\n",
        pkg_name,
        env!("CARGO_PKG_VERSION")
    );
    fs::write(dir.join("ayanami.toml"), toml).unwrap_or_else(|e| {
        eprintln!("error: failed to write ayanami.toml: {}", e);
        std::process::exit(1);
    });
    let lib = r#"// A5f 属性插件示例：返回 item 源码（identity，不做改写）。
// 也可把 #[attr] 换成 #[macro] / #[pass] / #[check]，见 docs/plugins.md。
#[attr]
pub fn identity(String input) -> String {
    return input
}
"#;
    fs::write(src_dir.join("lib.aya"), lib).unwrap_or_else(|e| {
        eprintln!("error: failed to write src/lib.aya: {}", e);
        std::process::exit(1);
    });
    let readme = format!(
        "# {}\n\nAyanami 第三方插件项目。\n\n```bash\nayanami plugin package .\nls src/lib.lcl\n```\n\n在消费方：\n\n```ayanami\nimport \"<path-to-plugin>/src/lib.lcl\"\n\n#[lib::identity]\nfn marked() -> int {{ return 7 }}\n```\n\n支持 `#[macro]` / `#[pass]` / `#[check]` / `#[attr]` 四类插件，详见 `docs/plugins.md`。\n",
        pkg_name
    );
    fs::write(dir.join("README.md"), readme).unwrap_or_else(|e| {
        eprintln!("error: failed to write README.md: {}", e);
        std::process::exit(1);
    });
    println!("created plugin project '{}'", name);
    println!("  {}/ayanami.toml", name);
    println!("  {}/src/lib.aya", name);
    println!("  {}/README.md", name);
}

fn plugin_package(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path.display(), e);
            std::process::exit(1);
        }
    };
    if !has_plugin_annotation(&code) {
        eprintln!(
            "error: plugin source '{}' has no #[macro]/#[pass]/#[check]/#[attr] declaration",
            path.display()
        );
        std::process::exit(1);
    }
    let path_s = path.to_string_lossy().into_owned();
    if let Err(e) = ayanami::compiler::package_source(&path_s, &code) {
        eprintln!("error: plugin package failed: {}", e);
        std::process::exit(1);
    }
    let lcl = path.with_extension("lcl");
    println!("plugin package: {}", lcl.display());
}

fn has_plugin_annotation(code: &str) -> bool {
    let Ok(program) = ayanami::parser::parse_source(code) else { return false };
    program.stmts.iter().any(stmt_has_plugin_decl)
}

fn stmt_has_plugin_decl(stmt: &ayanami::parser::ast::Stmt) -> bool {
    use ayanami::parser::ast::Stmt;
    match stmt {
        Stmt::FnDecl { attrs, .. } => attrs.iter().any(|a| {
            a.is_builtin()
                && matches!(a.name.as_str().as_str(), "macro" | "pass" | "check" | "attr")
        }),
        Stmt::Namespace { items, .. } => items.iter().any(stmt_has_plugin_decl),
        Stmt::ImplBlock { methods, .. } => methods.iter().any(stmt_has_plugin_decl),
        _ => false,
    }
}
