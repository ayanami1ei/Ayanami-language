//! A4/NLL：借用检查的语句/表达式遍历辅助。

use super::*;

pub(super) fn var_name(mir_fn: &MirFn, v: VarId) -> String {
    mir_fn
        .locals
        .get(v.0)
        .map(|l| l.name.as_str())
        .unwrap_or_else(|| format!("v{}", v.0))
}

/// 语句内按求值顺序遍历临时借用：调用结束时释放该调用内的借用。
pub(super) fn walk_stmt(
    stmt: &dyn MirStmtNode,
    active: &[(VarId, VarId, bool)],
    mir_fn: &MirFn,
    errors: &mut Vec<String>,
) {
    let sp = stmt.span();
    let at = if sp.start_line > 0 {
        format!(" (at {}:{})", sp.start_line, sp.start_col)
    } else {
        String::new()
    };
    let mut stack: Vec<Vec<(VarId, bool, bool)>> = vec![Vec::new()];
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
        walk_expr(r, &mut stack, active, mir_fn, errors, &at);
    }
}

fn walk_expr(
    node: &dyn MirNode,
    stack: &mut Vec<Vec<(VarId, bool, bool)>>,
    active: &[(VarId, VarId, bool)],
    mir_fn: &MirFn,
    errors: &mut Vec<String>,
    at: &str,
) {
    // 新借用（ref / ref mut）
    if let Some((var, mutable)) = node.as_ref() {
        let conflict = active
            .iter()
            .any(|(_, v, m)| *v == var && (*m || mutable))
            || stack
                .iter()
                .flatten()
                .any(|(v, m, _)| *v == var && (*m || mutable));
        if conflict {
            errors.push(format!(
                "cannot borrow `{}` as {}: already borrowed in this expression{}",
                var_name(mir_fn, var),
                if mutable { "mutable" } else { "immutable" },
                at
            ));
        }
        stack.last_mut().unwrap().push((var, mutable, false));
        return; // 不遍历 SMirRef 的目标
    }
    // 调用是求值上下文边界：调用结束后其内部临时借用全部释放
    if node.is_call() {
        // 两阶段借用：接收者位置（第一个子节点）的可变借用先记为 reserved，
        // 允许同一调用内后续参数读取该变量（s.add(s.v)）
        stack.push(Vec::new());
        let first = std::cell::Cell::new(true);
        node.for_each_child(&mut |c| {
            if first.replace(false) {
                if let Some((var, true)) = c.as_ref() {
                    let conflict = active.iter().any(|(_, v, m)| *v == var && *m)
                        || stack.iter().flatten().any(|(v, m, _)| *v == var && *m);
                    if conflict {
                        errors.push(format!(
                            "cannot borrow `{}` as mutable: already borrowed in this expression{}",
                            var_name(mir_fn, var),
                            at
                        ));
                    }
                    stack.last_mut().unwrap().push((var, true, true));
                    return;
                }
            }
            walk_expr(c, stack, active, mir_fn, errors, at);
        });
        stack.pop();
        return;
    }
    // 读取检查：可变借用期间不可读取
    if let Some(v) = node.as_local() {
        let mutably_borrowed = active.iter().any(|(_, bv, bm)| *bm && *bv == v)
            || stack.iter().flatten().any(|(bv, bm, reserved)| *bm && !*reserved && *bv == v);
        if mutably_borrowed {
            errors.push(format!(
                "cannot use `{}` because it is mutably borrowed{}",
                var_name(mir_fn, v),
                at
            ));
        }
    }
    node.for_each_child(&mut |c| walk_expr(c, stack, active, mir_fn, errors, at));
}
