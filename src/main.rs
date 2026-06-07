use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ayanami <command> [args...]");
        eprintln!("commands:");
        eprintln!("  new <name>         创建新项目");
        eprintln!("  check <file/proj>  前端检查（lex, parse, HIR）");
        eprintln!("  package <file/proj> 打包为 .lcl（不生成可执行文件）");
        eprintln!("  build [file/proj]  构建可执行文件 + .lcl 包");
        eprintln!("  install <lcl>      从 .lcl 构建目标产物");
        eprintln!("  run [file/proj]    构建并运行");
        eprintln!("  clean              清除 build/ 目录");
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "new" => cmd_new(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "package" => cmd_package(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "install" => cmd_install(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "clean" => cmd_clean(),
        _ => {
            eprintln!("unknown command: {}", command);
            std::process::exit(1);
        }
    }
}

/// Try to find ayanami.toml from `dir` upward. Returns (project_dir, config).
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

/// Read project entry point from ayanami.toml, default to src/main.aya.
fn project_entry(project_dir: &Path) -> Option<PathBuf> {
    let main_aya = project_dir.join("src").join("main.aya");
    if main_aya.exists() { Some(main_aya) } else { None }
}

/// Resolve target: if arg is a .aya file, use it; else treat as project dir (or cwd).
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

fn cmd_check(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    if !path.to_string_lossy().ends_with(".aya") {
        eprintln!("error: check requires a .aya file"); std::process::exit(1);
    }
    let code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    match ayanami::compiler::check_source(&code, "check_output") {
        Ok(()) => println!("check passed: {}", path.display()),
        Err(e) => { eprintln!("check failed: {}", e); std::process::exit(1); }
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

    // Load project config for per-file target overrides
    let _file_target = load_config()
        .map(|(_, cfg)| cfg.resolve_target(&path).to_string())
        .unwrap_or_else(|| {
            if path_str.ends_with("main.aya") { "executable".into() } else { "static-lib".into() }
        });

    let code = match fs::read_to_string(&path) {
        Ok(c) => c, Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    match ayanami::compiler::build_source_to(&path_str, &code, "build") {
        Ok(()) => {}
        Err(e) => { eprintln!("build failed: {}", e); std::process::exit(1); }
    }
}

fn cmd_run(args: &[String]) {
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();

    let _file_target = load_config()
        .map(|(_, cfg)| cfg.resolve_target(&path).to_string())
        .unwrap_or_else(|| "executable".into());

    let code = match fs::read_to_string(&path) {
        Ok(c) => c, Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    if let Err(e) = ayanami::compiler::build_source_to(&path_str, &code, "build") {
        eprintln!("build failed: {}", e); std::process::exit(1);
    }
    let exe_name = path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy();
    match ayanami::compiler::run_executable(&exe_name) {
        Ok(code) => println!("exit code: {}", code),
        Err(e) => { eprintln!("run failed: {}", e); std::process::exit(1); }
    }
}
