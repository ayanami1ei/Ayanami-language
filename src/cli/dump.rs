//! `ayanami dump <ast|expanded|macros|hir|mir|lir|llvm> <file.aya>` — 输出各阶段 IR 文本（回归快照用）。
//! `expanded`：宏展开后的源码（mcl.3 调试视图）。
use super::*;

pub(crate) fn cmd_dump(args: &[String]) {
    if args.len() < 2 {
        eprintln!("usage: ayanami dump <ast|expanded|macros|hir|mir|lir|llvm> <file.aya>");
        std::process::exit(1);
    }
    let stage = args[0].as_str();
    let path = resolve_path(Some(args[1].as_str()));
    let _ = configure_target_for(&path);
    let _code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => { eprintln!("error: failed to read '{}': {}", path.display(), e); std::process::exit(1); }
    };
    let tmp = unique_tmp_dir("dump");
    fs::create_dir_all(&tmp).ok();
    let base = path.parent().unwrap_or(std::path::Path::new("."));
    let mut compiling = std::collections::HashSet::new();
    let mut cache = std::collections::HashMap::new();
    if stage == "macros" {
        ayanami::compiler::macro_expand::begin_trace();
    }
    let cf = match ayanami::compiler::compile_file(&path, base, &tmp, &mut compiling, &mut cache, None, None) {
        Ok(c) => c,
        Err(e) => {
            ayanami::diagnostics::report_error(&path, &format!("dump failed: {}", e));
            std::process::exit(1);
        }
    };
    let visible_program = without_scope_items(&cf.program);
    let expanded = || -> std::result::Result<ayanami::parser::ast::Program, String> {
        ayanami::compiler::macro_expand::expand(&visible_program, &path).map_err(|e| e.to_string())
    };
    let out = match stage {
        "ast" => ayanami::compiler::format_program(&visible_program),
        // mcl.3：宏展开后的源码（调试/IDE 视图）
        "expanded" => match expanded() {
            Ok(p) => ayanami::formatter::format_program(&p),
            Err(e) => { ayanami::diagnostics::error(&format!("dump expanded failed: {}", e)); std::process::exit(1); }
        },
        "macros" => match ayanami::compiler::macro_expand::finish_trace() {
            Ok(json) => json,
            Err(e) => { ayanami::diagnostics::error(&format!("dump macros failed: {}", e)); std::process::exit(1); }
        },
        "hir" => match expanded().and_then(|p| lower_visible_hir(&p)) {
            Ok(hir) => ayanami::hir::hir_program_to_string(&hir),
            Err(e) => { ayanami::diagnostics::error(&format!("dump hir failed: {}", e)); std::process::exit(1); }
        },
        "mir" => match expanded()
            .and_then(|p| lower_visible_hir(&p))
            .and_then(|hir| lower_visible_mir(&hir))
        {
            Ok(mir) => ayanami::mir::mir_program_to_string(&mir),
            Err(e) => { ayanami::diagnostics::error(&format!("dump mir failed: {}", e)); std::process::exit(1); }
        },
        "lir" => {
            let mut lir = cf.lir_program.clone();
            lir.functions.retain(|f| !is_scope_name(&f.name));
            ayanami::lir::lir_program_to_string(&lir)
        }
        "llvm" => cf.llvm_ir.clone(),
        _ => { eprintln!("error: unknown stage `{}` (ast|expanded|macros|hir|mir|lir|llvm)", stage); std::process::exit(1); }
    };
    print!("{}", out);
    let _ = fs::remove_dir_all(&tmp);
}

fn is_scope_name(name: &ayanami::intern::Symbol) -> bool {
    name.as_str().starts_with(ayanami::package::SCOPE_PREFIX)
}

fn without_scope_items(program: &ayanami::parser::ast::Program) -> ayanami::parser::ast::Program {
    let mut visible = program.clone();
    visible.stmts.retain(|stmt| {
        !matches!(
            stmt,
            ayanami::parser::ast::Stmt::Namespace { name, .. } if is_scope_name(name)
        )
    });
    visible
}

fn lower_visible_hir(
    program: &ayanami::parser::ast::Program,
) -> std::result::Result<ayanami::hir::HirProgram, String> {
    let mut hir = ayanami::hir::lower_program(program).map_err(|e| e.to_string())?;
    hir.items.retain(|item| {
        !matches!(item, ayanami::hir::HirItem::Namespace { name, .. } if is_scope_name(name))
    });
    Ok(hir)
}

fn lower_visible_mir(
    hir: &ayanami::hir::HirProgram,
) -> std::result::Result<ayanami::mir::MirProgram, String> {
    let mut mir = ayanami::mir::lower_program(hir).map_err(|e| e.to_string())?;
    mir.items.retain(|item| {
        !matches!(item, ayanami::mir::MirItem::Namespace { name, .. } if is_scope_name(name))
    });
    Ok(mir)
}
