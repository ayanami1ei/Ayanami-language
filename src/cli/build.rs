use super::*;

/// 从源文件向上找 ayanami.toml（优先于 cwd），用于 arch 等构建参数。
fn project_config_for(path: &Path) -> Option<ayanami::package::config::ProjectConfig> {
    let mut d = path.parent();
    while let Some(cur) = d {
        if let Ok(s) = fs::read_to_string(cur.join("ayanami.toml")) {
            return Some(ayanami::package::config::ProjectConfig::load(&s));
        }
        d = cur.parent();
    }
    load_config().map(|(_, c)| c)
}

pub(crate) fn cmd_build(args: &[String]) {
    let (args, flags) = split_flags(args);
    apply_mode(&flags);
    let path = resolve_path(args.first().map(|s| s.as_str()));
    let path_str = path.to_string_lossy().into_owned();

    let cfg = project_config_for(&path);
    let target = cfg.as_ref()
        .map(|c| c.resolve_target(&path).to_string())
        .unwrap_or_else(|| {
            if path_str.ends_with("main.aya") { "executable".into() } else { "static-lib".into() }
        });
    let arch = cfg.as_ref().and_then(|c| c.arch.clone());
    let freestanding = cfg.as_ref().map_or(false, |c| c.freestanding);
    let target = if freestanding { "object".to_string() } else { target };
    ayanami::target::set_target_triple(arch.as_deref());
    ayanami::target::set_freestanding(freestanding);

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
    let cfg = project_config_for(&path);
    if cfg.as_ref().map_or(false, |c| c.freestanding) {
        eprintln!("error: freestanding mode only produces object files; cannot run");
        std::process::exit(1);
    }
    let arch = cfg.as_ref().and_then(|c| c.arch.clone());
    if let Some(a) = &arch {
        if ayanami::target::is_cross_target(a) {
            eprintln!("error: cannot run cross target `{}` on host {}; use `ayanami build` and a loader/emulator", a, std::env::consts::ARCH);
            std::process::exit(1);
        }
    }
    ayanami::target::set_target_triple(arch.as_deref());

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
