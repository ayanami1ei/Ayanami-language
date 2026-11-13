//! async CFG 的局部/类型辅助。
use std::collections::HashSet;

use crate::hir::async_mixed_types::type_default;
use crate::hir::async_transform_util::ty_src;
use crate::intern::Symbol;
use crate::parser::ast::{Stmt, Type};

pub(super) fn ast_copy(t: &Type) -> bool {
    match t {
        Type::Int(_) | Type::Float(_) | Type::Char(_) | Type::Bool(_) => true,
        Type::Void(_) | Type::Never(_) => true,
        Type::Named(n, _) => {
            let s = n.as_str();
            matches!(
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
                    | "f32"
                    | "f64"
            )
        }
        Type::Ref(..) => true,
        _ => false,
    }
}

pub(super) fn field_name(n: Symbol) -> Symbol {
    Symbol::intern(&format!("__v_{}", n.as_str()))
}

pub(super) fn bad_local_name(n: Symbol) -> bool {
    let s = n.as_str();
    s == "self" || s.starts_with("__async_")
}

pub(super) fn local_default(t: &Type) -> Option<String> {
    if let Type::Generic(n, _, _) = t {
        if n.as_str() == "Option" {
            return Some("Option::None".into());
        }
    }
    type_default(t).map(|s| s.to_string())
}

pub(super) fn source_ty(t: &Type) -> Option<String> {
    let s = ty_src(t)?;
    Some(s.replace('<', "[").replace('>', "]"))
}

pub(super) fn collect_assigned(stmts: &[Stmt], out: &mut HashSet<Symbol>) {
    for st in stmts {
        match st {
            Stmt::Assign { name, .. } => {
                out.insert(*name);
            }
            Stmt::If {
                then_block,
                elifs,
                else_block,
                ..
            } => {
                collect_assigned(&then_block.stmts, out);
                for (_, b) in elifs {
                    collect_assigned(&b.stmts, out);
                }
                if let Some(b) = else_block {
                    collect_assigned(&b.stmts, out);
                }
            }
            Stmt::While { body, .. } | Stmt::Block(body) => {
                collect_assigned(&body.stmts, out);
            }
            Stmt::For { iterator, body, .. } => {
                out.insert(*iterator);
                collect_assigned(&body.stmts, out);
            }
            Stmt::ForIn { iterator, body, .. } => {
                out.insert(*iterator);
                collect_assigned(&body.stmts, out);
            }
            Stmt::Unsafe { block, .. } => collect_assigned(&block.stmts, out),
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let crate::parser::ast::stmt::MatchBody::Block(b) = &arm.body {
                        collect_assigned(&b.stmts, out);
                    }
                }
            }
            Stmt::Attributed { stmt, .. } => {
                collect_assigned(std::slice::from_ref(stmt.as_ref()), out)
            }
            _ => {}
        }
    }
}
