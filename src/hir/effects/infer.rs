//! A3a-2：效应推断（io/throws）与诊断。

// ═══════════════════════════════════════════════════════════════════
//  A3a-2：效应推断（辅助）与诊断
// ═══════════════════════════════════════════════════════════════════

use crate::error::{Error, Result};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::hir::{HirFn, HirItem, HirNode, HirProgram, HirStmt};
use super::{EffectDecl, IO_NAMES};

/// 推断出的可能效应集合（advisory；`state`/`alloc` 暂不推断）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectSet {
    pub throws: BTreeSet<String>,
    pub io: bool,
}

impl EffectSet {
    fn union(&mut self, o: &EffectSet) {
        self.io |= o.io;
        self.throws.extend(o.throws.iter().cloned());
    }
}

/// `--verify-effects`：把提醒升级为错误。
static VERIFY_EFFECTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_verify_effects(v: bool) {
    VERIFY_EFFECTS.store(v, std::sync::atomic::Ordering::Relaxed);
}

pub fn verify_effects() -> bool {
    VERIFY_EFFECTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// 对 HIR 做效应推断并产生提醒/出入定位（不改变 codegen）。
pub fn check_effects(hir: &HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()> {
    let mut fns: Vec<&HirFn> = Vec::new();
    collect_fns(&hir.items, &mut fns);
    if fns.is_empty() {
        return Ok(());
    }

    let mut names: HashMap<crate::hir::ty::FnId, String> = HashMap::new();
    for f in &fns {
        names.insert(f.fn_id, f.name.as_str());
    }
    for imp in &hir.imported_fns {
        names.insert(imp.fn_id, imp.name.as_str());
    }

    // 无体函数（extern/导入）用声明作为已知集合
    let mut known: HashMap<crate::hir::ty::FnId, EffectSet> = HashMap::new();
    for f in &fns {
        if f.extern_c || f.body.stmts.is_empty() {
            known.insert(f.fn_id, declared_to_set(&f.effects));
        }
    }

    // 不动点推断有体函数（单调增长，有限步收敛）
    let mut inferred: HashMap<crate::hir::ty::FnId, EffectSet> = HashMap::new();
    for _ in 0..=fns.len() {
        let mut changed = false;
        for f in &fns {
            if f.extern_c || f.body.stmts.is_empty() {
                continue;
            }
            let mut calls = Vec::new();
            walk_stmts(&f.body.stmts, &mut calls);
            let mut set = EffectSet::default();
            for id in calls {
                let name = names.get(&id).cloned().unwrap_or_default();
                if name.starts_with("try_unwrap") {
                    set.throws.insert("?".to_string());
                }
                if IO_NAMES.contains(&name.as_str()) {
                    set.io = true;
                }
                if let Some(callee) = inferred.get(&id).or_else(|| known.get(&id)) {
                    set.union(callee);
                }
            }
            if inferred.get(&f.fn_id) != Some(&set) {
                inferred.insert(f.fn_id, set);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // AST 定位：精确到调用点/`?` 的行列
    let obs = super::scan::collect_ast_observations(ast);
    let mut issues: Vec<String> = Vec::new();
    for f in &fns {
        if f.extern_c {
            continue;
        }
        let actual = inferred.get(&f.fn_id).cloned().unwrap_or_default();
        let full = f.name.as_str();
        let sites = obs.get(&full).cloned().unwrap_or_default();
        let loc = |s: Option<(usize, usize)>| {
            s.map(|(l, c)| format!("{}:{}:{}", src_path.display(), l, c))
                .unwrap_or_else(|| format!("{}:{}:{}", src_path.display(), f.span.start_line, f.span.start_col))
        };

        // io
        if actual.io {
            let site = sites.iter().find_map(|o| match o {
                super::scan::Obs::Io(name, l, c) => Some((name.clone(), *l, *c)),
                _ => None,
            });
            match &f.effects.effs {
                None => issues.push(format!(
                    "{}: warning: function `{}` may have effect `io` (call to `{}`); consider adding #[eff(io)]",
                    loc(site.as_ref().map(|(_, l, c)| (*l, *c))),
                    full,
                    site.as_ref().map(|(n, _, _)| n.as_str()).unwrap_or("?"),
                )),
                Some(v) if !v.iter().any(|e| e.as_str() == "io") => issues.push(format!(
                    "{}: warning: function `{}` declares #[eff] without `io` but calls `{}` (hard discrepancy)",
                    loc(site.as_ref().map(|(_, l, c)| (*l, *c))),
                    full,
                    site.as_ref().map(|(n, _, _)| n.as_str()).unwrap_or("?"),
                )),
                _ => {}
            }
        }

        // throws
        if !actual.throws.is_empty() {
            let site = sites.iter().find_map(|o| match o {
                super::scan::Obs::Try(l, c) => Some((*l, *c)),
                _ => None,
            });
            match &f.effects.throws {
                None => issues.push(format!(
                    "{}: warning: function `{}` may throw (`?`/调用可能抛错); consider adding #[throws(...)]",
                    loc(site),
                    full,
                )),
                Some(v) if v.is_empty() => issues.push(format!(
                    "{}: warning: function `{}` declares #[throws()] but may throw (hard discrepancy)",
                    loc(site),
                    full,
                )),
                _ => {}
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

fn declared_to_set(d: &EffectDecl) -> EffectSet {
    let mut s = EffectSet::default();
    if let Some(effs) = &d.effs {
        for e in effs {
            if e.as_str() == "io" {
                s.io = true;
            }
        }
    }
    if let Some(ts) = &d.throws {
        for t in ts {
            s.throws.insert(t.as_str());
        }
    }
    s
}

fn collect_fns<'a>(items: &'a [HirItem], out: &mut Vec<&'a HirFn>) {
    for it in items {
        match it {
            HirItem::Fn(f) => out.push(f),
            HirItem::Namespace { items, .. } => collect_fns(items, out),
            _ => {}
        }
    }
}

fn walk_stmts(stmts: &[HirStmt], calls: &mut Vec<crate::hir::ty::FnId>) {
    for s in stmts {
        match s {
            HirStmt::Assign { target, value } => {
                walk_expr(&**target, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::FieldAssign { object, value, .. } => {
                walk_expr(&**object, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::IndexAssign { object, index, value } => {
                walk_expr(&**object, calls);
                walk_expr(&**index, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::Return { value } => {
                if let Some(v) = value { walk_expr(&**v, calls); }
            }
            HirStmt::If { cond, then_block, elifs, else_block } => {
                walk_expr(&**cond, calls);
                walk_stmts(&then_block.stmts, calls);
                for (c, b) in elifs {
                    walk_expr(&**c, calls);
                    walk_stmts(&b.stmts, calls);
                }
                if let Some(b) = else_block { walk_stmts(&b.stmts, calls); }
            }
            HirStmt::While { cond, body } => {
                walk_expr(&**cond, calls);
                walk_stmts(&body.stmts, calls);
            }
            HirStmt::Expr(e) | HirStmt::Assume(e) => walk_expr(&**e, calls),
            HirStmt::Contract { cond, .. } => walk_expr(&**cond, calls),
            HirStmt::Block(inner) => walk_stmts(inner, calls),
            _ => {}
        }
    }
}

fn walk_expr(e: &dyn HirNode, calls: &mut Vec<crate::hir::ty::FnId>) {
    if let Some(id) = e.as_call() {
        calls.push(id);
    }
    e.for_each_child(&mut |c| walk_expr(c, calls));
}

