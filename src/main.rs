// ============================================================
//  Ayanami 编译器 —— CLI 入口
//  命令行接口，支持以下命令：
//    new     — 创建新项目骨架
//    check   — 前端检查（lex → parse → HIR → MIR → 借用检查）
//    fmt     — 格式化代码
//    package — 打包为 .lcl（不生成可执行文件）
//    build   — 完整构建（生成可执行文件/静态库/动态库）
//    install — 从 .lcl 构建目标产物
//    defs    — 输出符号定义列表（JSON）
//    run     — 构建并运行
//    clean   — 清除构建产物
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ayanami <command> [args...]");
        eprintln!("commands:");
        eprintln!("  new <name>         创建新项目");
        eprintln!("  check [--watch] <file/proj> 前端检查（lex, parse, HIR）；--watch 监听文件变更自动重检");
        eprintln!("  fmt [file/dir]     格式化代码（类似 rustfmt）");
        eprintln!("  package <file/proj> 打包为 .lcl（不生成可执行文件）");
        eprintln!("  build [file/proj]  构建可执行文件 + .lcl 包");
        eprintln!("  install <lcl>      从 .lcl 构建目标产物");
        eprintln!("  defs <file>        输出符号定义列表（JSON 格式）");
        eprintln!("  run [file/proj]    构建并运行");
        eprintln!("  clean              清除 build/ 目录");
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "new" => cmd_new(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "fmt" => cmd_fmt(&args[2..]),
        "package" => cmd_package(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "install" => cmd_install(&args[2..]),
        "defs" => cmd_defs(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "clean" => cmd_clean(),
        _ => {
            eprintln!("unknown command: {}", command);
            std::process::exit(1);
        }
    }
}

/// 从 `dir` 向上级目录搜索 ayanami.toml 文件，找到项目根目录
/// 返回 (项目目录, toml 内容)
fn find_project(dir: &Path) -> Option<(PathBuf, String)> {
    let mut d = Some(dir);
    while let Some(cur) = d {
        let toml = cur.join("ayanami.toml");
        if toml.exists() {
            if let Ok(content) = fs::read_to_string(&toml) {
                return Some((cur.to_path_buf(), content));
            }
        }
        d = cur.parent();
    }
    None
}

/// 读取项目入口文件（默认 src/main.aya）
fn project_entry(project_dir: &Path) -> Option<PathBuf> {
    let main_aya = project_dir.join("src").join("main.aya");
    if main_aya.exists() { Some(main_aya) } else { None }
}

/// 解析命令行路径参数：
/// - 如果以 `.aya` / `.lcl` 结尾 → 直接作为文件路径
/// - 否则 → 当作项目目录，查找 src/main.aya
/// - 无参数 → 当前目录作为项目目录
fn resolve_path(arg: Option<&str>) -> PathBuf {
    match arg {
        Some(p) if p.ends_with(".aya") || p.ends_with(".lcl") => PathBuf::from(p),
        Some(p) => {
            // Treat as project directory
            let project_dir = Path::new(p);
            if let Some((_, _)) = find_project(project_dir) {
                project_entry(project_dir).unwrap_or_else(|| {
                    eprintln!("error: no src/main.aya in project '{}'", p);
                    std::process::exit(1);
                })
            } else {
                PathBuf::from(p)
            }
        }
        None => {
            // No arg: try current directory as project
            let cwd = std::env::current_dir().unwrap_or_default();
            if let Some((proj_dir, _)) = find_project(&cwd) {
                project_entry(&proj_dir).unwrap_or(cwd.join("main.aya"))
            } else {
                eprintln!("error: not in a project (no ayanami.toml found)");
                std::process::exit(1);
            }
        }
    }
}

fn cmd_new(args: &[String]) {
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
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[build]\ntarget = \"executable\"\n\n[build.targets]\n# \"src/utils.aya\" = \"static-lib\"\n# \"src/plugin.aya\" = \"dynamic-lib\"\n",
        name
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

fn cmd_fmt(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();
    if !path_str.ends_with(".aya") {
        eprintln!("error: fmt requires a .aya file");
        std::process::exit(1);
    }
    let code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path.display(), e);
            std::process::exit(1);
        }
    };
    match ayanami::formatter::format_file(&code) {
        Ok(formatted) => {
            if formatted == code {
                // No changes needed
            } else {
                fs::write(&path, &formatted).unwrap_or_else(|e| {
                    eprintln!("error: failed to write '{}': {}", path.display(), e);
                    std::process::exit(1);
                });
            }
            print!("{}", formatted);
        }
        Err(e) => {
            eprintln!("fmt error: {}", e);
            std::process::exit(1);
        }
    }
}

