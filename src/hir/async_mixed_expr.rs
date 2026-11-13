//! async mixed 过渡桥：语句/表达式源码渲染与局部→字段重写。
use std::collections::{HashMap, HashSet};

use crate::intern::Symbol;
use crate::parser::ast::{Expr, Literal, Stmt, Type};
use crate::span::Span;

pub(super) fn render_sync(st: &Stmt, promoted: &HashSet<Symbol>) -> Option<String> {
    match st {
        Stmt::Assign { name, value, .. } => {
            let r = render_expr(value, promoted)?;
            if promoted.contains(name) {
                Some(format!("self.__v_{} = {};", name.as_str(), r))
            } else {
                Some(format!("{} = {};", name.as_str(), r))
            }
        }
        Stmt::ExprStmt { expr, .. } => Some(format!("{};", render_expr(expr, promoted)?)),
        Stmt::Return { value: Some(e), .. } => Some(format!(
            "return Poll::Ready({});",
            render_expr(e, promoted)?
        )),
        Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            let mut s = format!(
                "if {} {{\n{}\n}}",
                render_expr(cond, promoted)?,
                render_block(&then_block.stmts, promoted)?
            );
            for (c, b) in elifs {
                s.push_str(&format!(
                    " elif {} {{\n{}\n}}",
                    render_expr(c, promoted)?,
                    render_block(&b.stmts, promoted)?
                ));
            }
            if let Some(b) = else_block {
                s.push_str(&format!(
                    " else {{\n{}\n}}",
                    render_block(&b.stmts, promoted)?
                ));
            }
            Some(s)
        }
        Stmt::While { cond, body, .. } => Some(format!(
            "while {} {{\n{}\n}}",
            render_expr(cond, promoted)?,
            render_block(&body.stmts, promoted)?
        )),
        Stmt::Break { .. } => Some("break;".into()),
        Stmt::Continue { .. } => Some("continue;".into()),
        Stmt::Block(b) => render_block(&b.stmts, promoted),
        _ => None,
    }
}

fn render_block(stmts: &[Stmt], promoted: &HashSet<Symbol>) -> Option<String> {
    let mut out = Vec::new();
    for st in stmts {
        out.push(render_sync(st, promoted)?);
    }
    Some(out.join("\n"))
}

pub(super) fn render_expr(e: &Expr, promoted: &HashSet<Symbol>) -> Option<String> {
    Some(crate::formatter::format_expr(&rewrite(e, promoted)?))
}

/// 构造器占位实参：把提升局部替换为其类型默认值，其余（全局/字面量）保留。
pub(super) fn render_default(e: &Expr, env: &HashMap<Symbol, Type>) -> Option<String> {
    let r = rewrite_with(e, &mut |n, span| default_expr(env.get(n)?, span))?;
    Some(crate::formatter::format_expr(&r))
}

fn default_expr(t: &Type, span: Span) -> Option<Expr> {
    Some(match t {
        Type::Int(_) => Expr::Literal(Literal::Int(0, span)),
        Type::Float(_) => Expr::Literal(Literal::Float(0.0, span)),
        Type::Bool(_) => Expr::Literal(Literal::Bool(false, span)),
        Type::Named(n, _) => {
            let s = n.as_str();
            if s == "String" {
                Expr::Literal(Literal::String(String::new(), span))
            } else if matches!(
                s.as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "isize"
            ) {
                Expr::Literal(Literal::Int(0, span))
            } else if s == "f32" || s == "f64" {
                Expr::Literal(Literal::Float(0.0, span))
            } else {
                return None;
            }
        }
        _ => return None,
    })
}

fn rewrite(e: &Expr, promoted: &HashSet<Symbol>) -> Option<Expr> {
    rewrite_with(e, &mut |n, span| {
        if promoted.contains(n) {
            Some(Expr::FieldAccess {
                object: Box::new(Expr::Ident(Symbol::intern("self"), span)),
                field: Symbol::intern(&format!("__v_{}", n.as_str())),
                span,
            })
        } else {
            None
        }
    })
}

fn rewrite_with(e: &Expr, subst: &mut dyn FnMut(&Symbol, Span) -> Option<Expr>) -> Option<Expr> {
    Some(match e {
        Expr::Ident(n, span) => subst(n, *span).unwrap_or_else(|| e.clone()),
        Expr::Literal(..) | Expr::Null(..) | Expr::Suffixed { .. } => e.clone(),
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary {
            op: *op,
            lhs: Box::new(rewrite_with(lhs, subst)?),
            rhs: Box::new(rewrite_with(rhs, subst)?),
            span: *span,
        },
        Expr::Unary { op, arg, span } => Expr::Unary {
            op: *op,
            arg: Box::new(rewrite_with(arg, subst)?),
            span: *span,
        },
        Expr::Cast { expr, ty, span } => Expr::Cast {
            expr: Box::new(rewrite_with(expr, subst)?),
            ty: ty.clone(),
            span: *span,
        },
        Expr::Move(inner, span) => Expr::Move(Box::new(rewrite_with(inner, subst)?), *span),
        Expr::Clone(inner, span) => Expr::Clone(Box::new(rewrite_with(inner, subst)?), *span),
        Expr::FnCall {
            name,
            args,
            generic_args,
            span,
        } => Expr::FnCall {
            name: *name,
            args: args
                .iter()
                .map(|a| rewrite_with(a, subst))
                .collect::<Option<Vec<_>>>()?,
            generic_args: generic_args.clone(),
            span: *span,
        },
        Expr::MethodCall {
            object,
            method,
            args,
            span,
        } => Expr::MethodCall {
            object: Box::new(rewrite_with(object, subst)?),
            method: *method,
            args: args
                .iter()
                .map(|a| rewrite_with(a, subst))
                .collect::<Option<Vec<_>>>()?,
            span: *span,
        },
        Expr::FieldAccess {
            object,
            field,
            span,
        } => Expr::FieldAccess {
            object: Box::new(rewrite_with(object, subst)?),
            field: *field,
            span: *span,
        },
        Expr::Index {
            object,
            index,
            span,
        } => Expr::Index {
            object: Box::new(rewrite_with(object, subst)?),
            index: Box::new(rewrite_with(index, subst)?),
            span: *span,
        },
        Expr::Ref(inner, mutable, span) => {
            Expr::Ref(Box::new(rewrite_with(inner, subst)?), *mutable, *span)
        }
        _ => return None,
    })
}

/// 渲染表达式，把 `fields` 中的局部替换为 `self.<name>`（形状模块的字段命名）。
pub(super) fn render_field_expr(e: &Expr, fields: &HashSet<Symbol>) -> Option<String> {
    let r = rewrite_with(e, &mut |n, span| {
        if fields.contains(n) {
            Some(Expr::FieldAccess {
                object: Box::new(Expr::Ident(Symbol::intern("self"), span)),
                field: *n,
                span,
            })
        } else {
            None
        }
    })?;
    Some(crate::formatter::format_expr(&r))
}
