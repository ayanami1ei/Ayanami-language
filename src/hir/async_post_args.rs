//! async post await 实参渲染：构造器占位（类型默认值）与真实状态（self.<field>）。
use std::collections::{HashMap, HashSet};

use crate::intern::Symbol;
use crate::parser::ast::{Expr, Type};

/// 返回 (构造器占位实参, 真实状态实参)。
/// 占位：参数/全局保留原名；y 与更早 post 结果替换为类型默认值。
/// 真实：参数/y/更早 post 结果替换为 `self.<name>`。
pub(super) fn build_post_args(
    args: &[Expr],
    params: &[(Symbol, Type)],
    y: Symbol,
    y_ty: &Type,
    prior: &[(Symbol, Type)],
) -> Option<(Vec<String>, Vec<String>)> {
    let mut env: HashMap<Symbol, Type> = params.iter().cloned().collect();
    env.insert(y, y_ty.clone());
    for (n, t) in prior {
        env.insert(*n, t.clone());
    }
    let fields: HashSet<Symbol> = env.keys().copied().collect();
    let mut placeholders = Vec::new();
    let mut reals = Vec::new();
    for a in args {
        placeholders.push(crate::hir::async_mixed_expr::render_default(a, &env)?);
        reals.push(crate::hir::async_mixed_expr::render_field_expr(a, &fields)?);
    }
    Some((placeholders, reals))
}