fn do_check(path: &Path) -> Result<(), String> {
    let tmp_dir = std::env::temp_dir().join("ayanami-check");
    std::fs::create_dir_all(&tmp_dir).ok();
    let mut compiling = std::collections::HashSet::new();
    let mut cache = std::collections::HashMap::new();
    let base = path.parent().unwrap_or(std::path::Path::new("."));
    let result = ayanami::compiler::compile_file(path, base, &tmp_dir, &mut compiling, &mut cache, None);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map(|_| ())
}

fn cmd_check(args: &[String]) {
    let watch = args.first().map(|s| s.as_str()) == Some("--watch");
    let path_arg = if watch { args.get(1) } else { args.first() };
    let path = resolve_path(path_arg.map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();
    if !path_str.ends_with(".aya") {
        eprintln!("error: check requires a .aya file"); std::process::exit(1);
    }
    match do_check(&path) {
        Ok(_) => println!("check passed: {}", path.display()),
        Err(e) => { eprintln!("check failed: {}", e); std::process::exit(1); }
    }
    if watch {
        watch_file(&path);
    }
}

fn watch_file(path: &Path) {
    use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let path = path.to_path_buf();
    let canonical = path.canonicalize().unwrap_or(path.clone());
    let dir_to_watch = canonical.parent().unwrap_or(&canonical).to_path_buf();

    println!("watching {} for changes...", canonical.display());

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Config::default()).expect("failed to create watcher");
    watcher
        .watch(&dir_to_watch, RecursiveMode::NonRecursive)
        .expect("failed to watch directory");

    for res in rx {
        match res {
            Ok(event) => {
                let is_relevant = matches!(&event.kind, EventKind::Modify(_) | EventKind::Create(_))
                    && event.paths.iter().any(|p| {
                        p.canonicalize().map(|c| c == canonical).unwrap_or(false)
                    });
                if !is_relevant {
                    continue;
                }
                // Small debounce: skip if the event is within 200ms of the last check
                std::thread::sleep(Duration::from_millis(200));
                match do_check(&canonical) {
                    Ok(_) => println!(
                        "\ncheck passed: {}",
                        canonical.display()
                    ),
                    Err(e) => eprintln!("\ncheck failed: {}", e),
                }
            }
            Err(e) => eprintln!("watch error: {}", e),
        }
    }
}

fn cmd_package(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();
    let code = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    match ayanami::compiler::package_source(&path_str, &code) {
        Ok(()) => {}
        Err(e) => { eprintln!("package failed: {}", e); std::process::exit(1); }
    }
}

fn cmd_install(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami install <package.lcl>");
        std::process::exit(1);
    }
    let lcl_path = &args[0];
    match ayanami::compiler::install_package(lcl_path, None) {
        Ok(()) => {}
        Err(e) => { eprintln!("install failed: {}", e); std::process::exit(1); }
    }
}

fn cmd_defs(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami defs <file.aya>");
        std::process::exit(1);
    }
    let path = std::path::Path::new(&args[0]);
    let path_str = path.to_string_lossy().into_owned();
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    let mut lexer = ayanami::lexer::Lexer::new(&code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, ayanami::lexer::TokenKind::EOF))
        .collect();
    let mut parser = ayanami::parser::Parser::new(filtered);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => { eprintln!("parse error: {}", e); std::process::exit(1); }
    };
    let mut defs = Vec::new();
    ayanami::compiler::collect_defs_from_stmts(&program.stmts, &path_str, "", &mut defs);
    // Also collect from recursively resolved .aya imports
    let mut visited = std::collections::HashSet::new();
    visited.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    resolve_import_defs(&program.stmts, &path_str, &mut visited, &mut defs);
    print!("{}", ayanami::compiler::defs_to_json(&defs));
}

