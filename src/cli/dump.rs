//! `ayanami dump <ast|hir|mir|lir|llvm> <file.aya>` — 输出各阶段 IR 文本（回归快照用）。
use super::*;

pub(crate) fn cmd_dump(args: &[String]) {
    if args.len() < 2 {
        eprintln!("usage: ayanami dump <ast|hir|mir|lir|llvm> <file.aya>");
        std::process::exit(1);
    }
    let stage = args[0].as_str();
    let path = resolve_path(Some(args[1].as_str()));
    let _code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    let tmp = std::env::temp_dir().join("ayanami-dump");
    fs::create_dir_all(&tmp).ok();
    let base = path.parent().unwrap_or(std::path::Path::new("."));
    let mut compiling = std::collections::HashSet::new();
    let mut cache = std::collections::HashMap::new();
    let cf = match ayanami::compiler::compile_file(&path, base, &tmp, &mut compiling, &mut cache, None, None) {
        Ok(c) => c,
        Err(e) => {
            ayanami::diagnostics::report_error(&path, &format!("dump failed: {}", e));
            std::process::exit(1);
        }
    };
    let expanded = || -> std::result::Result<ayanami::parser::ast::Program, String> {
        ayanami::compiler::macro_expand::expand(&cf.program, &path).map_err(|e| e.to_string())
    };
    let out = match stage {
        "ast" => ayanami::compiler::format_program(&cf.program),
        "hir" => match expanded().and_then(|p| ayanami::hir::lower_program(&p).map_err(|e| e.to_string())) {
            Ok(hir) => ayanami::hir::hir_program_to_string(&hir),
            Err(e) => { ayanami::diagnostics::error(&format!("dump hir failed: {}", e)); std::process::exit(1); }
        },
        "mir" => match expanded()
            .and_then(|p| ayanami::hir::lower_program(&p).map_err(|e| e.to_string()))
            .and_then(|hir| ayanami::mir::lower_program(&hir).map_err(|e| e.to_string()))
        {
            Ok(mir) => ayanami::mir::mir_program_to_string(&mir),
            Err(e) => { ayanami::diagnostics::error(&format!("dump mir failed: {}", e)); std::process::exit(1); }
        },
        "lir" => ayanami::lir::lir_program_to_string(&cf.lir_program),
        "llvm" => cf.llvm_ir.clone(),
        _ => { eprintln!("error: unknown stage `{}` (ast|hir|mir|lir|llvm)", stage); std::process::exit(1); }
    };
    print!("{}", out);
    let _ = fs::remove_dir_all(&tmp);
}
