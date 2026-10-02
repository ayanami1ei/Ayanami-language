use super::*;

pub(crate) fn cmd_build(args: &[String]) {
    let (args, flags) = split_flags(args);
    apply_mode(&flags);
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

pub(crate) fn cmd_run(args: &[String]) {
    let (args, flags) = split_flags(args);
    apply_mode(&flags);
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
