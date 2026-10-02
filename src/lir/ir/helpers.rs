use super::*;

pub fn sanitize_name(name: &str) -> String {
    name.replace('<', "_lt_").replace('>', "_gt_")
        .replace(',', "_c_").replace('[', "_lb_").replace(']', "_rb_").replace(' ', "_")
}

pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Int(0), ty) if is_pointer_type(ty) && !matches!(ty, HirType::Named(_)) => "null".into(),
        (HirLiteral::Int(0), HirType::Named(_)) => "zeroinitializer".into(),
        (HirLiteral::Int(n), _) => format!("{}", n),
        (HirLiteral::Float(n), _) => {
            let s = format!("{}", n);
            if !s.contains('.') { format!("{}.0", s) } else { s }
        }
        (HirLiteral::Char(c), _) => format!("{}", *c as u8),
        (HirLiteral::Bool(b), _) => if *b { "1".into() } else { "0".into() },
        (HirLiteral::String(_), _) => "null".into(),
    }
}

pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str {
    match ty {
        HirType::Int => "8",
        HirType::Float => "8",
        HirType::Char => "1",
        HirType::Bool => "1",
        HirType::Void => "0",
        HirType::Named(_) | HirType::FatPtr { .. } => "16",
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => llvm_type_size(inner),
        HirType::Array(_) | HirType::ArraySized(_, _) => "16",
        HirType::FnPtr(..) => "8",
        HirType::Ref(_, _) => "16",
    }
}

/// Compute the actual size of a Named struct type from its field definitions.
pub(crate) fn struct_llvm_size(ty: &HirType, struct_defs: &std::collections::HashMap<Symbol, Vec<(Symbol, HirType)>>) -> String {
    if let HirType::Named(name) = ty {
        if let Some(fields) = struct_defs.get(name) {
            let total: u64 = fields.iter().map(|(_, ft)| {
                let s = llvm_type_size(ft);
                let n: u64 = s.parse().unwrap_or(8);
                if matches!(ft, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)) { 8u64 } else { n }
            }).sum();
            return total.to_string();
        }
    }
    if let HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) = ty {
        return struct_llvm_size(inner, struct_defs);
    }
    llvm_type_size(ty).to_string()
}

pub(super) fn needs_heap_ops(ty: &HirType) -> bool {
    match ty {
        HirType::FatPtr { .. } => true,
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
            matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. })
        }
        HirType::Named(_) => false,
        HirType::Array(_) | HirType::ArraySized(_, _) => false,
        HirType::Ref(_, _) => false,
        _ => false,
    }
}

pub(super) fn is_pointer_type(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_)
        | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_)
        | HirType::Ref(_, _) | HirType::FnPtr(..)
    )
}
