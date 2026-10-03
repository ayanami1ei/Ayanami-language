use super::*;
use super::cfg::Payload;
use super::walk::{var_name, walk_stmt};

/// 引用局部变量/参数对应的借用。
pub(super) struct Loan {
    /// 被借用的变量
    pub(super) var: VarId,
    pub(super) mutable: bool,
    /// 借用的"来源"：ref 局部自身，或（拷贝时）来源参数/局部
    pub(super) origin: VarId,
    /// A4c：多来源联合约束（follow_with(a, b) 取最短）
    pub(super) extra: Vec<(VarId, bool)>,
}

/// 检查入口。
/// `ref_params`: 函数的引用参数（VarId, 是否可变）。
pub fn check_fn(mir_fn: &MirFn, ref_params: &[(VarId, bool)], table: &super::FollowTable) -> Result<()> {
    let cfg = cfg::build(&mir_fn.body);
    let live_in = liveness::live_in(&cfg);

    let (loans, defined_at, struct_ref_locals) = super::collect::collect(mir_fn, ref_params, table, &cfg)?;

    // A4b：解析 #[follow_with] 来源 → 引用参数
    let declared: Vec<(crate::intern::Symbol, VarId)> = mir_fn.follow_sources.iter()
        .map(|s| super::follow::match_source(mir_fn, s, ref_params).map(|v| (*s, v)))
        .collect::<Result<Vec<_>>>()?;
    let mut errors: Vec<String> = Vec::new();

    for (i, node) in cfg.nodes.iter().enumerate() {
        let live = &live_in[i];
        // A4c：扁平化（key, var, mutable），含多来源 extra 约束
        let active: Vec<(VarId, VarId, bool)> = live
            .iter()
            .filter_map(|r| loans.get(r).map(|l| (*r, l)))
            .flat_map(|(k, l)| {
                std::iter::once((k, l.var, l.mutable))
                    .chain(l.extra.iter().map(move |(v, m)| (k, *v, *m)))
            })
            .collect();

        // 活跃借用之间的冲突（同一变量、至少一个可变）
        for a in 0..active.len() {
            for b in (a + 1)..active.len() {
                let (_, va, ma) = active[a];
                let (_, vb, mb) = active[b];
                if va == vb && (ma || mb) {
                    errors.push(format!(
                        "cannot borrow `{}` as {} because it is also borrowed",
                        var_name(mir_fn, va),
                        if ma { "mutable" } else { "immutable" }
                    ));
                }
            }
        }
        // 借用期间写入/移动/drop 被借用的变量（结构体引用字段的首次定义除外）
        let defined = if let Payload::Stmt(s) = &node.payload {
            s.assign_parts().and_then(|(t, _)| t.as_local())
        } else {
            None
        };
        for w in &node.writes {
            if Some(*w) == defined && defined_at.get(w) == Some(&i) {
                continue;
            }
            if struct_ref_locals.contains(w) {
                errors.push(format!(
                    "struct `{}` contains reference fields and cannot be moved/returned (borrow checker limitation)",
                    var_name(mir_fn, *w)
                ));
                continue;
            }
            if active.iter().any(|(_, v, _)| *v == *w) {
                errors.push(format!(
                    "cannot assign to or move `{}` because it is borrowed",
                    var_name(mir_fn, *w)
                ));
            }
        }

        // 可变借用期间读取
        for u in &node.uses {
            if active.iter().any(|(_, v, m)| *m && *v == *u) {
                errors.push(format!(
                    "cannot use `{}` because it is mutably borrowed",
                    var_name(mir_fn, *u)
                ));
            }
        }

        if let Payload::Stmt(s) = &node.payload {
            // 返回引用：必须来自唯一的引用参数（生命周期省略）
            if let Some(v) = s.return_value() {
                if matches!(mir_fn.return_type, HirType::Ref(..)) {
                    let origin = v.as_local().and_then(|lv| {
                        if ref_params.iter().any(|(p, _)| *p == lv) {
                            Some(lv)
                        } else {
                            loans.get(&lv).map(|l| l.origin)
                        }
                    });
                    if !declared.is_empty() {
                        // A4b：返回引用必须跟随声明的来源之一
                        let ok = origin.map_or(false, |o| declared.iter().any(|(_, p)| *p == o));
                        let mut_ok = !matches!(mir_fn.return_type, HirType::Ref(_, true))
                            || origin.map_or(false, |o| ref_params.iter().any(|(p, m)| *p == o && *m));
                        if !ok || !mut_ok {
                            errors.push(format!(
                                "returned reference must follow one of the #[follow_with(...)] sources"
                            ));
                        }
                    } else {
                        let ok = ref_params.iter().any(|(p, pm)| {
                            (origin == Some(*p))
                                && (!matches!(mir_fn.return_type, HirType::Ref(_, true)) || *pm)
                        });
                        if !ok {
                            errors.push(format!(
                                "returned reference must be derived from the elided reference parameter"
                            ));
                        }
                    }
                }
            }
            // 引用局部定义：检查新借用与已有活跃借用的冲突
            if let Some((target, value)) = s.assign_parts() {
                if matches!(target.expr_type(), HirType::Ref(..)) {
                    let defining = target.as_local();
                    let borrowed = loans.get(&defining.unwrap_or(VarId(usize::MAX))).map(|l| (l.var, l.mutable));
                    if let Some((var, mutable)) = borrowed {
                        let conflict = active.iter().any(|(k, v, m)| {
                            Some(*k) != defining && *v == var && (*m || mutable)
                        });
                        if conflict {
                            errors.push(format!(
                                "cannot borrow `{}` as {} because it is also borrowed",
                                var_name(mir_fn, var),
                                if mutable { "mutable" } else { "immutable" }
                            ));
                        }
                    }
                    let _ = value;
                    continue; // 定义语句不再按临时借用遍历
                }
            }
            walk_stmt(*s, &active, mir_fn, &mut errors);
        }
    }

    errors.sort();
    errors.dedup();
    if let Some(e) = errors.into_iter().next() {
        return Err(Error::Borrow(e));
    }
    Ok(())
}
