use super::*;

fn do_check(path: &Path) -> ayanami::error::Result<()> {
    let tmp_dir = std::env::temp_dir().join("ayanami-check");
    std::fs::create_dir_all(&tmp_dir).ok();
    let mut compiling = std::collections::HashSet::new();
    let mut cache = std::collections::HashMap::new();
    let base = path.parent().unwrap_or(std::path::Path::new("."));
    let result = ayanami::compiler::compile_file(path, base, &tmp_dir, &mut compiling, &mut cache, None);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map(|_| ())
}

pub(crate) fn cmd_check(args: &[String]) {
    let (args, release) = split_release(args);
    apply_mode(release);
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
