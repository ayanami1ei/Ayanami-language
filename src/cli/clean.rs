use super::*;

pub(crate) fn cmd_clean() {
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
