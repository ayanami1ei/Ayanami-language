use std::collections::HashMap;

use crate::hir::async_mixed_types::type_default;
use crate::intern::Symbol;
use crate::parser::ast::stmt::MatchBody;
use crate::parser::ast::{Block, Expr, Stmt, Type};

use super::await_parts;

pub(super) fn body_has_await(body: &Block) -> bool {
    block_has_await(body)
}

fn block_has_await(b: &Block) -> bool {
    b.stmts.iter().any(stmt_has_await) || b.tail.as_ref().is_some_and(|e| expr_has_await(e))
}

pub(crate) fn stmt_has_await(st: &Stmt) -> bool {
    match st {
        Stmt::Assign { value, .. } => expr_has_await(value),
        Stmt::ExprStmt { expr, .. } => expr_has_await(expr),
        Stmt::Return { value, .. } => value.as_ref().map_or(false, |e| expr_has_await(e)),
        Stmt::FieldAssign { object, value, .. } => expr_has_await(object) || expr_has_await(value),
        Stmt::IndexAssign {
            object,
            index,
            value,
            ..
        } => expr_has_await(object) || expr_has_await(index) || expr_has_await(value),
        Stmt::DerefAssign { target, value, .. } => expr_has_await(target) || expr_has_await(value),
        Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            expr_has_await(cond)
                || then_block.stmts.iter().any(stmt_has_await)
                || elifs
                    .iter()
                    .any(|(c, b)| expr_has_await(c) || b.stmts.iter().any(stmt_has_await))
                || else_block
                    .as_ref()
                    .map_or(false, |b| b.stmts.iter().any(stmt_has_await))
        }
        Stmt::While { cond, body, .. } => expr_has_await(cond) || block_has_await(body),
        Stmt::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            expr_has_await(start)
                || expr_has_await(end)
                || step.as_ref().is_some_and(|e| expr_has_await(e))
                || block_has_await(body)
        }
        Stmt::ForIn { iterable, body, .. } => expr_has_await(iterable) || block_has_await(body),
        Stmt::Match { value, arms, .. } => {
            expr_has_await(value)
                || arms.iter().any(|a| {
                    a.guard.as_ref().map_or(false, |g| expr_has_await(g))
                        || match &a.body {
                            MatchBody::Expr(e) => expr_has_await(e),
                            MatchBody::Block(b) => block_has_await(b),
                        }
                })
        }
        Stmt::Block(b) | Stmt::Unsafe { block: b, .. } => block_has_await(b),
        Stmt::Attributed { stmt, .. } => stmt_has_await(stmt),
        _ => false,
    }
}

pub(super) fn expr_has_await(e: &Expr) -> bool {
    match e {
        Expr::Literal(..) | Expr::Suffixed { .. } | Expr::Ident(..) | Expr::Null(..) => false,
        Expr::Binary { lhs, rhs, .. } => expr_has_await(lhs) || expr_has_await(rhs),
        Expr::Unary { arg, .. } => expr_has_await(arg),
        Expr::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            expr_has_await(cond)
                || then_block.stmts.iter().any(stmt_has_await)
                || elifs
                    .iter()
                    .any(|(c, b)| expr_has_await(c) || b.stmts.iter().any(stmt_has_await))
                || else_block
                    .as_ref()
                    .map_or(false, |b| b.stmts.iter().any(stmt_has_await))
        }
        Expr::Cast { expr, .. }
        | Expr::Move(expr, _)
        | Expr::Clone(expr, _)
        | Expr::ToUnique(expr, _)
        | Expr::Ref(expr, _, _)
        | Expr::TryOp(expr, _) => expr_has_await(expr),
        Expr::FnCall { name, args, .. } => {
            name.as_str() == "__await" || args.iter().any(expr_has_await)
        }
        Expr::MethodCall { object, args, .. } => {
            expr_has_await(object) || args.iter().any(expr_has_await)
        }
        Expr::FieldAccess { object, .. } => expr_has_await(object),
        Expr::StructLiteral { fields, .. } => fields.iter().any(|(_, e)| expr_has_await(e)),
        Expr::ArrayLiteral(elems, _) => elems.iter().any(expr_has_await),
        Expr::ArraySized { count, .. } => expr_has_await(count),
        Expr::ArrayRepeat { value, count, .. } => expr_has_await(value) || expr_has_await(count),
        Expr::Asm {
            outputs, inputs, ..
        } => {
            outputs.iter().any(|(_, e)| expr_has_await(e))
                || inputs.iter().any(|(_, e)| expr_has_await(e))
        }
        Expr::Index { object, index, .. } => expr_has_await(object) || expr_has_await(index),
        Expr::CallExpr { target, args, .. } => {
            expr_has_await(target) || args.iter().any(expr_has_await)
        }
        Expr::Match { value, arms, .. } => {
            expr_has_await(value)
                || arms.iter().any(|a| {
                    a.guard.as_ref().map_or(false, |g| expr_has_await(g))
                        || match &a.body {
                            MatchBody::Expr(e) => expr_has_await(e),
                            MatchBody::Block(b) => block_has_await(b),
                        }
                })
        }
        Expr::EnumConstruct {
            tuple_args,
            named_args,
            ..
        } => {
            tuple_args.iter().any(expr_has_await)
                || named_args.iter().any(|(_, e)| expr_has_await(e))
        }
        Expr::Lambda { body, .. } => block_has_await(body),
        // 宏展开在 async 变换前完成；残留宏调用不含 await 语义。
        Expr::MacroCall { .. } => false,
    }
}

