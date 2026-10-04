use super::*;

pub(crate) fn cmd_package(args: &[String]) {
    let (args, flags) = split_flags(args);
    apply_mode(&flags);
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();
    let code = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    let stem = path.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned();
    ayanami::diagnostics::status("Packaging", &stem);
    let started = std::time::Instant::now();
    match ayanami::compiler::package_source(&path_str, &code) {
        Ok(()) => ayanami::diagnostics::status(
            "Finished", &format!("in {:.2}s", started.elapsed().as_secs_f64())),
        Err(e) => { ayanami::diagnostics::report_error(&path, &format!("package failed: {}", e)); std::process::exit(1); }
    }
}

pub(crate) fn cmd_install(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami install <package.lcl>");
        std::process::exit(1);
    }
    let lcl_path = &args[0];
    let stem = std::path::Path::new(lcl_path).file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned();
    ayanami::diagnostics::status("Installing", &stem);
    let started = std::time::Instant::now();
    match ayanami::compiler::install_package(lcl_path, None) {
        Ok(()) => ayanami::diagnostics::status(
            "Finished", &format!("in {:.2}s", started.elapsed().as_secs_f64())),
        Err(e) => { ayanami::diagnostics::error(&format!("install failed: {}", e)); std::process::exit(1); }
    }
}
