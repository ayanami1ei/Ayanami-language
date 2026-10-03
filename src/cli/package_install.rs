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
    match ayanami::compiler::package_source(&path_str, &code) {
        Ok(()) => {}
        Err(e) => { ayanami::diagnostics::error(&format!("package failed: {}", e)); std::process::exit(1); }
    }
}

pub(crate) fn cmd_install(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami install <package.lcl>");
        std::process::exit(1);
    }
    let lcl_path = &args[0];
    match ayanami::compiler::install_package(lcl_path, None) {
        Ok(()) => {}
        Err(e) => { ayanami::diagnostics::error(&format!("install failed: {}", e)); std::process::exit(1); }
    }
}