fn has_break_continue(st: &Stmt) -> bool {
    match st {
        Stmt::Break { .. } | Stmt::Continue { .. } => true,
        Stmt::If {
            then_block,
            elifs,
            else_block,
            ..
        } => {
            then_block.stmts.iter().any(has_break_continue)
                || elifs
                    .iter()
                    .any(|(_, b)| b.stmts.iter().any(has_break_continue))
                || else_block
                    .as_ref()
                    .is_some_and(|b| b.stmts.iter().any(has_break_continue))
        }
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForIn { body, .. } => {
            // break/continue 归属于该循环自身，不会逃逸到状态机外层。
            let _ = body;
            false
        }
        Stmt::Block(b) => b.stmts.iter().any(has_break_continue),
        Stmt::Match { arms, .. } => arms.iter().any(|a| match &a.body {
            MatchBody::Expr(_) => false,
            MatchBody::Block(b) => b.stmts.iter().any(has_break_continue),
        }),
        Stmt::Unsafe { block, .. } => block.stmts.iter().any(has_break_continue),
        Stmt::Attributed { stmt, .. } => has_break_continue(stmt),
        _ => false,
    }
}

pub(super) fn check_supported(
    stmts: &[Stmt],
    async_fns: &HashMap<Symbol, Vec<(Symbol, Type)>>,
) -> bool {
    for st in stmts {
        match st {
            Stmt::Assign { name: _, value, .. } => {
                if let Some((g, args)) = await_parts(value) {
                    let Some(params) = async_fns.get(&g) else {
                        return false;
                    };
                    if args.iter().any(expr_has_await) {
                        return false;
                    }
                    if params.iter().any(|(_, t)| type_default(t).is_none()) {
                        return false;
                    }
                } else if expr_has_await(value) {
                    return false;
                }
            }
            Stmt::ExprStmt { expr, .. } if expr_has_await(expr) => return false,
            Stmt::Return { value, .. } => {
                if value.as_ref().map_or(false, |e| expr_has_await(e)) {
                    return false;
                }
            }
            Stmt::FieldAssign { object, value, .. } => {
                if expr_has_await(object) || expr_has_await(value) {
                    return false;
                }
            }
            Stmt::IndexAssign {
                object,
                index,
                value,
                ..
            } => {
                if expr_has_await(object) || expr_has_await(index) || expr_has_await(value) {
                    return false;
                }
            }
            Stmt::DerefAssign { target, value, .. } => {
                if expr_has_await(target) || expr_has_await(value) {
                    return false;
                }
            }
            Stmt::If {
                cond,
                then_block,
                elifs,
                else_block,
                ..
            } => {
                if then_block.tail.is_some()
                    || elifs.iter().any(|(_, b)| b.tail.is_some())
                    || else_block.as_ref().is_some_and(|b| b.tail.is_some())
                {
                    return false;
                }
                if expr_has_await(cond) {
                    return false;
                }
                if !check_supported(&then_block.stmts, async_fns) {
                    return false;
                }
                for (c, b) in elifs {
                    if expr_has_await(c) || !check_supported(&b.stmts, async_fns) {
                        return false;
                    }
                }
                if let Some(b) = else_block {
                    if !check_supported(&b.stmts, async_fns) {
                        return false;
                    }
                }
            }
            Stmt::While { cond, body, .. } => {
                if body.tail.is_some() {
                    return false;
                }
                if expr_has_await(cond) || !check_supported(&body.stmts, async_fns) {
                    return false;
                }
            }
            Stmt::Block(b) => {
                if b.tail.is_some() || !check_supported(&b.stmts, async_fns) {
                    return false;
                }
            }
            Stmt::For { .. } | Stmt::ForIn { .. } => {
                if stmt_has_await(st) {
                    return false;
                }
            }
            Stmt::Match { .. } | Stmt::Unsafe { .. } | Stmt::Attributed { .. } => {
                // 这些构造不拆状态；若内部 break/continue 指向外层 await 循环，
                // 原样放进状态块会错误跳出 poll 的 while true。
                if stmt_has_await(st) || has_break_continue(st) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}
