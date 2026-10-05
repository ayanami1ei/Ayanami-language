//! `ayanami types <file.aya>` — 输出变量类型（JSON），供编辑器 inlay hints 使用。
//! 类型来自 HIR 推断结果；位置为首次赋值处（语句 span 即变量名位置）。

use super::*;
use ayanami::hir::ty::HirType;
use ayanami::hir::{HirFn, HirItem, HirStmt};
use ayanami::intern::Symbol;

pub(crate) fn cmd_types(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami types <file.aya>");
        std::process::exit(1);
    }
    let path = std::path::Path::new(&args[0]);
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path.display(), e);
            print!("{{\"types\":[]}}");
            return;
        }
    };
    let mut lexer = ayanami::lexer::Lexer::new(&code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, ayanami::lexer::TokenKind::EOF))
        .collect();
    let mut parser = ayanami::parser::Parser::new(filtered);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: parse error: {}", e);
            print!("{{\"types\":[]}}");
            return;
        }
    };
    let hir = match ayanami::hir::lower_program(&program) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error: {}", e);
            print!("{{\"types\":[]}}");
            return;
        }
    };
    let mut entries: Vec<(String, String, usize, usize)> = Vec::new();
    collect_items(&hir.items, &mut entries);
    // 只保留“位置处确实是该变量名”的条目：过滤 for 迭代变量与单态化体里的合成局部
    entries.retain(|(name, _, line, col)| name_at(&code, *line, *col, name));

    let mut out = String::from("{\"types\":[");
    for (i, (name, ty, line, col)) in entries.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"type\":\"{}\",\"line\":{},\"col\":{}}}",
            esc(name), esc(ty), line, col
        ));
    }
    out.push_str("]}");
    println!("{}", out);
}

fn collect_items(items: &[HirItem], out: &mut Vec<(String, String, usize, usize)>) {
    for item in items {
        match item {
            HirItem::Fn(f) => collect_fn(f, out),
            HirItem::Namespace { items, .. } => collect_items(items, out),
            _ => {}
        }
    }
}

fn collect_fn(f: &HirFn, out: &mut Vec<(String, String, usize, usize)>) {
    let params: std::collections::HashSet<Symbol> = f.params.iter().map(|(n, _)| *n).collect();
    let mut seen: std::collections::HashSet<ayanami::hir::ty::VarId> = std::collections::HashSet::new();
    walk_stmts(&f.body.stmts, f, &params, &mut seen, out);
}

fn walk_stmts(
    stmts: &[HirStmt],
    f: &HirFn,
    params: &std::collections::HashSet<Symbol>,
    seen: &mut std::collections::HashSet<ayanami::hir::ty::VarId>,
    out: &mut Vec<(String, String, usize, usize)>,
) {
    for s in stmts {
        match s {
            HirStmt::Assign { target, span, .. } => {
                if let Some(v) = target.as_local() {
                    if seen.insert(v) {
                        if let Some(local) = f.locals.get(v.0) {
                            let name = local.name.as_str();
                            if !params.contains(&local.name)
                                && !name.starts_with("__")
                                && span.start_line > 0
                            {
                                out.push((
                                    name.to_string(),
                                    fmt_type(&local.ty),
                                    span.start_line,
                                    span.start_col,
                                ));
                            }
                        }
                    }
                }
            }
            HirStmt::If { then_block, elifs, else_block, .. } => {
                walk_stmts(&then_block.stmts, f, params, seen, out);
                for (_, b) in elifs {
                    walk_stmts(&b.stmts, f, params, seen, out);
                }
                if let Some(b) = else_block {
                    walk_stmts(&b.stmts, f, params, seen, out);
                }
            }
            HirStmt::While { body, .. } => walk_stmts(&body.stmts, f, params, seen, out),
            HirStmt::Block { stmts, .. } => walk_stmts(stmts, f, params, seen, out),
            _ => {}
        }
    }
}

/// 与 HIR 展示一致的可读类型名（`ArrayList<int>` / `ref String` / `[int]`）。
fn fmt_type(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::F32 => "f32".into(),
        HirType::Char => "char".into(),
        HirType::Bool => "bool".into(),
        HirType::Void => "void".into(),
        HirType::IntN { bits, signed } => format!("{}{}", if *signed { "i" } else { "u" }, bits),
        HirType::Named(n) => n.as_str().to_string(),
        HirType::Ref(inner, mutable) => {
            format!("ref{}{}", if *mutable { " mut" } else { "" }, fmt_type(inner))
        }
        HirType::Unique(inner) => fmt_type(inner),
        HirType::FatPtr { name, .. } => format!("ref {}", name.as_str()),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => format!("[{}]", fmt_type(inner)),
        HirType::FnPtr(..) => "fn(...)".into(),
    }
}

/// 源码 `line:col`（1-based，字节列）处是否正好是 `name` 标识符。
fn name_at(code: &str, line: usize, col: usize, name: &str) -> bool {
    let Some(line_text) = code.lines().nth(line.saturating_sub(1)) else {
        return false;
    };
    let idx = col.saturating_sub(1);
    let Some(rest) = line_text.get(idx..) else {
        return false;
    };
    if !rest.starts_with(name) {
        return false;
    }
    match rest[name.len()..].chars().next() {
        Some(c) => !(c.is_alphanumeric() || c == '_'),
        None => true,
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
