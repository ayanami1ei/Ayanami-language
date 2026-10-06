//! A4：借用来源收集（引用局部/引用拷贝/返回引用的调用/结构体引用字段）。

use std::collections::{HashMap, HashSet};

use super::*;
use super::cfg::Payload;
use super::loans::Loan;
use super::walk::var_name;

/// 收集结果：引用局部的 loan、结构体引用字段的首次定义节点、含引用字段的局部。
pub(super) fn collect(
    mir_fn: &MirFn,
    ref_params: &[(VarId, bool)],
    table: &super::FollowTable,
    cfg: &cfg::Cfg,
) -> Result<(HashMap<VarId, Loan>, HashMap<VarId, usize>, HashSet<VarId>)> {
    // 1) 收集引用局部变量：r = ref x / r = s（引用拷贝）/ r = f(args)（返回引用的调用）
    //    A4b-2：结构体字面量的引用字段 → 以结构体局部为键登记 loan
    let mut loans: HashMap<VarId, Loan> = HashMap::new();
    let mut defined_at: HashMap<VarId, usize> = HashMap::new();
    // A4b-2：含引用字段的结构体局部（v1 禁止整体移动/返回）
    let mut struct_ref_locals: std::collections::HashSet<VarId> = std::collections::HashSet::new();
    // cfg 节点按源码逆序构建；收集借用来源时按源码顺序遍历
    for (node_idx, node) in cfg.nodes.iter().enumerate().rev() {
        if let Payload::Stmt(s) = &node.payload {
            let at = {
                let sp = s.span();
                if sp.start_line > 0 { format!(" (at {}:{})", sp.start_line, sp.start_col) } else { String::new() }
            };
            let Some((target, value)) = s.assign_parts() else { continue };
            let Some(r) = target.as_local() else { continue };
            if !matches!(target.expr_type(), HirType::Ref(..)) {
                // A4b-2：结构体引用字段（值可能被 implicit_move 包一层）
                let lit: &dyn MirNode = value.move_expr().map(|v| &**v).unwrap_or(&**value);
                if let Some(fields) = lit.struct_literal_fields() {
                    let mut primary: Option<Loan> = None;
                    let mut extras: Vec<(VarId, bool)> = Vec::new();
                    let mut bad = false;
                    for (_, fv) in fields {
                        if !matches!(fv.expr_type(), HirType::Ref(..)) {
                            continue;
                        }
                        let resolved = if let Some((var, _)) = fv.as_ref() {
                            Some(Loan { var, mutable: matches!(fv.expr_type(), HirType::Ref(_, true)), origin: var, extra: Vec::new() })
                        } else if let Some(src) = fv.as_local() {
                            if let Some(l) = loans.get(&src) {
                                Some(Loan { var: l.var, mutable: l.mutable, origin: l.origin, extra: Vec::new() })
                            } else if let Some(&(_, m)) = ref_params.iter().find(|(v, _)| *v == src) {
                                Some(Loan { var: src, mutable: m, origin: src, extra: Vec::new() })
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        match resolved {
                            Some(l) => {
                                if primary.is_none() {
                                    primary = Some(l);
                                } else {
                                    extras.push((l.var, l.mutable));
                                }
                            }
                            None => bad = true,
                        }
                    }
                    if bad {
                        return Err(Error::Borrow(format!(
                            "reference stored in field of `{}` must come directly from a reference parameter or local",
                            var_name(mir_fn, r)
                        ) + &at));
                    }
                    if let Some(mut p) = primary {
                        if loans.contains_key(&r) {
                            return Err(Error::Borrow(format!(
                                "struct `{}` cannot hold reference fields from multiple/reassigned sources (borrow checker limitation)",
                                var_name(mir_fn, r)
                            ) + &at));
                        }
                        p.extra = extras;
                        loans.insert(r, p);
                        defined_at.insert(r, node_idx);
                        struct_ref_locals.insert(r);
                    }
                }
                continue;
            }
            if let Some((var, mutable)) = value.as_ref() {
                if loans.contains_key(&r) {
                    return Err(Error::Borrow(format!(
                        "reference local `{}` cannot be reassigned (borrow checker limitation)",
                        var_name(mir_fn, r)
                    ) + &at));
                }
                loans.insert(r, Loan { var, mutable, origin: r, extra: Vec::new() });
            } else if let Some(src) = value.as_local() {
                // 引用拷贝：r = s
                if loans.contains_key(&r) {
                    return Err(Error::Borrow(format!(
                        "reference local `{}` cannot be reassigned (borrow checker limitation)",
                        var_name(mir_fn, r)
                    ) + &at));
                }
                if let Some(l) = loans.get(&src) {
                    loans.insert(r, Loan { var: l.var, mutable: l.mutable, origin: l.origin, extra: Vec::new() });
                } else if let Some(&(_, mutable)) = ref_params.iter().find(|(v, _)| *v == src) {
                    loans.insert(r, Loan { var: src, mutable, origin: src, extra: Vec::new() });
                }
            } else if value.is_call() {
                // 返回引用的调用：借用目标来自实参中的引用
                let ret_mut = matches!(value.expr_type(), HirType::Ref(_, true));
                let mut found = None;
                // A4c：若被调函数声明了 follow_with，按声明的来源绑定实参（多来源联合约束）
                let resolved = value.call_fn_id()
                    .and_then(|id| table.get(&id))
                    .map(|info| info.resolved.clone())
                    .filter(|r| !r.is_empty());
                if let Some(resolved) = resolved {
                    let mut primary: Option<Loan> = None;
                    let mut extras: Vec<(VarId, bool)> = Vec::new();
                    let mut bad = false;
                    let mut idx = 0usize;
                    value.for_each_child(&mut |c| {
                        if resolved.iter().any(|(_, ri)| *ri == idx) {
                            let l = if let Some((var, mutable)) = c.as_ref() {
                                Some(Loan { var, mutable, origin: r, extra: Vec::new() })
                            } else if let Some(src) = c.as_local() {
                                if let Some(l) = loans.get(&src) {
                                    Some(Loan { var: l.var, mutable: l.mutable, origin: l.origin, extra: Vec::new() })
                                } else if let Some(&(_, mutable)) = ref_params.iter().find(|(v, _)| *v == src) {
                                    Some(Loan { var: src, mutable, origin: src, extra: Vec::new() })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            match l {
                                Some(l) => {
                                    if primary.is_none() { primary = Some(l); } else { extras.push((l.var, l.mutable)); }
                                }
                                None => bad = true,
                            }
                        }
                        idx += 1;
                    });
                    if bad {
                        return Err(Error::Borrow(format!(
                            "follow_with source argument of call assigned to `{}` is not a reference",
                            var_name(mir_fn, r)
                        ) + &at));
                    }
                    if let Some(mut p) = primary {
                        p.extra = extras;
                        found = Some(p);
                    }
                }
                if found.is_none() {
                value.for_each_child(&mut |c| {
                    if found.is_none() {
                        if let Some((var, mutable)) = c.as_ref() {
                            found = Some(Loan { var, mutable, origin: r, extra: Vec::new() });
                        } else if let Some(src) = c.as_local() {
                            if let Some(l) = loans.get(&src) {
                                found = Some(Loan { var: l.var, mutable: l.mutable, origin: l.origin, extra: Vec::new() });
                            } else if let Some(&(_, mutable)) = ref_params.iter().find(|(v, _)| *v == src) {
                                found = Some(Loan { var: src, mutable, origin: src, extra: Vec::new() });
                            }
                        }
                    }
                });
                }
                match found {
                    Some(l) => {
                        if loans.contains_key(&r) {
                            return Err(Error::Borrow(format!(
                                "reference local `{}` cannot be reassigned (borrow checker limitation)",
                                var_name(mir_fn, r)
                            ) + &at));
                        }
                        if ret_mut && !l.mutable {
                            return Err(Error::Borrow(format!(
                                "cannot return `ref mut` from an immutable borrow of `{}`",
                                var_name(mir_fn, l.var)
                            ) + &at));
                        }
                        loans.insert(r, Loan { var: l.var, mutable: ret_mut, origin: l.origin, extra: Vec::new() });
                    }
                    None => {
                        return Err(Error::Borrow(format!(
                            "call returning a reference has no reference argument (at assignment to `{}`)",
                            var_name(mir_fn, r)
                        ) + &at));
                    }
                }
            } else if value.refs_global().is_some() {
                // M6.2：引用全局（static）——全局始终存活，不参与局部 loan 跟踪
            } else {
                return Err(Error::Borrow(format!(
                    "reference local `{}` must be initialized from `ref`, another reference, or a call returning a reference",
                    var_name(mir_fn, r)
                ) + &at));
            }
        }
    }

    Ok((loans, defined_at, struct_ref_locals))
}
