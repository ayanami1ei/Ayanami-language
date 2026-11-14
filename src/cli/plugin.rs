//! 第三方插件作者工作流：`ayanami plugin new/package/install`。
//!
//! 插件项目以 `src/lib.aya` 为入口，使用 `#[macro]` / `#[pass]` / `#[check]` /
//! `#[attr]` 声明导出注解；`plugin package` 校验至少有一个声明后打包为 `.lcl`。

use super::*;

pub(crate) fn cmd_plugin(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => plugin_new(&args[1..]),
        Some("package") => plugin_package(&args[1..]),
        Some("install") => plugin_install(&args[1..]),
        _ => {
            eprintln!("usage: ayanami plugin <new|package|install> ...");
            eprintln!("  new <name>        创建第三方插件项目");
            eprintln!("  package <project> 校验注解声明并打包 .lcl");
            eprintln!("  install <lcl> [alias] 安装到当前项目 plugins/ 并写入依赖");
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
        "# {}\n\nAyanami 第三方插件项目。\n\n```bash\nayanami plugin package .\nls src/lib.lcl\n```\n\n在消费方（`.lcl` 跨平台，动态库由消费方本机构建）：\n\n```bash\ncd <consumer-project>\nayanami plugin install <path-to-plugin>/src/lib.lcl {}\n# 会自动写入 ayanami.toml 的 [dependencies]\n```\n\n```ayanami\nimport \"{}\"\n\n#[{}::identity]\nfn marked() -> int {{ return 7 }}\n```\n\n支持 `#[macro]` / `#[pass]` / `#[check]` / `#[attr]` 四类插件，详见 `docs/plugins.md`。\n",
        pkg_name, pkg_name, pkg_name, pkg_name
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

fn plugin_install(args: &[String]) {
    let Some(src) = args.first() else {
        eprintln!("usage: ayanami plugin install <plugin.lcl> [alias]");
        std::process::exit(1);
    };
    let src = Path::new(src);
    if src.extension().and_then(|e| e.to_str()) != Some("lcl") || !src.is_file() {
        eprintln!("error: plugin install requires an existing .lcl file");
        std::process::exit(1);
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    let Some((project, _)) = crate::find_project(&cwd) else {
        eprintln!("error: plugin install must run inside a project (no ayanami.toml found)");
        std::process::exit(1);
    };
    let alias = args.get(1).cloned().unwrap_or_else(|| {
        src.file_stem().unwrap_or_default().to_string_lossy().into_owned()
    });
    if alias.is_empty() || !alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        eprintln!("error: invalid plugin alias '{}'", alias);
        std::process::exit(1);
    }
    let dep_path = format!("plugins/{}.lcl", alias);
    let config_path = project.join("ayanami.toml");
    let toml = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", config_path.display(), e);
            std::process::exit(1);
        }
    };
    let updated = match add_dependency(&toml, &alias, &dep_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };
    let dir = project.join("plugins");
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("error: failed to create '{}': {}", dir.display(), e);
        std::process::exit(1);
    }
    let dst = dir.join(format!("{}.lcl", alias));
    if let Err(e) = fs::copy(src, &dst) {
        eprintln!("error: failed to install plugin to '{}': {}", dst.display(), e);
        std::process::exit(1);
    }
    if updated != toml {
        if let Err(e) = fs::write(&config_path, updated) {
            eprintln!("error: failed to update '{}': {}", config_path.display(), e);
            std::process::exit(1);
        }
        println!("dependency added: {} = \"{}\"", alias, dep_path);
    } else {
        println!("dependency already configured: {} = \"{}\"", alias, dep_path);
    }
    println!("installed plugin: {}", dst.display());
    println!("then: import \"{}\"", alias);
}

/// 在 `[dependencies]` 中新增/校验一条依赖，保留其余配置文本。
fn add_dependency(toml: &str, alias: &str, path: &str) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
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
}
