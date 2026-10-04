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
        Type::FnPtr(..) => "fn(...)".to_string(),
        Type::Self_(_) => "Self".into(),
    }
}


/// 定宽整数类型名 → (bits, signed)
pub(crate) fn fixed_width_int(name: &str) -> Option<(u8, bool)> {
    Some(match name {
        "i8" => (8, true), "i16" => (16, true), "i32" => (32, true), "i64" => (64, true), "i128" => (128, true),
        "u8" => (8, false), "u16" => (16, false), "u32" => (32, false), "u64" => (64, false), "u128" => (128, false),
        "isize" => (64, true), "usize" => (64, false),
        _ => return None,
    })
}

/// 定宽整数的可读名（i8/u32/...）
pub(crate) fn intn_name(bits: u8, signed: bool) -> String {
    format!("{}{}", if signed { "i" } else { "u" }, bits)
}

pub(crate) fn sig_str_to_hir(s: &str) -> HirType {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("ref mut ") {
        HirType::Ref(Box::new(sig_str_to_hir(inner)), true)
    } else if let Some(inner) = s.strip_prefix("ref ") {
        HirType::Ref(Box::new(sig_str_to_hir(inner)), false)
    } else if let Some(inner) = s.strip_prefix("unique ") {
        HirType::Unique(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        HirType::Array(Box::new(sig_str_to_hir(inner)))
    } else {
        match s {
            "int" => HirType::Int,
            "float" => HirType::Float,
            "f32" => HirType::F32,
            "f64" => HirType::Float,
            "char" => HirType::Char,
            "bool" => HirType::Bool,
            "void" => HirType::Void,
            other => match fixed_width_int(other) {
                Some((bits, signed)) => HirType::IntN { bits, signed },
                None => HirType::Named(Symbol::intern(other)),
            },
        }
    }
}

    /// 将 AST 类型节点转换为 HIR 类型（含接口信息）
fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool {
    match inner_hir {
        HirType::Named(n) | HirType::FatPtr { name: n, .. } if interfaces.contains_key(n) => true,
        HirType::Named(n) | HirType::FatPtr { name: n, .. } => {
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
        // `[T]` 即拥有堆数组（unique 已移除；借用写 `ref [T]`）
        Type::Array(inner, _) => HirType::Unique(Box::new(HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))))),
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
            else if name == "f32" { HirType::F32 }
            else if name == "f64" { HirType::Float }
            else if name == "char" { HirType::Char }
            else if name == "void" { HirType::Void }
            else if name == "bool" { HirType::Bool }
            else if let Some((bits, signed)) = fixed_width_int(&name) { HirType::IntN { bits, signed } }
            else if interfaces.contains_key(s) {
                // 裸接口类型：拥有所有权的胖指针（Box<dyn Trait>）
                HirType::FatPtr { name: *s, kind: Box::new(HirType::Unique(Box::new(HirType::Void))) }
            }
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
        Type::Ref(inner, mutable, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if is_iface_type(&inner_hir, interfaces) {
                HirType::FatPtr {
                    name: *extract_named(&inner_hir).unwrap(),
                    kind: Box::new(HirType::Ref(Box::new(HirType::Void), *mutable)),
                }
            } else {
                HirType::Ref(Box::new(inner_hir), *mutable)
            }
        }
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
        HirType::FatPtr { name, .. } => Some(name),
        _ => None,
    }
}

    /// 将 HirType 格式化为可读字符串（用于错误消息和调试输出）
pub(crate) fn hir_type_display(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::F32 => "f32".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::IntN { bits, signed } => intn_name(*bits, *signed),
        HirType::Named(s) => s.as_str().to_string(),
        HirType::Unique(inner) => format!("unique {}", hir_type_display(inner)),
        HirType::FnPtr(..) => "fn(...)".into(),
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


pub(crate) fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Unique(inner) => *inner,
        other => other,
    }
}

    /// 剥去所有权包装的引用版本（不消耗所有权）
pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType {
    match ty {
        HirType::Unique(inner) | HirType::Ref(inner, _) => inner.as_ref(),
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
        HirType::Named(_) | HirType::Unique(_)
        | HirType::FatPtr { .. } | HirType::Array(_)
    )
}
