//! 类型 → 源码文本（从 expr.rs 拆出，保持文件 ≤300 行）。

use super::*;
use super::helpers::write_fn_type;

pub(super) fn write_type(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::FnPtr(params, ret, _) => write_fn_type("fn", params, ret),
        Type::Closure(params, ret, _, once) => write_fn_type(if *once { "FnOnce" } else { "Fn" }, params, ret),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Never(_) => "!".into(),
        Type::RawPtr(inner, _) => format!("*{}", write_type(inner)),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Array(inner, _) => format!("[{}]", write_type(inner)),
        Type::ArraySized(inner, n, _) => format!("[{}; {}]", write_type(inner), n),
        Type::Unique(inner, _) => format!("unique {}", write_type(inner)),
        Type::Generic(name, args, _) => format!("{}[{}]", name, args.iter().map(|a| write_type(a)).collect::<Vec<_>>().join(", ")),
        Type::Ref(inner, mutable, _) => format!("ref {}{}", if *mutable { "mut " } else { "" }, write_type(inner)),
        Type::Self_(_) => "Self".into(),
    }
}

