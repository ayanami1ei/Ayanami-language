//! A3a：效应注解解析与存储（注解权威；推断辅助在后续阶段）。
//!
//! - `#[throws(E1, E2)]`：异常效应；`#[throws()]`/`#[throws]` 显式空集。
//! - `#[eff(io, state, alloc)]`：其他效应；`#[eff()]` 显式空集。
//! - `#[pure]` 视为 `#[eff()]` 的别名。
//! - 允许多声明与少声明；不引入 `unknown` 效应（无注解即无保证）。

use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, AttrArg, Expr};

/// 已知效应名（`#[eff(...)]` 集合）。
pub const KNOWN_EFFECTS: &[&str] = &["io", "state", "alloc"];

/// 函数声明的效应集合。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectDecl {
    /// None = 未声明；Some([]) = 显式空集（可作为优化保证）
    pub throws: Option<Vec<Symbol>>,
    /// None = 未声明；Some([]) = 显式空集
    pub effs: Option<Vec<Symbol>>,
}

impl EffectDecl {
    /// 显式声明了空 eff 集合（无 io/state/alloc）。
    pub fn no_effects(&self) -> bool {
        matches!(&self.effs, Some(v) if v.is_empty())
    }

    /// 显式声明了空 throws 集合。
    pub fn no_throws(&self) -> bool {
        matches!(&self.throws, Some(v) if v.is_empty())
    }
}

/// 解析函数级效应注解。
pub fn parse(attrs: &[Attr]) -> Result<EffectDecl> {
    let mut decl = EffectDecl::default();
    for a in attrs.iter().filter(|a| a.is_builtin()) {
        match a.name.as_str().as_str() {
            "throws" => {
                let names = parse_names(a, "throws")?;
                let e = decl.throws.get_or_insert_with(Vec::new);
                for n in names {
                    if !e.contains(&n) { e.push(n); }
                }
            }
            "eff" => {
                let names = parse_names(a, "eff")?;
                for n in &names {
                    if !KNOWN_EFFECTS.contains(&n.as_str().as_str()) {
                        return Err(Error::Hir(format!(
                            "unknown effect `{}` (supported: {}) (at {}:{})",
                            n, KNOWN_EFFECTS.join(", "),
                            a.span.start_line, a.span.start_col
                        )));
                    }
                }
                let e = decl.effs.get_or_insert_with(Vec::new);
                for n in names {
                    if !e.contains(&n) { e.push(n); }
                }
            }
            // `#[pure]` = 显式空 eff 集合
            "pure" if decl.effs.is_none() => decl.effs = Some(Vec::new()),
            _ => {}
        }
    }
    Ok(decl)
}

fn parse_names(a: &Attr, kind: &str) -> Result<Vec<Symbol>> {
    let mut out = Vec::new();
    for arg in &a.args {
        match arg {
            AttrArg::Expr(e) => match e.as_ref() {
                Expr::Ident(s, _) => out.push(*s),
                _ => return Err(bad_arg(kind, a)),
            },
            AttrArg::KeyValue(..) => return Err(bad_arg(kind, a)),
        }
    }
    Ok(out)
}

fn bad_arg(kind: &str, a: &Attr) -> Error {
    Error::Hir(format!(
        "#[{}] expects effect/type names (at {}:{})",
        kind, a.span.start_line, a.span.start_col
    ))
}

// ═══════════════════════════════════════════════════════════════════
//  A3a-2：效应推断（辅助）与诊断
// ═══════════════════════════════════════════════════════════════════

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::hir::{HirFn, HirItem, HirNode, HirProgram, HirStmt};

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

/// runtime / libc 中已知的 IO 函数。
const IO_NAMES: &[&str] = &[
    "putchar", "printf", "getchar",
    "__ayanami_getchar", "__ayanami_putchar",
    "__ayanami_print_int", "__ayanami_print_str", "__ayanami_print_ln",
];

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
    let obs = collect_ast_observations(ast);
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
                Obs::Io(name, l, c) => Some((name.clone(), *l, *c)),
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
                Obs::Try(l, c) => Some((*l, *c)),
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

/// AST 观测点（用于精确行列定位）。
#[derive(Clone)]
enum Obs {
    Io(String, usize, usize),
    Try(usize, usize),
}

fn collect_ast_observations(program: &crate::parser::ast::Program) -> HashMap<String, Vec<Obs>> {
    let mut map = HashMap::new();
    for s in &program.stmts {
        scan_decl(s, "", &mut map);
    }
    map
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_string() } else { format!("{}.{}", prefix, name) }
}

