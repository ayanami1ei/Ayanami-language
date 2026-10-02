use super::*;

pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String {
    match ty {
        Type::Default => "?".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Generic(name, args, _) => {
            let a: Vec<String> = args.iter().map(|a| type_to_string_generic(a, interfaces)).collect();
            format!("{}<{}>", name, a.join(","))
        }
        Type::Array(inner, _) => format!("[{}]", type_to_string_generic(inner, interfaces)),
        Type::Ref(inner, mutable, _) => format!("ref{}{}",
            if *mutable { " mut" } else { "" },
            type_to_string_generic(inner, interfaces)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string_generic(inner, interfaces)),
        Type::Shared(inner, _) => format!("shared {}", type_to_string_generic(inner, interfaces)),
        Type::Weak(inner, _) => format!("weak {}", type_to_string_generic(inner, interfaces)),
        Type::FnPtr(..) => "fn(...)".to_string(),
        Type::Self_(_) => "Self".into(),
    }
}

pub(crate) fn sig_str_to_hir(s: &str) -> HirType {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("shared ") {
        HirType::Shared(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("unique ") {
        HirType::Unique(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("weak ") {
        HirType::Weak(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        HirType::Array(Box::new(sig_str_to_hir(inner)))
    } else {
        match s {
            "int" => HirType::Int,
            "float" => HirType::Float,
            "char" => HirType::Char,
            "bool" => HirType::Bool,
            "void" => HirType::Void,
            other => HirType::Named(Symbol::intern(other)),
        }
    }
}

    /// 将 AST 类型节点转换为 HIR 类型（含接口信息）
fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool {
    match inner_hir {
        HirType::Named(n) if interfaces.contains_key(n) => true,
        HirType::Named(n) => {
            let base = strip_generic_name(n);
            base != *n && interfaces.contains_key(&base)
        }
        _ => false,
    }
}

pub(crate) fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType {
    match ty {
        Type::Default | Type::Int(_) => HirType::Int,
        Type::Float(_) => HirType::Float,
        Type::Char(_) => HirType::Char,
        Type::Bool(_) => HirType::Bool,
        Type::Void(_) => HirType::Void,
        Type::Array(inner, _) => HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Generic(name, args, _) => {
            // Encode generic instantiation as a unique named type
            let args_str: Vec<String> = args.iter()
                .map(|a| type_to_string_generic(a, interfaces))
                .collect();
            HirType::Named(Symbol::intern(&format!("{}<{}>", name, args_str.join(","))))
        }
        Type::Named(s, _) => {
            let name = s.as_str();
            if name == "int" { HirType::Int }
            else if name == "float" { HirType::Float }
            else if name == "char" { HirType::Char }
            else if name == "void" { HirType::Void }
            else if name == "bool" { HirType::Bool }
            else { HirType::Named(*s) }
        }
        Type::Unique(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if is_iface_type(&inner_hir, interfaces) {
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Unique(Box::new(HirType::Void))) }
            } else {
                HirType::Unique(Box::new(inner_hir))
            }
        }
        Type::Shared(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if is_iface_type(&inner_hir, interfaces) {
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Shared(Box::new(HirType::Void))) }
            } else {
                HirType::Shared(Box::new(inner_hir))
            }
        }
        Type::Weak(inner, _) => HirType::Weak(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Ref(inner, mutable, _) => HirType::Ref(Box::new(ast_type_to_hir(inner, interfaces)), *mutable),
        Type::FnPtr(params, ret, _) => HirType::FnPtr(
            params.iter().map(|p| ast_type_to_hir(p, interfaces)).collect(),
            Box::new(ast_type_to_hir(ret, interfaces)),
        ),
        Type::Self_(_) => {
            // Self_ should not appear outside impl blocks since the parser
            // already fills in the concrete type
            HirType::Void
        }
    }
}

    /// 从 HirType 中提取命名类型的名称（剥去所有权包装）
pub(crate) fn extract_named(ty: &HirType) -> Option<&Symbol> {
    match ty {
        HirType::Named(s) => Some(s),
        _ => None,
    }
}

    /// 将 HirType 格式化为可读字符串（用于错误消息和调试输出）
pub(crate) fn hir_type_display(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::Named(s) => s.as_str().to_string(),
        HirType::Unique(inner) => format!("unique {}", hir_type_display(inner)),
        HirType::Shared(inner) => format!("shared {}", hir_type_display(inner)),
        HirType::FnPtr(..) => "fn(...)".into(),
        HirType::Weak(inner) => format!("weak {}", hir_type_display(inner)),
        HirType::FatPtr { name, kind } => format!("{} {}", hir_type_display(kind), name.as_str()),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => format!("[{}]", hir_type_display(inner)),
        HirType::Ref(inner, mutable) => {
            if *mutable {
                format!("ref mut {}", hir_type_display(inner))
            } else {
                format!("ref {}", hir_type_display(inner))
            }
        }
    }
}

/// Check if a type needs deep copy (heap-allocated data).
pub(crate) fn needs_deep_copy(ty: &HirType) -> bool {
    matches!(ty, HirType::Named(_) | HirType::Array(_) | HirType::FatPtr { .. })
}


pub(crate) fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => *inner,
        other => other,
    }
}

    /// 剥去所有权包装的引用版本（不消耗所有权）
pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) | HirType::Ref(inner, _) => inner.as_ref(),
        other => other,
    }
}

    /// 推断 HIR 表达式的类型
pub(crate) fn expr_type(expr: &HirNodeBox) -> HirType {
    expr.expr_type()
}

/// Check if a HirExpr is a null literal (lowered to Int(0)).
pub(crate) fn is_null_literal(expr: &HirNodeBox) -> bool {
    matches!(expr.as_const(), Some(HirLiteral::Int(0)))
}

/// Check if a type is a pointer-like type for null comparison purposes.
pub(crate) fn is_pointer_type_for_cmp(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::Shared(_) | HirType::Unique(_)
        | HirType::Weak(_) | HirType::FatPtr { .. } | HirType::Array(_)
    )
}
