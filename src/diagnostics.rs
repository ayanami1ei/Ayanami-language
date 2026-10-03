//! 终端诊断高亮（尊重 `NO_COLOR` 与非 TTY 环境）。
//!
//! 错误红色、警告黄色、成功绿色；`warning_at` 额外打印源码行与插入符。

use std::io::IsTerminal;
use std::path::Path;

use owo_colors::OwoColorize;

pub fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal()
}

pub fn error(msg: &str) {
    if supports_color() {
        eprintln!("{} {}", "error:".red().bold(), msg);
    } else {
        eprintln!("error: {}", msg);
    }
}

pub fn warning(msg: &str) {
    if supports_color() {
        eprintln!("{} {}", "warning:".yellow().bold(), msg);
    } else {
        eprintln!("warning: {}", msg);
    }
}

/// 成功信息（stdout，保持可管道）。
pub fn success(msg: &str) {
    if supports_color() {
        println!("{}", msg.green());
    } else {
        println!("{}", msg);
    }
}

/// 带源码片段与插入符的警告（A3 效应/契约诊断）。
pub fn warning_at(file: &Path, line: usize, col: usize, msg: &str) {
    warning(msg);
    if supports_color() {
        eprintln!("  {} {}:{}:{}", "-->".blue().bold(), file.display(), line, col);
    } else {
        eprintln!("  --> {}:{}:{}", file.display(), line, col);
    }
    let Ok(text) = std::fs::read_to_string(file) else { return };
    let Some(src) = text.lines().nth(line.saturating_sub(1)) else { return };
    let gutter = line.to_string();
    let pad = " ".repeat(gutter.len());
    if supports_color() {
        eprintln!("{} {} {}", pad.blue().bold(), "|".blue().bold(), "");
        eprintln!("{} {} {}", gutter.blue().bold(), "|".blue().bold(), src);
        let caret = format!("{}{}", " ".repeat(col.saturating_sub(1)), "^");
        eprintln!("{} {} {}", pad.blue().bold(), "|".blue().bold(), caret.red().bold());
    } else {
        eprintln!("{} |", pad);
        eprintln!("{} | {}", gutter, src);
        eprintln!("{} | {}{}", pad, " ".repeat(col.saturating_sub(1)), "^");
    }
}
