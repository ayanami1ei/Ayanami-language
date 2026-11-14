//! match guard 自由变量检查：只允许引用当前模式的绑定。
use std::collections::HashSet;

use crate::intern::Symbol;
use crate::parser::ast::{Expr, Pattern};

pub(super) fn guard_ok(e: &Expr, binds: &HashSet<Symbol>) -> bool {
    match e {
        Expr::Literal(..) | Expr::Suffixed { .. } | Expr::Null(..) => true,
        Expr::Ident(n, _) => binds.contains(n),
        Expr::Binary { lhs, rhs, .. } => guard_ok(lhs, binds) && guard_ok(rhs, binds),
        Expr::Unary { arg, .. } => guard_ok(arg, binds),
        Expr::Cast { expr, .. }
        | Expr::Move(expr, _)
        | Expr::Clone(expr, _)
        | Expr::ToUnique(expr, _)
        | Expr::TryOp(expr, _) => guard_ok(expr, binds),
        Expr::Ref(expr, _, _) => guard_ok(expr, binds),
        Expr::FnCall { args, .. } => args.iter().all(|a| guard_ok(a, binds)),
        Expr::MethodCall { object, args, .. } => {
            guard_ok(object, binds) && args.iter().all(|a| guard_ok(a, binds))
        }
        Expr::FieldAccess { object, .. } => guard_ok(object, binds),
        Expr::Index { object, index, .. } => guard_ok(object, binds) && guard_ok(index, binds),
        Expr::ArrayLiteral(elems, _) => elems.iter().all(|a| guard_ok(a, binds)),
        Expr::ArraySized { count, .. } => guard_ok(count, binds),
        Expr::ArrayRepeat { value, count, .. } => guard_ok(value, binds) && guard_ok(count, binds),
        Expr::CallExpr { target, args, .. } => {
            guard_ok(target, binds) && args.iter().all(|a| guard_ok(a, binds))
        }
        Expr::StructLiteral { fields, .. } => fields.iter().all(|(_, e)| guard_ok(e, binds)),
        Expr::EnumConstruct {
            tuple_args,
            named_args,
            ..
        } => {
            tuple_args.iter().all(|a| guard_ok(a, binds))
                && named_args.iter().all(|(_, e)| guard_ok(e, binds))
        }
        // 控制流/宏/汇编 guard 暂不支持。
        Expr::If { .. }
        | Expr::Match { .. }
        | Expr::Lambda { .. }
        | Expr::MacroCall { .. }
        | Expr::Asm { .. } => false,
    }
}

pub(super) fn pattern_bindings(p: &Pattern) -> Vec<Symbol> {
    let mut out = Vec::new();
    for b in p.bindings() {
        // 无参枚举变体在模式 AST 中是带点绑定名（如 Option.None），不是变量绑定。
        if !b.as_str().contains('.') && !out.contains(&b) {
            out.push(b);
        }
    }
    out
}

pub(super) fn or_consistent(p: &Pattern) -> bool {
    match p {
        Pattern::Or(ps) => {
            let Some(first) = ps.first() else { return true };
            let want = pattern_bindings(first);
            ps.iter().all(|q| pattern_bindings(q) == want) && ps.iter().all(or_consistent)
        }
        Pattern::Enum { args, .. } => args.iter().all(or_consistent),
        Pattern::Struct { fields, .. } => fields.iter().all(|(_, p)| or_consistent(p)),
        _ => true,
    }
}
