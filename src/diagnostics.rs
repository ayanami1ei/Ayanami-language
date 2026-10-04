//! 终端诊断高亮（尊重 `NO_COLOR` 与非 TTY 环境）。
//!
//! 错误红色、警告黄色、成功绿色；`error_at`/`warning_at` 额外打印
//! `--> file:line:col` 与源码行/插入符；`report_error` 从错误文本提取位置。

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

enum Sev {
    Error,
    Warning,
}

fn at_location(sev: Sev, file: &Path, line: usize, col: usize, msg: &str) {
    let is_error = matches!(sev, Sev::Error);
    if is_error {
        error(msg);
    } else {
        warning(msg);
    }
    if line == 0 {
        return; // 无有效位置（如合成函数）
    }
    if supports_color() {
        eprintln!("  {} {}:{}:{}", "-->".blue().bold(), file.display(), line, col);
    } else {
        eprintln!("  --> {}:{}:{}", file.display(), line, col);
    }
    let Ok(text) = std::fs::read_to_string(file) else { return };
    let Some(src) = text.lines().nth(line.saturating_sub(1)) else { return };
    let gutter = line.to_string();
    let pad = " ".repeat(gutter.len());
    let caret = format!("{}{}", " ".repeat(col.saturating_sub(1)), "^");
    if supports_color() {
        eprintln!("{} {} {}", pad.blue().bold(), "|".blue().bold(), "");
        eprintln!("{} {} {}", gutter.blue().bold(), "|".blue().bold(), src);
        if is_error {
            eprintln!("{} {} {}", pad.blue().bold(), "|".blue().bold(), caret.red().bold());
        } else {
            eprintln!("{} {} {}", pad.blue().bold(), "|".blue().bold(), caret.yellow().bold());
        }
    } else {
        eprintln!("{} |", pad);
        eprintln!("{} | {}", gutter, src);
        eprintln!("{} | {}", pad, caret);
    }
}

/// 带源码片段与插入符的错误。
pub fn error_at(file: &Path, line: usize, col: usize, msg: &str) {
    at_location(Sev::Error, file, line, col, msg);
}

/// 带源码片段与插入符的警告（A3 效应/契约诊断）。
pub fn warning_at(file: &Path, line: usize, col: usize, msg: &str) {
    at_location(Sev::Warning, file, line, col, msg);
}

/// 错误文本 → (file, line, col, 清洗后的消息)
fn parse_error(raw: &str) -> (Option<String>, usize, usize, String) {
    let mut msg = raw.trim().to_string();
    let mut file: Option<String> = None;
    let mut line = 0usize;
    let mut col = 0usize;

    // 文件：第一个包含 ".aya:" 的路径 token
    if let Some(pos) = msg.find(".aya:") {
        let end = pos + 4;
        let start = msg[..pos]
            .rfind(|c: char| c.is_whitespace() || c == '(')
            .map(|i| i + 1)
            .unwrap_or(0);
        file = Some(msg[start..end].to_string());
    }

    // (at L:C)
    if let Some(pos) = msg.find("(at ") {
        if let Some(rel_end) = msg[pos..].find(')') {
            let inner = msg[pos + 4..pos + rel_end].to_string();
            if let Some((l, c)) = parse_lc(&inner) {
                line = l;
                col = c;
            }
            msg.replace_range(pos..pos + rel_end + 1, "");
        }
    }
    // 尾部 at L:C
    if line == 0 {
        if let Some(pos) = msg.rfind(" at ") {
            if let Some((l, c)) = parse_lc(msg[pos + 4..].trim()) {
                line = l;
                col = c;
                msg.truncate(pos);
            }
        }
    }
    // path:L:C
    if line == 0 {
        if let Some(pos) = msg.find(".aya:") {
            let rest = &msg[pos + 5..];
            let end = rest.find(|c: char| !c.is_ascii_digit() && c != ':').unwrap_or(rest.len());
            if let Some((l, c)) = parse_lc(&rest[..end]) {
                line = l;
                col = c;
            }
        }
    }

    (file, line, col, clean_error_message(&msg))
}

fn parse_lc(s: &str) -> Option<(usize, usize)> {
    let (l, c) = s.trim().split_once(':')?;
    Some((l.trim().parse().ok()?, c.trim().parse().ok()?))
}

fn clean_error_message(msg: &str) -> String {
    let prefixes = [
        "check failed: ", "build failed: ", "run failed: ", "package failed: ",
        "install failed: ", "fmt error: ", "compile error: ", "hir error: ",
        "mir error: ", "lir error: ", "borrow error: ", "parse error: ",
        "import error: ", "driver error: ", "package error: ", "serialize error: ",
        "error: ",
    ];
    let mut m = msg.trim().to_string();
    let mut changed = true;
    while changed {
        changed = false;
        for p in prefixes {
            if let Some(rest) = m.strip_prefix(p) {
                m = rest.trim().to_string();
                changed = true;
            }
        }
        // 前导文件路径（/path/x.aya: ）
        if let Some(pos) = m.find(".aya:") {
            let start = m[..pos].rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
            if start == 0 {
                m = m[pos + 5..].trim().to_string();
                changed = true;
            }
        }
    }
    m
}

/// 报告编译错误：自动提取 `(at l:c)` / `at l:c` / `path:l:c` 并打印 `-->` 定位；
/// 无法定位时退化为普通 `error:` 输出。
pub fn report_error(default_file: &Path, raw: &str) {
    let (file, line, col, msg) = parse_error(raw);
    if line == 0 {
        error(&msg);
        return;
    }
    let f = file.unwrap_or_else(|| default_file.display().to_string());
    error_at(Path::new(&f), line, col, &msg);
}
