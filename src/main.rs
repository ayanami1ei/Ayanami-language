use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ayanami <command> [args...]");
        eprintln!("commands:");
        eprintln!("  new <name>     create a new project");
        eprintln!("  check <file>   frontend checks (lex, parse, HIR)");
        eprintln!("  build <file>   build to executable + .lcl package");
        eprintln!("  run <file>     build and run");
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "new" => cmd_new(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "run" => cmd_run(&args[2..]),
        _ => {
            eprintln!("unknown command: {}", command);
            std::process::exit(1);
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
    println!("created project '{}'", name);
    println!("  {}/", name);
    println!("  └── src/");
    println!("       └── main.aya");
}

fn cmd_check(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami check <file.aya>");
        std::process::exit(1);
    }
    let path = &args[0];
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path, e);
            std::process::exit(1);
        }
    };
    match ayanami::compiler::check_source(&code, "check_output") {
        Ok(()) => println!("check passed: {}", path),
        Err(e) => {
            eprintln!("check failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_build(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami build <file.aya>");
        std::process::exit(1);
    }
    let path = &args[0];
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path, e);
            std::process::exit(1);
        }
    };
    match ayanami::compiler::build_source(path, &code) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("build failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami run <file.aya>");
        std::process::exit(1);
    }
    let path = &args[0];
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = ayanami::compiler::build_source(path, &code) {
        eprintln!("build failed: {}", e);
        std::process::exit(1);
    }
    let exe_name = Path::new(path).file_stem()
        .unwrap_or(std::ffi::OsStr::new("a"))
        .to_string_lossy();
    let exe_path = format!("./{}", exe_name);
    println!("running: {}", exe_path);
    match ayanami::compiler::run_executable(&exe_path) {
        Ok(code) => println!("exit code: {}", code),
        Err(e) => {
            eprintln!("run failed: {}", e);
            std::process::exit(1);
        }
    }
}