/// Recursively collect defs from imported .aya files.
fn resolve_import_defs(stmts: &[ayanami::parser::ast::Stmt], base_path: &str, visited: &mut std::collections::HashSet<std::path::PathBuf>, defs: &mut Vec<ayanami::compiler::SymDef>) {
    let base_dir = std::path::Path::new(base_path).parent().unwrap_or(std::path::Path::new("."));
    for stmt in stmts {
        if let ayanami::parser::ast::Stmt::Import { path, .. } = stmt {
            let dep = base_dir.join(path);
            let dep_aya = if dep.to_string_lossy().ends_with(".aya") {
                dep
            } else {
                let with_ext = dep.with_extension("aya");
                if with_ext.exists() { with_ext } else { continue; }
            };
            let canonical = match dep_aya.canonicalize() {
                Ok(c) => c,
                Err(_) => continue,
            };
            if !visited.insert(canonical.clone()) { continue; }
            if !canonical.to_string_lossy().ends_with(".aya") { continue; }
            let code = match std::fs::read_to_string(&canonical) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut lexer = ayanami::lexer::Lexer::new(&code);
            let tokens = lexer.tokenize_all();
            let filtered: Vec<_> = tokens.into_iter()
                .filter(|t| !matches!(t.kind, ayanami::lexer::TokenKind::EOF))
                .collect();
            let mut parser = ayanami::parser::Parser::new(filtered);
            if let Ok(prog) = parser.parse_program() {
                let dep_str = canonical.to_string_lossy().into_owned();
                ayanami::compiler::collect_defs_from_stmts(&prog.stmts, &dep_str, "", defs);
                resolve_import_defs(&prog.stmts, &dep_str, visited, defs);
            }
        }
    }
}

/// Load project config from current or parent directory.
fn load_config() -> Option<(PathBuf, ayanami::package::config::ProjectConfig)> {
    let cwd = std::env::current_dir().ok()?;
    if let Some((proj_dir, toml_str)) = find_project(&cwd) {
        Some((proj_dir, ayanami::package::config::ProjectConfig::load(&toml_str)))
    } else {
        None
    }
}

fn cmd_clean() {
    let build_dir = Path::new("build");
    if build_dir.exists() {
        fs::remove_dir_all(build_dir).unwrap_or_else(|e| {
            eprintln!("error: failed to remove build/: {}", e);
            std::process::exit(1);
        });
        println!("removed build/");
    } else {
        println!("build/ does not exist");
    }
}

fn cmd_build(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();

    let target = load_config()
        .map(|(_, cfg)| cfg.resolve_target(&path).to_string())
        .unwrap_or_else(|| {
            if path_str.ends_with("main.aya") { "executable".into() } else { "static-lib".into() }
        });

    let code = match fs::read_to_string(&path) {
        Ok(c) => c, Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    match ayanami::compiler::build_source_with_target(&path_str, &code, "build", Some(&target)) {
        Ok(()) => {}
        Err(e) => { eprintln!("build failed: {}", e); std::process::exit(1); }
    }
}

fn cmd_run(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();

    let target = "executable";

    let code = match fs::read_to_string(&path) {
        Ok(c) => c, Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    if let Err(e) = ayanami::compiler::build_source_with_target(&path_str, &code, "build", Some(target)) {
        eprintln!("build failed: {}", e); std::process::exit(1);
    }
    let exe_name = path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy();
    match ayanami::compiler::run_executable(&exe_name) {
        Ok(code) => println!("exit code: {}", code),
        Err(e) => { eprintln!("run failed: {}", e); std::process::exit(1); }
    }
}