fn scan_decl(stmt: &crate::parser::ast::Stmt, prefix: &str, map: &mut HashMap<String, Vec<Obs>>) {
    use crate::parser::ast::Stmt;
    match stmt {
        Stmt::FnDecl { name, body, .. } => {
            let mut obs = Vec::new();
            for s in &body.stmts {
                scan_body(s, &mut obs);
            }
            map.insert(qualify(prefix, &name.as_str()), obs);
        }
        Stmt::Namespace { name, items, .. } => {
            let p = qualify(prefix, &name.as_str());
            for s in items { scan_decl(s, &p, map); }
        }
        Stmt::ImplBlock { methods, .. } => {
            for m in methods { scan_decl(m, prefix, map); }
        }
        Stmt::Attributed { stmt, .. } => scan_decl(stmt, prefix, map),
        _ => {}
    }
}

fn scan_body(stmt: &crate::parser::ast::Stmt, obs: &mut Vec<Obs>) {
    use crate::parser::ast::Stmt;
    match stmt {
        Stmt::Assign { value, .. } => scan_expr(value, obs),
        Stmt::FieldAssign { object, value, .. } => {
            scan_expr(object, obs);
            scan_expr(value, obs);
        }
        Stmt::IndexAssign { object, index, value, .. } => {
            scan_expr(object, obs);
            scan_expr(index, obs);
            scan_expr(value, obs);
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value { scan_expr(v, obs); }
        }
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            scan_expr(cond, obs);
            for s in &then_block.stmts { scan_body(s, obs); }
            for (c, b) in elifs {
                scan_expr(c, obs);
                for s in &b.stmts { scan_body(s, obs); }
            }
            if let Some(b) = else_block {
                for s in &b.stmts { scan_body(s, obs); }
            }
        }
        Stmt::For { start, end, step, body, .. } => {
            scan_expr(start, obs);
            scan_expr(end, obs);
            if let Some(s) = step { scan_expr(s, obs); }
            for s in &body.stmts { scan_body(s, obs); }
        }
        Stmt::While { cond, body, .. } => {
            scan_expr(cond, obs);
            for s in &body.stmts { scan_body(s, obs); }
        }
        Stmt::Match { value, arms, .. } => {
            scan_expr(value, obs);
            for a in arms { scan_expr(&a.body, obs); }
        }
        Stmt::ExprStmt { expr, .. } => scan_expr(expr, obs),
        Stmt::Attributed { stmt, .. } => scan_body(stmt, obs),
        _ => {}
    }
}

fn scan_expr(e: &crate::parser::ast::Expr, obs: &mut Vec<Obs>) {
    use crate::parser::ast::Expr;
    match e {
        Expr::Binary { lhs, rhs, .. } => {
            scan_expr(lhs, obs);
            scan_expr(rhs, obs);
        }
        Expr::Unary { arg, .. } => scan_expr(arg, obs),
        Expr::FnCall { name, args, span } => {
            if IO_NAMES.contains(&name.as_str().as_str()) {
                obs.push(Obs::Io(name.as_str(), span.start_line, span.start_col));
            }
            for a in args { scan_expr(a, obs); }
        }
        Expr::Move(i, _) | Expr::Clone(i, _) | Expr::ToUnique(i, _) | Expr::Ref(i, _, _) => {
            scan_expr(i, obs);
        }
        Expr::MethodCall { object, args, .. } => {
            scan_expr(object, obs);
            for a in args { scan_expr(a, obs); }
        }
        Expr::FieldAccess { object, .. } => scan_expr(object, obs),
        Expr::StructLiteral { fields, .. } => {
            for (_, v) in fields { scan_expr(v, obs); }
        }
        Expr::ArrayLiteral(elems, _) => {
            for x in elems { scan_expr(x, obs); }
        }
        Expr::ArraySized { count, .. } => scan_expr(count, obs),
        Expr::Asm { outputs, inputs, .. } => {
            for (_, x) in outputs { scan_expr(x, obs); }
            for (_, x) in inputs { scan_expr(x, obs); }
        }
        Expr::Index { object, index, .. } => {
            scan_expr(object, obs);
            scan_expr(index, obs);
        }
        Expr::CallExpr { target, args, .. } => {
            scan_expr(target, obs);
            for a in args { scan_expr(a, obs); }
        }
        Expr::TryOp(inner, span) => {
            obs.push(Obs::Try(span.start_line, span.start_col));
            scan_expr(inner, obs);
        }
        Expr::Match { value, arms, .. } => {
            scan_expr(value, obs);
            for a in arms { scan_expr(&a.body, obs); }
        }
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for x in tuple_args { scan_expr(x, obs); }
            for (_, x) in named_args { scan_expr(x, obs); }
        }
        Expr::Lambda { body, .. } => {
            for s in body { scan_body(s, obs); }
        }
        Expr::Literal(_) | Expr::Ident(..) | Expr::Null(_) => {}
    }
}
