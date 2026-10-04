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
    let stem = path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned();
    ayanami::diagnostics::status("Compiling", &stem);
    let started = std::time::Instant::now();
    match ayanami::compiler::build_source_with_target(&path_str, &code, "build", Some(&target)) {
        Ok(()) => ayanami::diagnostics::status(
            "Finished", &format!("in {:.2}s", started.elapsed().as_secs_f64())),
        Err(e) => { ayanami::diagnostics::report_error(&path, &format!("build failed: {}", e)); std::process::exit(1); }
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
    let stem = path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned();
    ayanami::diagnostics::status("Compiling", &stem);
    let started = std::time::Instant::now();
    if let Err(e) = ayanami::compiler::build_source_with_target(&path_str, &code, "build", Some(target)) {
        ayanami::diagnostics::report_error(&path, &format!("build failed: {}", e)); std::process::exit(1);
    }
    ayanami::diagnostics::status("Finished", &format!("in {:.2}s", started.elapsed().as_secs_f64()));
    let exe_name = stem.clone();
    let shown = if std::path::Path::new(&format!("build/{}", exe_name)).exists() {
        format!("build/{}", exe_name)
    } else {
        exe_name.clone()
    };
    ayanami::diagnostics::status("Running", &format!("`{}`", shown));
    match ayanami::compiler::run_executable(&exe_name) {
        Ok(0) => {} // 成功不打印退出码
        Ok(code) => {
            ayanami::diagnostics::error(&format!(
                "process didn't exit successfully: `{}` (exit code: {})", shown, code
            ));
            std::process::exit(code);
        }
        Err(e) => { ayanami::diagnostics::error(&format!("run failed: {}", e)); std::process::exit(1); }
    }
}
