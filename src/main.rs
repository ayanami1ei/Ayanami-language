use std::{env, path::Path, process::Command};

use ayanami::pakager::{Arch, System, Target, UseMode};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: ayanami <check|build|run> <file.aya> <out_dir>");
        eprintln!("       ayanami install <file.lcl> <out_dir>");
        return;
    }

    let cmd = args[1].as_str();
    let file_path = &args[2];
    let out_dir = &args[3];

    match cmd {
        "install" => {
            ayanami::compile(file_path, out_dir);
        }
        "check" => {}
        "build" => {
            let tokens = match ayanami::tokenlize(file_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("tokenlize error: {:?}", e);
                    return;
                }
            };

            let (stmts, symbol_table) = match ayanami::parse(tokens) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("parse error: {:?}", e);
                    return;
                }
            };

            let stmts = match ayanami::type_infer(stmts, symbol_table.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("type infer error: {:?}", e);
                    return;
                }
            };

            ayanami::pakage(
                stmts,
                symbol_table,
                out_dir,
                file_path.to_string(),
                "0.0.0".to_string(),
                UseMode::AsUser,
                Target::default(),
                Arch::default(),
                System::default(),
            );
        }
        "run" => {
            let tokens = match ayanami::tokenlize(file_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("tokenlize error: {:?}", e);
                    return;
                }
            };

            let (stmts, symbol_table) = match ayanami::parse(tokens) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("parse error: {:?}", e);
                    return;
                }
            };

            let stmts = match ayanami::type_infer(stmts, symbol_table.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("type infer error: {:?}", e);
                    return;
                }
            };

            ayanami::pakage(
                stmts,
                symbol_table,
                out_dir,
                file_path.to_string(),
                "0.0.0".to_string(),
                UseMode::AsDeveloper,
                Target::default(),
                Arch::default(),
                System::default(),
            );

            let base_name = Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("ayanami");
            let pak_path = Path::new(out_dir).join(format!("{}.lcl", base_name));
            ayanami::compile(pak_path.to_string_lossy().as_ref(), out_dir);

            let bin_path = Path::new(out_dir).join(base_name);
            let status = Command::new(&bin_path).status();
            if let Err(e) = status {
                eprintln!("run failed: {}", e);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
        }
    }
}
