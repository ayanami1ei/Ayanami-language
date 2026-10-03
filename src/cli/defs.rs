use super::*;

pub(crate) fn cmd_defs(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: ayanami defs <file.aya>");
        std::process::exit(1);
    }
    let path = std::path::Path::new(&args[0]);
    let path_str = path.to_string_lossy().into_owned();
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    let mut lexer = ayanami::lexer::Lexer::new(&code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, ayanami::lexer::TokenKind::EOF))
        .collect();
    let mut parser = ayanami::parser::Parser::new(filtered);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => { eprintln!("parse error: {}", e); std::process::exit(1); }
    };
    let mut defs = Vec::new();
    ayanami::compiler::collect_defs_from_stmts(&program.stmts, &path_str, "", &mut defs);
    // A3：尽力接 HIR 推断摘要（失败则只保留声明 tokens）
    if let Ok(hir) = ayanami::hir::lower_program(&program) {
        let summaries = ayanami::hir::effects::infer::summarize(&hir);
        for d in defs.iter_mut() {
            if let Some(s) = summaries.get(&d.name) {
                d.effects = s.tokens();
            }
        }
    }
    // Also collect from recursively resolved .aya imports
    let mut visited = std::collections::HashSet::new();
    visited.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    resolve_import_defs(&program.stmts, &path_str, &mut visited, &mut defs);
    print!("{}", ayanami::compiler::defs_to_json(&defs));
}

/// Recursively collect defs from imported .aya files.
fn resolve_import_defs(stmts: &[ayanami::parser::ast::Stmt], base_path: &str, visited: &mut std::collections::HashSet<std::path::PathBuf>, defs: &mut Vec<ayanami::compiler::SymDef>) {
    let base_dir = std::path::Path::new(base_path).parent().unwrap_or(std::path::Path::new("."));
    for stmt in stmts {
        if let ayanami::parser::ast::Stmt::Import { path, .. } = stmt {
            let dep = base_dir.join(path);
            let dep_aya = if dep.to_string_lossy().ends_with(".aya") {
                dep
            } else {
                let with_ext = dep.with_extension("aya");
                if with_ext.exists() { with_ext } else { continue; }
            };
            let canonical = match dep_aya.canonicalize() {
                Ok(c) => c,
                Err(_) => continue,
            };
            if !visited.insert(canonical.clone()) { continue; }
            if !canonical.to_string_lossy().ends_with(".aya") { continue; }
            let code = match std::fs::read_to_string(&canonical) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut lexer = ayanami::lexer::Lexer::new(&code);
            let tokens = lexer.tokenize_all();
            let filtered: Vec<_> = tokens.into_iter()
                .filter(|t| !matches!(t.kind, ayanami::lexer::TokenKind::EOF))
                .collect();
            let mut parser = ayanami::parser::Parser::new(filtered);
            if let Ok(prog) = parser.parse_program() {
                let dep_str = canonical.to_string_lossy().into_owned();
                ayanami::compiler::collect_defs_from_stmts(&prog.stmts, &dep_str, "", defs);
                resolve_import_defs(&prog.stmts, &dep_str, visited, defs);
            }
        }
    }
}
