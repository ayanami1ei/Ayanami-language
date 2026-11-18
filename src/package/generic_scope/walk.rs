use std::collections::HashSet;

use crate::intern::Symbol;
use crate::parser::ast::{Expr, Stmt};

pub(super) fn calls_in_block(
    block: &crate::parser::ast::Block,
    out: &mut HashSet<Symbol>,
) {
    for stmt in &block.stmts {
        calls_in_stmt(stmt, out);
    }
    if let Some(tail) = &block.tail {
        calls_in_expr(tail, out);
    }
}

fn calls_in_stmt(stmt: &Stmt, out: &mut HashSet<Symbol>) {
    match stmt {
        Stmt::Block(block) | Stmt::Unsafe { block, .. } => calls_in_block(block, out),
        Stmt::DerefAssign { target, value, .. } => {
            calls_in_expr(target, out);
            calls_in_expr(value, out);
        }
        Stmt::FnDecl { body, .. } => calls_in_block(body, out),
        Stmt::StaticDecl { value, .. } | Stmt::ConstDecl { value, .. } => calls_in_expr(value, out),
        Stmt::Assign { value, .. } => calls_in_expr(value, out),
        Stmt::FieldAssign { object, value, .. } => {
            calls_in_expr(object, out);
            calls_in_expr(value, out);
        }
        Stmt::IndexAssign { object, index, value, .. } => {
            calls_in_expr(object, out);
            calls_in_expr(index, out);
            calls_in_expr(value, out);
        }
        Stmt::Return { value, .. } => {
            if let Some(value) = value {
                calls_in_expr(value, out);
            }
        }
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            calls_in_expr(cond, out);
            calls_in_block(then_block, out);
            for (cond, block) in elifs {
                calls_in_expr(cond, out);
                calls_in_block(block, out);
            }
            if let Some(block) = else_block {
                calls_in_block(block, out);
            }
        }
        Stmt::For { start, end, step, body, .. } => {
            calls_in_expr(start, out);
            calls_in_expr(end, out);
            if let Some(step) = step {
                calls_in_expr(step, out);
            }
            calls_in_block(body, out);
        }
        Stmt::ForIn { iterable, body, .. } => {
            calls_in_expr(iterable, out);
            calls_in_block(body, out);
        }
        Stmt::While { cond, body, .. } => {
            calls_in_expr(cond, out);
            calls_in_block(body, out);
        }
        Stmt::Match { value, arms, .. } => {
            calls_in_expr(value, out);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    calls_in_expr(guard, out);
                }
                match &arm.body {
                    crate::parser::ast::stmt::MatchBody::Expr(expr) => calls_in_expr(expr, out),
                    crate::parser::ast::stmt::MatchBody::Block(block) => calls_in_block(block, out),
                }
            }
        }
        Stmt::ExprStmt { expr, .. } => calls_in_expr(expr, out),
        Stmt::Namespace { items, .. } => {
            for item in items {
                calls_in_stmt(item, out);
            }
        }
        Stmt::ImplBlock { methods, .. } => {
            for method in methods {
                calls_in_stmt(method, out);
            }
        }
        Stmt::Attributed { stmt, .. } => calls_in_stmt(stmt, out),
        _ => {}
    }
}

fn calls_in_expr(expr: &Expr, out: &mut HashSet<Symbol>) {
    match expr {
        Expr::FnCall { name, args, .. } => {
            out.insert(*name);
            for arg in args {
                calls_in_expr(arg, out);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            calls_in_expr(lhs, out);
            calls_in_expr(rhs, out);
        }
        Expr::Unary { arg, .. }
        | Expr::Cast { expr: arg, .. }
        | Expr::Move(arg, _)
        | Expr::Clone(arg, _)
        | Expr::ToUnique(arg, _)
        | Expr::Ref(arg, _, _)
        | Expr::TryOp(arg, _) => calls_in_expr(arg, out),
        Expr::If { cond, then_block, elifs, else_block, .. } => {
            calls_in_expr(cond, out);
            calls_in_block(then_block, out);
            for (cond, block) in elifs {
                calls_in_expr(cond, out);
                calls_in_block(block, out);
            }
            if let Some(block) = else_block {
                calls_in_block(block, out);
            }
        }
        Expr::MethodCall { object, args, .. } => {
            calls_in_expr(object, out);
            for arg in args {
                calls_in_expr(arg, out);
            }
        }
        Expr::FieldAccess { object, .. } => calls_in_expr(object, out),
        Expr::Index { object, index, .. } => {
            calls_in_expr(object, out);
            calls_in_expr(index, out);
        }
        Expr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                calls_in_expr(value, out);
            }
        }
        Expr::ArrayLiteral(values, _) => {
            for value in values {
                calls_in_expr(value, out);
            }
        }
        Expr::ArraySized { count, .. } => calls_in_expr(count, out),
        Expr::ArrayRepeat { value, count, .. } => {
            calls_in_expr(value, out);
            calls_in_expr(count, out);
        }
        Expr::Asm { outputs, inputs, .. } => {
            for (_, value) in outputs.iter().chain(inputs) {
                calls_in_expr(value, out);
            }
        }
        Expr::CallExpr { target, args, .. } => {
            calls_in_expr(target, out);
            for arg in args {
                calls_in_expr(arg, out);
            }
        }
        Expr::Match { value, arms, .. } => {
            calls_in_expr(value, out);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    calls_in_expr(guard, out);
                }
                match &arm.body {
                    crate::parser::ast::stmt::MatchBody::Expr(expr) => calls_in_expr(expr, out),
                    crate::parser::ast::stmt::MatchBody::Block(block) => calls_in_block(block, out),
                }
            }
        }
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for arg in tuple_args {
                calls_in_expr(arg, out);
            }
            for (_, arg) in named_args {
                calls_in_expr(arg, out);
            }
        }
        Expr::Lambda { body, .. } => calls_in_block(body, out),
        Expr::MacroCall { args, .. } => {
            for arg in args {
                match arg {
                    crate::parser::ast::expr::MacroArg::Expr(expr) => calls_in_expr(expr, out),
                    crate::parser::ast::expr::MacroArg::Block(block) => calls_in_block(block, out),
                }
            }
        }
        Expr::Ident(..) | Expr::Literal(_) | Expr::Suffixed { .. } | Expr::Null(_) => {}
    }
}
