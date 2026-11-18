//! A1b：async 展开用的类型串/默认值辅助。
use crate::intern::Symbol;
use crate::parser::ast::{Attr, Stmt, Type};

/// 生成源码用的类型串（crate 内可见）（仅 v1 允许的类型）。
pub(crate) fn ty_src(ty: &Type) -> Option<String> {
    match ty {
        Type::Int(_) => Some("int".into()),
        Type::Float(_) => Some("float".into()),
        Type::Char(_) => Some("char".into()),
        Type::Bool(_) => Some("bool".into()),
        Type::Named(s, _) => Some(s.as_str().to_string()),
        Type::Ref(inner, mutable, _) => Some(format!(
            "ref{} {}",
            if *mutable { " mut" } else { "" },
            ty_src(inner)?
        )),
        Type::Generic(n, args, _) => {
            let mut out = Vec::with_capacity(args.len());
            for a in args { out.push(ty_src(a)?); }
            Some(format!("{}[{}]", n.as_str(), out.join(", ")))
        }
        _ => None,
    }
}

/// await 载荷类型 → (字段类型串, 默认值字面量)；v1 仅基础类型/String。
pub(crate) fn await_default(ty: &Type) -> Option<(&'static str, &'static str)> {
    match ty {
        Type::Int(_) => Some(("int", "0")),
        Type::Float(_) => Some(("float", "0.0")),
        Type::Bool(_) => Some(("bool", "false")),
        Type::Named(s, _) if s.as_str() == "String" => Some(("String", "\"\"")),
        _ => None,
    }
}

pub(crate) fn type_name(ty: &Type) -> String {
    match ty {
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Bool(_) => "bool".into(),
        Type::Char(_) => "char".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Generic(n, _, _) => format!("{}[...]", n.as_str()),
        _ => "?".into(),
    }
}

pub(crate) fn attach_qualified_attrs(out: &mut [Stmt], name: Symbol, attrs: &[Attr]) {
    let keep: Vec<Attr> = attrs.iter()
        .filter(|a| !a.qualifier.is_empty())
        .cloned()
        .collect();
    if keep.is_empty() {
        return;
    }
    for stmt in out {
        if let Stmt::FnDecl { name: n, attrs: a, .. } = stmt {
            if *n == name {
                *a = keep;
                return;
            }
        }
    }
}
