//! async mixed 过渡桥：语句/表达式源码渲染与局部→字段重写。
use std::collections::HashSet;

use crate::intern::Symbol;
use crate::parser::ast::{Expr, Stmt};

pub(super) fn render_sync(st: &Stmt, promoted: &HashSet<Symbol>, final_seg: bool) -> Option<String> {
    match st {
        Stmt::Assign { name, value, .. } => {
            let r = render_expr(value, promoted)?;
            Some(format!("self.__v_{} = {};", name.as_str(), r))
        }
        Stmt::ExprStmt { expr, .. } => Some(format!("{};", render_expr(expr, promoted)?)),
        Stmt::Return { value: Some(e), .. } if final_seg => {
            Some(format!("return Poll::Ready({});", render_expr(e, promoted)?))
        }
        _ => None,
    }
}

pub(super) fn render_expr(e: &Expr, promoted: &HashSet<Symbol>) -> Option<String> {
    Some(crate::formatter::format_expr(&rewrite(e, promoted)?))
}

fn rewrite(e: &Expr, promoted: &HashSet<Symbol>) -> Option<Expr> {
    Some(match e {
        Expr::Ident(n, span) if promoted.contains(n) => Expr::FieldAccess {
            object: Box::new(Expr::Ident(Symbol::intern("self"), *span)),
            field: Symbol::intern(&format!("__v_{}", n.as_str())),
            span: *span,
        },
        Expr::Ident(..) | Expr::Literal(..) | Expr::Null(..) | Expr::Suffixed { .. } => e.clone(),
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary {
            op: *op, lhs: Box::new(rewrite(lhs, promoted)?),
            rhs: Box::new(rewrite(rhs, promoted)?), span: *span,
        },
        Expr::Unary { op, arg, span } => Expr::Unary {
            op: *op, arg: Box::new(rewrite(arg, promoted)?), span: *span,
        },
        Expr::Cast { expr, ty, span } => Expr::Cast {
            expr: Box::new(rewrite(expr, promoted)?), ty: ty.clone(), span: *span,
        },
        Expr::Move(inner, span) => Expr::Move(Box::new(rewrite(inner, promoted)?), *span),
        Expr::Clone(inner, span) => Expr::Clone(Box::new(rewrite(inner, promoted)?), *span),
        Expr::FnCall { name, args, generic_args, span } => Expr::FnCall {
            name: *name, args: args.iter().map(|a| rewrite(a, promoted)).collect::<Option<Vec<_>>>()?,
            generic_args: generic_args.clone(), span: *span,
        },
        Expr::MethodCall { object, method, args, span } => Expr::MethodCall {
            object: Box::new(rewrite(object, promoted)?), method: *method,
            args: args.iter().map(|a| rewrite(a, promoted)).collect::<Option<Vec<_>>>()?, span: *span,
        },
        Expr::FieldAccess { object, field, span } => Expr::FieldAccess {
            object: Box::new(rewrite(object, promoted)?), field: *field, span: *span,
        },
        Expr::Index { object, index, span } => Expr::Index {
            object: Box::new(rewrite(object, promoted)?), index: Box::new(rewrite(index, promoted)?), span: *span,
        },
        Expr::Ref(inner, mutable, span) => Expr::Ref(Box::new(rewrite(inner, promoted)?), *mutable, *span),
        _ => return None,
    })
}
