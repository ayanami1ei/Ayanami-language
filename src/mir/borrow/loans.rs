use std::collections::HashMap;
use super::*;
use super::cfg::Payload;

/// 引用局部变量对应的借用。
struct Loan {
    var: VarId,
    mutable: bool,
}

pub fn check_fn(mir_fn: &MirFn) -> Result<()> {
    let cfg = cfg::build(&mir_fn.body);
    let live_in = liveness::live_in(&cfg);

    // 1) 收集 `r = ref x` / `r = ref mut x` 形式的引用局部变量
    let mut loans: HashMap<VarId, Loan> = HashMap::new();
    for node in &cfg.nodes {
        if let Payload::Stmt(s) = &node.payload {
            if let Some((target, value)) = s.assign_parts() {
                if let Some(r) = target.as_local() {
                    if matches!(target.expr_type(), HirType::Ref(..)) {
                        let Some((var, mutable)) = value.as_ref() else {
                            return Err(Error::Borrow(format!(
                                "reference local `{}` must be initialized directly from `ref`/`ref mut`",
                                var_name(mir_fn, r)
                            )));
                        };
                        loans.insert(r, Loan { var, mutable });
                    }
                }
            }
        }
    }

    let mut errors: Vec<String> = Vec::new();

    for (i, node) in cfg.nodes.iter().enumerate() {
        let live = &live_in[i];
        let active: Vec<(VarId, &Loan)> = live
            .iter()
            .filter_map(|r| loans.get(r).map(|l| (*r, l)))
            .collect();

        // 活跃借用之间的冲突（同一变量、至少一个可变）
        for a in 0..active.len() {
            for b in (a + 1)..active.len() {
                let (_, la) = active[a];
                let (_, lb) = active[b];
                if la.var == lb.var && (la.mutable || lb.mutable) {
                    errors.push(format!(
                        "cannot borrow `{}` as {} because it is also borrowed",
                        var_name(mir_fn, la.var),
                        if la.mutable { "mutable" } else { "immutable" }
                    ));
                }
            }
        }

        // 借用期间写入/移动/drop 被借用的变量
        for w in &node.writes {
            if active.iter().any(|(_, l)| l.var == *w) {
                errors.push(format!(
                    "cannot assign to or move `{}` because it is borrowed",
                    var_name(mir_fn, *w)
                ));
            }
        }

        // 可变借用期间读取
        for u in &node.uses {
            if active.iter().any(|(_, l)| l.mutable && l.var == *u) {
                errors.push(format!(
                    "cannot use `{}` because it is mutably borrowed",
                    var_name(mir_fn, *u)
                ));
            }
        }

        if let Payload::Stmt(s) = &node.payload {
            // 引用局部定义：检查新借用与已有活跃借用的冲突
            if let Some((target, value)) = s.assign_parts() {
                if matches!(target.expr_type(), HirType::Ref(..)) {
                    let defining = target.as_local();
                    if let Some((var, mutable)) = value.as_ref() {
                        let conflict = active.iter().any(|(r, l)| {
                            Some(*r) != defining && l.var == var && (l.mutable || mutable)
                        });
                        if conflict {
                            errors.push(format!(
                                "cannot borrow `{}` as {} because it is also borrowed",
                                var_name(mir_fn, var),
                                if mutable { "mutable" } else { "immutable" }
                            ));
                        }
                    }
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

fn var_name(mir_fn: &MirFn, v: VarId) -> String {
    mir_fn
        .locals
        .get(v.0)
        .map(|l| l.name.as_str())
        .unwrap_or_else(|| format!("v{}", v.0))
}

/// 语句内按求值顺序遍历临时借用：调用结束时释放该调用内的借用。
fn walk_stmt(
    stmt: &dyn MirStmtNode,
    active: &[(VarId, &Loan)],
    mir_fn: &MirFn,
    errors: &mut Vec<String>,
) {
    let mut stack: Vec<Vec<(VarId, bool)>> = vec![Vec::new()];
    let mut roots: Vec<&dyn MirNode> = Vec::new();
    if let Some((_, v)) = stmt.assign_parts() {
        roots.push(&**v);
    }
    if let Some((o, v)) = stmt.field_assign_parts() {
        roots.push(&**o);
        roots.push(&**v);
    }
    if let Some((o, i, v)) = stmt.index_assign_parts() {
        roots.push(&**o);
        roots.push(&**i);
        roots.push(&**v);
    }
    if let Some(v) = stmt.return_value() {
        roots.push(&**v);
    }
    if let Some(e) = stmt.expr_part() {
        roots.push(&**e);
    }
    for r in roots {
        walk_expr(r, &mut stack, active, mir_fn, errors);
    }
}

fn walk_expr(
    node: &dyn MirNode,
    stack: &mut Vec<Vec<(VarId, bool)>>,
    active: &[(VarId, &Loan)],
    mir_fn: &MirFn,
    errors: &mut Vec<String>,
) {
    // 新借用（ref / ref mut）
    if let Some((var, mutable)) = node.as_ref() {
        let conflict = active
            .iter()
            .any(|(_, l)| l.var == var && (l.mutable || mutable))
            || stack
                .iter()
                .flatten()
                .any(|(v, m)| *v == var && (*m || mutable));
        if conflict {
            errors.push(format!(
                "cannot borrow `{}` as {}: already borrowed in this expression",
                var_name(mir_fn, var),
                if mutable { "mutable" } else { "immutable" }
            ));
        }
        stack.last_mut().unwrap().push((var, mutable));
        return; // 不遍历 SMirRef 的目标
    }
    // 调用是求值上下文边界：调用结束后其内部临时借用全部释放
    if node.is_call() {
        stack.push(Vec::new());
        node.for_each_child(&mut |c| walk_expr(c, stack, active, mir_fn, errors));
        stack.pop();
        return;
    }
    // 读取检查：可变借用期间不可读取
    if let Some(v) = node.as_local() {
        let mutably_borrowed = active.iter().any(|(_, l)| l.mutable && l.var == v)
            || stack.iter().flatten().any(|(bv, bm)| *bm && *bv == v);
        if mutably_borrowed {
            errors.push(format!(
                "cannot use `{}` because it is mutably borrowed",
                var_name(mir_fn, v)
            ));
        }
    }
    node.for_each_child(&mut |c| walk_expr(c, stack, active, mir_fn, errors));
}
