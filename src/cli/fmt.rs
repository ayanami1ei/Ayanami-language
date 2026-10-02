use super::*;

pub(crate) fn cmd_fmt(args: &[String]) {
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
