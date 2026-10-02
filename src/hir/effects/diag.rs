//! A3：效应诊断（硬性出入定位 + pub 缺失建议）。

use crate::error::{Error, Result};
use std::path::Path;

use super::infer::{collect_fns, verify_effects};
use crate::hir::{HirFn, HirProgram};

pub(super) fn diagnostics(hir: &HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()> {
    let mut fns: Vec<&HirFn> = Vec::new();
    collect_fns(&hir.items, &mut fns);
    let obs = super::scan::collect_ast_observations(ast);
    let mut issues: Vec<String> = Vec::new();

    for f in &fns {
        if f.extern_c {
            continue;
        }
        let full = f.name.as_str();
        let sites = obs.get(&full).cloned().unwrap_or_default();
        let inf = &f.inferred;
        let loc = |l: usize, c: usize| format!("{}:{}:{}", src_path.display(), l, c);
        let site_of = |kind: &str| -> Option<(usize, usize)> {
            sites.iter().find_map(|o| o.site_of(kind))
        };

        // 硬性出入（所有函数）
        if f.effects.pure {
            for e in &inf.effects {
                let (l, c) = site_of(e).unwrap_or((f.span.start_line, f.span.start_col));
                issues.push(format!("{}: warning: function `{}` is #[pure] but has effect `{}` (hard discrepancy)", loc(l, c), full, e));
            }
            if !inf.throws.is_empty() {
                let (l, c) = site_of("throws").unwrap_or((f.span.start_line, f.span.start_col));
                issues.push(format!("{}: warning: function `{}` is #[pure] but may throw (hard discrepancy)", loc(l, c), full));
            }
            if inf.may_unknown_effects {
                issues.push(format!("{}: warning: function `{}` is #[pure] but calls into unknown extern code (hard discrepancy)", loc(f.span.start_line, f.span.start_col), full));
            }
        }
        if f.effects.no_error {
            if !inf.throws.is_empty() {
                let (l, c) = site_of("throws").unwrap_or((f.span.start_line, f.span.start_col));
                issues.push(format!("{}: warning: function `{}` is #[no_error] but may throw (hard discrepancy)", loc(l, c), full));
            }
        }

        // 缺失建议（只对 pub 接口）
        if f.is_pub && !f.effects.pure {
            for e in &inf.effects {
                if !f.effects.has_effect(e) {
                    let (l, c) = site_of(e).unwrap_or((f.span.start_line, f.span.start_col));
                    issues.push(format!("{}: warning: function `{}` may have effect `{}`; consider adding #[{}]", loc(l, c), full, e, e));
                }
            }
            if !inf.throws.is_empty() && f.effects.throws.is_none() {
                let (l, c) = site_of("throws").unwrap_or((f.span.start_line, f.span.start_col));
                issues.push(format!("{}: warning: function `{}` may throw; consider adding #[throws(...)]", loc(l, c), full));
            }
        }
    }

    if issues.is_empty() {
        return Ok(());
    }
    if verify_effects() {
        return Err(Error::Compile(format!(
            "{}: effect verification failed:\n{}",
            src_path.display(),
            issues.join("\n")
        )));
    }
    for m in issues {
        eprintln!("warning: {}", m);
    }
    Ok(())
}

