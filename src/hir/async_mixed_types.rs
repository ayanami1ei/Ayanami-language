//! async mixed 过渡桥：AST 轻量类型推断与 HIR 类型补充。
use std::collections::HashMap;

use crate::hir::async_transform_util::ty_src;
use crate::hir::ir::HirType;
use crate::intern::Symbol;
use crate::parser::ast::{Expr, Literal, Type};
use crate::span::Span;

pub(super) fn infer(e: &Expr, env: &HashMap<Symbol, Type>, span: Span) -> Option<Type> {
    match e {
        Expr::Ident(n, _) => env.get(n).cloned(),
        Expr::Literal(l) => literal_type(l, span),
        Expr::Cast { ty, .. } => Some(ty.clone()),
        Expr::Binary { op, lhs, rhs, .. } => {
            use crate::parser::ast::BinaryOp::*;
            let lt = infer(lhs, env, span)?;
            let rt = infer(rhs, env, span)?;
            match op {
                Add | Sub | Mul | Div | Mod | BitAnd | BitOr | BitXor | Shl | Shr => {
                    if is_float(&lt) || is_float(&rt) { Some(Type::Float(span)) }
                    else if is_int(&lt) && is_int(&rt) { Some(Type::Int(span)) }
                    else { None }
                }
                Eq | Neq | Lt | Gt | Le | Ge | And | Or => Some(Type::Bool(span)),
            }
        }
        Expr::Unary { op, arg, .. } => match op {
            crate::parser::ast::UnaryOp::Neg => infer(arg, env, span),
            crate::parser::ast::UnaryOp::Not => Some(Type::Bool(span)),
            crate::parser::ast::UnaryOp::BitNot => infer(arg, env, span),
            crate::parser::ast::UnaryOp::Deref => None,
        },
        Expr::Move(inner, _) | Expr::Clone(inner, _) => infer(inner, env, span),
        _ => None,
    }
}

fn literal_type(l: &Literal, span: Span) -> Option<Type> {
    Some(match l {
        Literal::Int(..) => Type::Int(span),
        Literal::Float(..) => Type::Float(span),
        Literal::Bool(..) => Type::Bool(span),
        Literal::String(..) => Type::Named(Symbol::intern("String"), span),
        Literal::Char(..) => Type::Char(span),
    })
}

fn is_int(t: &Type) -> bool {
    if matches!(t, Type::Int(_)) { return true; }
    let Type::Named(n, _) = t else { return false };
    let s = n.as_str();
    matches!(s.as_str(), "i8"|"i16"|"i32"|"i64"|"i128"|"u8"|"u16"|"u32"|"u64"|"u128"|"usize"|"isize")
}

fn is_float(t: &Type) -> bool {
    matches!(t, Type::Float(_))
        || matches!(t, Type::Named(n, _) if n.as_str() == "f32" || n.as_str() == "f64")
}

pub(super) fn same_type(a: &Type, b: &Type) -> bool {
    match (ty_src(a), ty_src(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

pub(super) fn type_default(t: &Type) -> Option<&'static str> {
    match t {
        Type::Int(_) => Some("0"),
        Type::Float(_) => Some("0.0"),
        Type::Bool(_) => Some("false"),
        Type::Char(_) => Some("'\\0'"),
        Type::Named(n, _) if n.as_str() == "String" => Some("\"\""),
        Type::Named(n, _) if is_int(t) => Some("0"),
        Type::Named(n, _) if n.as_str() == "f32" || n.as_str() == "f64" => Some("0.0"),
        _ => None,
    }
}

pub(super) fn hir_to_ast(t: &HirType, span: Span) -> Option<Type> {
    Some(match t {
        HirType::Int => Type::Int(span),
        HirType::Float => Type::Float(span),
        HirType::F32 => Type::Named(Symbol::intern("f32"), span),
        HirType::Bool => Type::Bool(span),
        HirType::Char => Type::Char(span),
        HirType::IntN { bits, signed } => Type::Named(
            Symbol::intern(&format!("{}{}", if *signed { "i" } else { "u" }, bits)), span),
        HirType::Named(n) if n.as_str() == "String" => Type::Named(*n, span),
        _ => return None,
    })
}
