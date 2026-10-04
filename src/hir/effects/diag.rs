//! A3：效应诊断（硬性出入定位 + pub 缺失建议）。

use crate::error::{Error, Result};
use std::path::Path;

use super::infer::{collect_fns, verify_effects};
use crate::hir::{HirFn, HirProgram};

pub(super) fn diagnostics(hir: &HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()> {
    let mut fns: Vec<&HirFn> = Vec::new();
    collect_fns(&hir.items, &mut fns);
    let obs = super::scan::collect_ast_observations(ast);
    // (line, col, message)
    let mut issues: Vec<(usize, usize, String)> = Vec::new();

    for f in &fns {
        if f.extern_c {
            continue;
        }
        let full = f.name.as_str();
        let sites = obs.get(&full).cloned().unwrap_or_default();
        let inf = &f.inferred;
        let site_of = |kind: &str| -> Option<(usize, usize)> {
            sites.iter().find_map(|o| o.site_of(kind))
        };

        // 硬性出入（所有函数）
        if f.effects.pure {
            for e in &inf.effects {
                let (l, c) = site_of(e).unwrap_or((f.span.start_line, f.span.start_col));
                issues.push((l, c, format!("function `{}` is #[pure] but has effect `{}` (hard discrepancy)", full, e)));
            }
            if !inf.throws.is_empty() {
                let (l, c) = site_of("throws").unwrap_or((f.span.start_line, f.span.start_col));
                issues.push((l, c, format!("function `{}` is #[pure] but may throw (hard discrepancy)", full)));
            }
            if inf.may_unknown_effects {
                issues.push((f.span.start_line, f.span.start_col, format!("function `{}` is #[pure] but calls into unknown extern code (hard discrepancy)", full)));
            }
        }
        if f.effects.no_error {
            if !inf.throws.is_empty() {
                let (l, c) = site_of("throws").unwrap_or((f.span.start_line, f.span.start_col));
                issues.push((l, c, format!("function `{}` is #[no_error] but may throw (hard discrepancy)", full)));
            }
        }

        // 缺失建议（只对 pub 接口）
        if f.is_pub && !f.effects.pure {
            // 缺失建议锚定到函数声明（编辑器波浪线落在声明上）
            for e in &inf.effects {
                if !f.effects.has_effect(e) {
                    issues.push((f.span.start_line, f.span.start_col,
                        format!("function `{}` may have effect `{}`; consider adding #[{}]", full, e, e)));
                }
            }
            if !inf.throws.is_empty() && f.effects.throws.is_none() {
                issues.push((f.span.start_line, f.span.start_col,
                    format!("function `{}` may throw; consider adding #[throws(...)]", full)));
            }
        }
    }

    if issues.is_empty() {
        return Ok(());
    }
    if verify_effects() {
        let joined: Vec<String> = issues.iter()
            .map(|(l, c, m)| format!("{}:{}:{}: {}", src_path.display(), l, c, m))
            .collect();
        return Err(Error::Compile(format!(
            "{}: effect verification failed:\n{}",
            src_path.display(),
            joined.join("\n")
        )));
    }
    for (l, c, m) in issues {
        crate::diagnostics::warning_at(src_path, l, c, &m);
    }
    Ok(())
}

