//! M2 闭包健全性检查：拥有捕获不可移出闭包（FnOnce 语义未建模）。
//!
//! 检测闭包函数体内把 env 捕获字段整体 move 出去的表达式
//! （`x = a` / `return a` / 按值传参等经 `implicit_move` 包装为 SMove）。
//! 否则捕获缓冲会被 env 与目标各释放一次（double free）。

use super::*;
use crate::hir::ty::VarId;

/// 在闭包函数体内查找“把 env 捕获字段 move 出去”的表达式，返回捕获名。
pub(crate) fn find_capture_move_out(stmts: &[HirStmt], env_var: VarId) -> Option<Symbol> {
    stmts.iter().find_map(|s| stmt_move_out(s, env_var))
}

fn stmt_move_out(stmt: &HirStmt, env_var: VarId) -> Option<Symbol> {
    match stmt {
        HirStmt::Assign { value, .. } | HirStmt::Expr { expr: value, .. } => {
            node_move_out(&**value, env_var)
        }
        HirStmt::Return { value: Some(v), .. } => node_move_out(&**v, env_var),
        HirStmt::DerefAssign { target, value, .. } => {
            node_move_out(&**target, env_var).or_else(|| node_move_out(&**value, env_var))
        }
        HirStmt::FieldAssign { object, value, .. } => {
            node_move_out(&**object, env_var).or_else(|| node_move_out(&**value, env_var))
        }
        HirStmt::IndexAssign { object, index, value, .. } => node_move_out(&**object, env_var)
            .or_else(|| node_move_out(&**index, env_var))
            .or_else(|| node_move_out(&**value, env_var)),
        HirStmt::Assume { cond, .. } | HirStmt::Contract { cond, .. } => node_move_out(&**cond, env_var),
        HirStmt::If { cond, then_block, elifs, else_block, .. } => {
            node_move_out(&**cond, env_var)
                .or_else(|| find_capture_move_out(&then_block.stmts, env_var))
                .or_else(|| elifs.iter().find_map(|(c, b)| {
                    node_move_out(&**c, env_var).or_else(|| find_capture_move_out(&b.stmts, env_var))
                }))
                .or_else(|| else_block.as_ref().and_then(|b| find_capture_move_out(&b.stmts, env_var)))
        }
        HirStmt::While { cond, body, .. } => {
            node_move_out(&**cond, env_var).or_else(|| find_capture_move_out(&body.stmts, env_var))
        }
        HirStmt::Block { stmts, .. } => find_capture_move_out(stmts, env_var),
        HirStmt::Return { value: None, .. } | HirStmt::Break { .. } | HirStmt::Continue { .. } => None,
    }
}

fn node_move_out(node: &dyn HirNode, env_var: VarId) -> Option<Symbol> {
    if let Some(inner) = node.as_move() {
        if let Some((obj, field, _)) = inner.as_field_access() {
            if obj.as_local() == Some(env_var) {
                return Some(field);
            }
        }
    }
    let mut found = None;
    node.for_each_child(&mut |c| {
        if found.is_none() {
            found = node_move_out(c, env_var);
        }
    });
    found
}
