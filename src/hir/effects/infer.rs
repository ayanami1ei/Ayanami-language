//! A3：效应推断（辅助事实）与诊断。
//!
//! - 默认最好情况：从空集起步，扫函数体与调用链得到实际集合；
//! - 有效效应 = 声明 ∪ 推断（属性生成见 `lir/lower/util.rs::lir_effects`）；
//! - 告警：硬性出入（承诺与实际矛盾）报所有函数；缺失建议只对 pub 接口。

use crate::error::Result;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use super::IO_NAMES;
use crate::hir::{HirFn, HirItem, HirNode, HirProgram, HirStmt};
use crate::hir::effects::EffectDecl;

/// 推断出的实际效应（advisory）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectSet {
    /// 可能失败（`?`/Err 构造/调用链）
    pub throws: BTreeSet<String>,
    /// 具体效应名（io/state/alloc/...）
    pub effects: BTreeSet<String>,
    /// 调用了不可见 extern 且未承诺 pure → 优化保守
    pub may_unknown_effects: bool,
    /// 调用了不可见 extern 且未承诺 no_error → 优化保守
    pub may_unknown_errors: bool,
}

impl EffectSet {
    fn union(&mut self, o: &EffectSet) {
        self.throws.extend(o.throws.iter().cloned());
        self.effects.extend(o.effects.iter().cloned());
        self.may_unknown_effects |= o.may_unknown_effects;
        self.may_unknown_errors |= o.may_unknown_errors;
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

/// 分析：推断 + 写入 HirFn.inferred + 诊断。
pub fn analyze(hir: &mut HirProgram, ast: &crate::parser::ast::Program, src_path: &Path) -> Result<()> {
    let summaries = compute(hir);
    // 写回每个 HirFn
    for_each_fn_mut(&mut hir.items, &mut |f| {
        if let Some(s) = summaries.get(&f.fn_id) {
            f.inferred = s.clone();
        }
    });
    super::diag::diagnostics(hir, ast, src_path)?;
    Ok(())
}

/// 静默摘要：函数名 → 摘要（不写回、不告警；供 `defs` 等工具）。
pub fn summarize(hir: &HirProgram) -> HashMap<String, crate::hir::effects::EffectSummary> {
    let mut fns: Vec<&HirFn> = Vec::new();
    collect_fns(&hir.items, &mut fns);
    let inf = compute(hir);
    let mut out = HashMap::new();
    for f in fns {
        out.insert(f.name.as_str(), crate::hir::effects::EffectSummary {
            declared: f.effects.clone(),
            inferred: inf.get(&f.fn_id).cloned().unwrap_or_default(),
        });
    }
    out
}

/// 调用图不动点推断。
fn compute(hir: &HirProgram) -> HashMap<crate::hir::ty::FnId, EffectSet> {
    let mut fns: Vec<&HirFn> = Vec::new();
    collect_fns(&hir.items, &mut fns);
    let mut declared: HashMap<crate::hir::ty::FnId, EffectDecl> = HashMap::new();
    let mut has_body: HashSet<crate::hir::ty::FnId> = HashSet::new();
    for f in &fns {
        declared.insert(f.fn_id, f.effects.clone());
        if !(f.extern_c || f.body.stmts.is_empty()) {
            has_body.insert(f.fn_id);
        }
    }

    // 无体函数（extern/导入）的已知效应来自声明
    let mut known: HashMap<crate::hir::ty::FnId, EffectSet> = HashMap::new();
    for f in &fns {
        if !has_body.contains(&f.fn_id) {
            known.insert(f.fn_id, declared_to_set(&f.effects));
        }
    }

    // IO 原语判定：按函数名匹配
    let mut names: HashMap<crate::hir::ty::FnId, String> = HashMap::new();
    for f in &fns {
        names.insert(f.fn_id, f.name.as_str());
    }
    for imp in &hir.imported_fns {
        names.insert(imp.fn_id, imp.name.as_str());
    }
    // 导入函数的已知效应（声明 ∪ 包导出的推断事实）
    let mut imported_known: HashMap<crate::hir::ty::FnId, EffectSet> = HashMap::new();
    for imp in &hir.imported_fns {
        let mut s = declared_to_set(&imp.effects);
        s.union(&imp.inferred);
        imported_known.insert(imp.fn_id, s);
    }
    let names_io: HashSet<crate::hir::ty::FnId> = names.iter()
        .filter(|(_, n)| IO_NAMES.contains(&n.as_str()))
        .map(|(id, _)| *id)
        .collect();

    let mut inferred: HashMap<crate::hir::ty::FnId, EffectSet> = HashMap::new();
    for _ in 0..=fns.len() {
        let mut changed = false;
        for f in &fns {
            if !has_body.contains(&f.fn_id) {
                continue;
            }
            let mut calls = Vec::new();
            walk_stmts(&f.body.stmts, &mut calls);
            let mut set = EffectSet::default();
            for c in calls {
                for e in &c.effects {
                    set.effects.insert(e.clone());
                }
                if c.try_unwrap {
                    set.throws.insert("?".to_string());
                }
                if names_io.contains(&c.fn_id) {
                    set.effects.insert("io".to_string());
                }
                if let Some(v) = c.err_construct {
                    set.throws.insert(v);
                }
                if let Some(callee) = inferred.get(&c.fn_id)
                    .or_else(|| imported_known.get(&c.fn_id))
                {
                    set.union(callee);
                } else if let Some(k) = known.get(&c.fn_id) {
                    // 本地无体声明（extern）：声明效应生效；未显式承诺 pure/no_error 时按「未知」保守处理
                    // （extern 可能读写内存/失败）；编译器合成助手（溢出/契约失败）例外（#117）。
                    set.union(k);
                    let synth = names.get(&c.fn_id).map_or(false, |n| n.starts_with("__ayanami_ovf_")
                        || n == "__ayanami_require_fail" || n == "__ayanami_ensure_fail" || n == "__ayanami_invariant_fail");
                    if !synth {
                        let d = declared.get(&c.fn_id);
                        if !d.map_or(false, |d| d.pure) { set.may_unknown_effects = true; }
                        if !d.map_or(false, |d| d.no_error) { set.may_unknown_errors = true; }
                    }
                } else if c.fn_id.0 != usize::MAX {
                    // 不可见目标（导入/未声明的 extern）。
                    // 合成标记（alloc/state/err，FnId(MAX)）已贡献各自效应，不算未知 extern（#117）。
                    let d = declared.get(&c.fn_id);
                    let pure = d.map_or(false, |d| d.pure);
                    let no_error = d.map_or(false, |d| d.no_error);
                    if !pure {
                        set.may_unknown_effects = true;
                    }
                    if !no_error {
                        set.may_unknown_errors = true;
                    }
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
    inferred
}

fn declared_to_set(d: &EffectDecl) -> EffectSet {
    let mut s = EffectSet::default();
    for e in &d.effects {
        s.effects.insert(e.as_str());
    }
    s
}

pub(super) fn collect_fns<'a>(items: &'a [HirItem], out: &mut Vec<&'a HirFn>) {
    for it in items {
        match it {
            HirItem::Fn(f) => out.push(f),
            HirItem::Namespace { items, .. } => collect_fns(items, out),
            _ => {}
        }
    }
}

fn for_each_fn_mut(items: &mut [HirItem], f: &mut impl FnMut(&mut HirFn)) {
    for it in items {
        match it {
            HirItem::Fn(hf) => f(hf),
            HirItem::Namespace { items, .. } => for_each_fn_mut(items, f),
            _ => {}
        }
    }
}

/// 收集到的单个调用点信息。
struct CallInfo {
    fn_id: crate::hir::ty::FnId,
    try_unwrap: bool,
    err_construct: Option<String>,
    effects: BTreeSet<String>,
}

fn walk_stmts(stmts: &[HirStmt], calls: &mut Vec<CallInfo>) {
    for s in stmts {
        match s {
            HirStmt::Assign { target, value, .. } => {
                walk_expr(&**target, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::FieldAssign { object, value, .. } => {
                if is_observable_target(object) {
                    calls.push(state_marker());
                }
                walk_expr(&**object, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::DerefAssign { target, value, .. } => {
                if is_observable_target(target) {
                    calls.push(state_marker());
                }
                walk_expr(&**target, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::IndexAssign { object, index, value, .. } => {
                if is_observable_target(object) {
                    calls.push(state_marker());
                }
                walk_expr(&**object, calls);
                walk_expr(&**index, calls);
                walk_expr(&**value, calls);
            }
            HirStmt::Return { value, .. } => {
                if let Some(v) = value { walk_expr(&**v, calls); }
            }
            HirStmt::If { cond, then_block, elifs, else_block, .. } => {
                walk_expr(&**cond, calls);
                walk_stmts(&then_block.stmts, calls);
                for (c, b) in elifs {
                    walk_expr(&**c, calls);
                    walk_stmts(&b.stmts, calls);
                }
                if let Some(b) = else_block { walk_stmts(&b.stmts, calls); }
            }
            HirStmt::While { cond, body, .. } => {
                walk_expr(&**cond, calls);
                walk_stmts(&body.stmts, calls);
            }
            HirStmt::Expr { expr: e, .. } | HirStmt::Assume { cond: e, .. } => walk_expr(&**e, calls),
            HirStmt::Contract { cond, .. } => walk_expr(&**cond, calls),
            HirStmt::Block { stmts: inner, .. } => walk_stmts(inner, calls),
            _ => {}
        }
    }
}

fn state_marker() -> CallInfo {
    CallInfo { fn_id: crate::hir::ty::FnId(usize::MAX), try_unwrap: false, err_construct: None, effects: ["state".to_string()].into_iter().collect() }
}

fn is_observable_target(object: &crate::hir::HirNodeBox) -> bool {
    matches!(object.expr_type(), crate::hir::HirType::Ref(..) | crate::hir::HirType::Unique(..))
}

fn walk_expr(e: &dyn HirNode, calls: &mut Vec<CallInfo>) {
    let mut info: Option<CallInfo> = None;
    if let Some(id) = e.as_call() {
        info = Some(CallInfo { fn_id: id, try_unwrap: false, err_construct: None, effects: BTreeSet::new() });
    }
    if e.is_alloc() {
        let i = info.get_or_insert_with(|| CallInfo { fn_id: crate::hir::ty::FnId(usize::MAX), try_unwrap: false, err_construct: None, effects: BTreeSet::new() });
        i.effects.insert("alloc".to_string());
    }
    if let Some(v) = e.enum_variant() {
        if v.as_str() == "Err" {
            let i = info.get_or_insert_with(|| CallInfo { fn_id: crate::hir::ty::FnId(usize::MAX), try_unwrap: false, err_construct: None, effects: BTreeSet::new() });
            i.err_construct = Some("err".to_string());
        }
    }
    if let Some(mut i) = info {
        // 名字已知的 IO 原语
        let _ = &mut i;
        calls.push(i);
    }
    e.for_each_child(&mut |c| walk_expr(c, calls));
}
