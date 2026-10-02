// ============================================================
//  Ayanami 编译器 —— CLI 入口
//  命令行接口，支持以下命令：
//    new     — 创建新项目骨架
//    check   — 前端检查（lex → parse → HIR → MIR → 借用检查）
//    fmt     — 格式化代码
//    package — 打包为 .lcl（不生成可执行文件）
//    build   — 完整构建（生成可执行文件/静态库/动态库）
//    install — 从 .lcl 构建目标产物
//    defs    — 输出符号定义列表（JSON）
//    run     — 构建并运行
//    clean   — 清除构建产物
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ayanami <command> [args...]");
        eprintln!("commands:");
        eprintln!("  new <name>         创建新项目");
        eprintln!("  check [--watch] [--release] <file/proj> 前端检查；--release 关闭运行检查（转 assume）");
        eprintln!("  fmt [file/dir]     格式化代码（类似 rustfmt）");
        eprintln!("  package [--release] <file/proj> 打包为 .lcl（不生成可执行文件）");
        eprintln!("  build [--release] [file/proj] 构建可执行文件 + .lcl 包");
        eprintln!("  install <lcl>      从 .lcl 构建目标产物");
        eprintln!("  defs <file>        输出符号定义列表（JSON 格式）");
        eprintln!("  run [--release] [file/proj] 构建并运行");
        eprintln!("  clean              清除 build/ 目录");
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "new" => cmd_new(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "fmt" => cmd_fmt(&args[2..]),
        "package" => cmd_package(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "install" => cmd_install(&args[2..]),
        "defs" => cmd_defs(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "clean" => cmd_clean(),
        _ => {
            eprintln!("unknown command: {}", command);
            std::process::exit(1);
        }
    }
}

/// 从 `dir` 向上级目录搜索 ayanami.toml 文件，找到项目根目录
/// 返回 (项目目录, toml 内容)
pub(crate) fn find_project(dir: &Path) -> Option<(PathBuf, String)> {
    let mut d = Some(dir);
    while let Some(cur) = d {
        let toml = cur.join("ayanami.toml");
        if toml.exists() {
            if let Ok(content) = fs::read_to_string(&toml) {
                return Some((cur.to_path_buf(), content));
            }
        }
        d = cur.parent();
    }
    None
}

/// 读取项目入口文件（默认 src/main.aya）
pub(crate) fn project_entry(project_dir: &Path) -> Option<PathBuf> {
    let main_aya = project_dir.join("src").join("main.aya");
    if main_aya.exists() { Some(main_aya) } else { None }
}

/// 解析命令行路径参数：
/// - 如果以 `.aya` / `.lcl` 结尾 → 直接作为文件路径
/// - 否则 → 当作项目目录，查找 src/main.aya
/// - 无参数 → 当前目录作为项目目录
pub(crate) fn resolve_path(arg: Option<&str>) -> PathBuf {
    match arg {
        Some(p) if p.ends_with(".aya") || p.ends_with(".lcl") => PathBuf::from(p),
        Some(p) => {
            // Treat as project directory
            let project_dir = Path::new(p);
            if let Some((_, _)) = find_project(project_dir) {
                project_entry(project_dir).unwrap_or_else(|| {
                    eprintln!("error: no src/main.aya in project '{}'", p);
                    std::process::exit(1);
                })
            } else {
                PathBuf::from(p)
            }
        }
        None => {
            // No arg: try current directory as project
            let cwd = std::env::current_dir().unwrap_or_default();
            if let Some((proj_dir, _)) = find_project(&cwd) {
                project_entry(&proj_dir).unwrap_or(cwd.join("main.aya"))
            } else {
                eprintln!("error: not in a project (no ayanami.toml found)");
                std::process::exit(1);
            }
        }
    }
}

mod cli;

pub(crate) use cli::{cmd_build, cmd_check, cmd_clean, cmd_defs, cmd_fmt, cmd_install, cmd_new, cmd_package, cmd_run};

pub(crate) fn load_config() -> Option<(PathBuf, ayanami::package::config::ProjectConfig)> {
    let cwd = std::env::current_dir().ok()?;
    if let Some((proj_dir, toml_str)) = find_project(&cwd) {
        Some((proj_dir, ayanami::package::config::ProjectConfig::load(&toml_str)))
    } else {
        None
    }
}
