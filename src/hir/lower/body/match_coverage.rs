//! Phase 1.3 / atb.3：match 穷尽性覆盖分析（枚举变体 / bool / 整数区间与字面量枚举）
//! 与 or-pattern 绑定一致性诊断。
use super::*;
use crate::parser::ast::pattern::Pattern;
use crate::parser::ast::Literal;

/// 枚举：无 guard 分支覆盖全部变体。
pub(super) fn enum_exhaustive(total: usize, pats: &[&Pattern]) -> bool {
    let mut covered = std::collections::HashSet::new();
    for p in pats { collect_variants(p, &mut covered); }
    total > 0 && covered.len() == total
}

/// bool：无 guard 分支同时覆盖 true/false。
pub(super) fn bool_exhaustive(pats: &[&Pattern]) -> bool {
    let mut t = false;
    let mut f = false;
    for p in pats { collect_bools(p, &mut t, &mut f); }
    t && f
}

/// 标量域（闭区间）；None = 不做穷尽判定（如 Int 以外的类型）。
fn scalar_domain(ty: &HirType) -> Option<(i128, i128)> {
    match ty {
        HirType::Int => Some((i64::MIN as i128, i64::MAX as i128)),
        HirType::IntN { bits, signed } => {
            let b = *bits as i128;
            if *signed {
                Some((-(1i128 << (b - 1)), (1i128 << (b - 1)) - 1))
            } else {
                Some((0, (1i128 << b) - 1))
            }
        }
        // 当前 char 为 8-bit（Unicode 迁移见 #93）
        HirType::Char => Some((0, 255)),
        _ => None,
    }
}

/// 整数/定宽整数/char：区间与字面量枚举合并后覆盖整个值域。
pub(super) fn scalar_exhaustive(base: &HirType, pats: &[&Pattern]) -> bool {
    let Some((min, max)) = scalar_domain(base) else { return false; };
    let mut ivs = Vec::new();
    for p in pats { collect_intervals(p, &mut ivs); }
    covers_domain(&mut ivs, min, max)
}

fn collect_intervals(p: &Pattern, out: &mut Vec<(i128, i128)>) {
    match p {
        Pattern::Literal(Literal::Int(i, _)) => out.push((*i as i128, *i as i128)),
        Pattern::Literal(Literal::Char(c, _)) => out.push((*c as i128, *c as i128)),
        Pattern::Range { lo, hi, inclusive } => {
            let (l, h) = match (lo, hi) {
                (Literal::Int(l, _), Literal::Int(h, _)) => (*l as i128, *h as i128),
                (Literal::Char(l, _), Literal::Char(h, _)) => (*l as i128, *h as i128),
                _ => return,
            };
            out.push(if *inclusive { (l, h) } else { (l, h - 1) });
        }
        Pattern::Or(ps) => for x in ps { collect_intervals(x, out); },
        _ => {}
    }
}

/// 区间集合（已裁剪到 [min,max]）是否连续覆盖整个域。
fn covers_domain(intervals: &mut Vec<(i128, i128)>, min: i128, max: i128) -> bool {
    let mut iv: Vec<(i128, i128)> = intervals.iter().cloned()
        .filter(|(l, h)| *h >= min && *l <= max)
        .map(|(l, h)| (l.max(min), h.min(max)))
        .filter(|(l, h)| l <= h)
        .collect();
    if iv.is_empty() { return false; }
    iv.sort();
    if iv[0].0 > min { return false; }
    let mut end = iv[0].1;
    for (l, h) in iv.iter().skip(1) {
        if *l > end + 1 { return false; }
        if *h > end { end = *h; }
    }
    end >= max
}

fn collect_variants(p: &Pattern, out: &mut std::collections::HashSet<Symbol>) {
    match p {
        Pattern::Enum { name, .. } => { out.insert(*name); }
        Pattern::Or(ps) => for x in ps { collect_variants(x, out); },
        _ => {}
    }
}

fn collect_bools(p: &Pattern, t: &mut bool, f: &mut bool) {
    match p {
        Pattern::Literal(Literal::Bool(b, _)) => { if *b { *t = true; } else { *f = true; } }
        Pattern::Or(ps) => for x in ps { collect_bools(x, t, f); },
        _ => {}
    }
}

/// atb.3：or-pattern 绑定一致性诊断（名字集合 + 类型；顺序无关）。
pub(super) fn check_or_bindings(
    first: &[(Symbol, HirType)],
    branch: &[(Symbol, HirType)],
    branch_no: usize,
    span: &Span,
) -> Result<()> {
    for (n, t) in first {
        match branch.iter().find(|(mn, _)| mn == n) {
            None => return Err(Error::Hir(format!(
                "or-pattern branch {} does not bind `{}` (all branches must bind the same names) (at {}:{})",
                branch_no, n.as_str(), span.start_line, span.start_col
            ))),
            Some((_, mt)) if mt != t => return Err(Error::Hir(format!(
                "or-pattern branch {} binds `{}` with a different type ({} vs {}) (at {}:{})",
                branch_no, n.as_str(), hir_type_display(t), hir_type_display(mt),
                span.start_line, span.start_col
            ))),
            _ => {}
        }
    }
    for (n, _) in branch {
        if !first.iter().any(|(f, _)| f == n) {
            return Err(Error::Hir(format!(
                "or-pattern branch {} binds extra name `{}` (at {}:{})",
                branch_no, n.as_str(), span.start_line, span.start_col
            )));
        }
    }
    Ok(())
}
