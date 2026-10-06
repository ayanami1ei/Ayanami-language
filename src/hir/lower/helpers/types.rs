use super::*;

pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String {
    match ty {
        Type::Default => "?".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Never(_) => "!".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Generic(name, args, _) => {
            let a: Vec<String> = args.iter().map(|a| type_to_string_generic(a, interfaces)).collect();
            format!("{}<{}>", name, a.join(","))
        }
        Type::Array(inner, _) => format!("[{}]", type_to_string_generic(inner, interfaces)),
        Type::ArraySized(inner, n, _) => format!("[{}; {}]", type_to_string_generic(inner, interfaces), n),
        Type::Ref(inner, mutable, _) => format!("ref{}{}",
            if *mutable { " mut" } else { "" },
            type_to_string_generic(inner, interfaces)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string_generic(inner, interfaces)),
        Type::FnPtr(..) => "fn(...)".to_string(),
        Type::Self_(_) => "Self".into(),
    }
}


/// 定宽类型名 → HirType（M1.10：`i64`/`isize`/`int` 归一为 `Int`；`f64`/`float` 归一为 `Float`）
pub(crate) fn fixed_width_type(name: &str) -> Option<HirType> {
    Some(match name {
        "int" | "i64" | "isize" => HirType::Int,
        "float" | "f64" => HirType::Float,
        "f32" => HirType::F32,
        "i8" => HirType::IntN { bits: 8, signed: true },
        "i16" => HirType::IntN { bits: 16, signed: true },
        "i32" => HirType::IntN { bits: 32, signed: true },
        "i128" => HirType::IntN { bits: 128, signed: true },
        "u8" => HirType::IntN { bits: 8, signed: false },
        "u16" => HirType::IntN { bits: 16, signed: false },
        "u32" => HirType::IntN { bits: 32, signed: false },
        "u64" | "usize" => HirType::IntN { bits: 64, signed: false },
        "u128" => HirType::IntN { bits: 128, signed: false },
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
        // M6.2c：定长数组 `[T; N]`
        if let Some((elem, n)) = inner.split_once(';') {
            if let Ok(n) = n.trim().parse::<usize>() {
                return HirType::ArraySized(Box::new(sig_str_to_hir(elem)), n);
            }
        }
        HirType::Array(Box::new(sig_str_to_hir(inner)))
    } else {
        match s {
            "char" => HirType::Char,
            "bool" => HirType::Bool,
            "void" => HirType::Void,
            "!" => HirType::Never,
            other => {
                // #104/#144：.lcl 签名可能写 `Option[int]`（源语法）；归一化为 `Option<int>`（HIR 泛型命名）。
                // 嵌套泛型 `Option[ArrayList[String]]` 需要递归转换（此前只换外层 → 基名回退 → ABI 不一致）。
                let canonical = normalize_generic_brackets(other);
                match fixed_width_type(&canonical) {
                    Some(t) => t,
                    None => HirType::Named(Symbol::intern(&canonical)),
                }
            }
        }
    }
}

/// #144：签名类型文本 `Name[...]` → `Name<...>`（递归，尊重嵌套；`[T]`/`[T;N]` 数组保持方括号）
pub(crate) fn normalize_generic_brackets(s: &str) -> String {
    let s = s.trim();
    // 数组：以 `[` 开头 → 保持方括号（元素递归归一）
    if let Some(rest) = s.strip_prefix('[') {
        if let Some(inner) = rest.strip_suffix(']') {
            if let Some((elem, n)) = inner.split_once(';') {
                return format!("[{};{}]", normalize_generic_brackets(elem), n.trim());
            }
            return format!("[{}]", normalize_generic_brackets(inner));
        }
        return s.to_string();
    }
    // 泛型：`Name[args]` / `Name<args>` → `Name<normalized args>`
    if let Some(open) = s.find(['[', '<']) {
        let base = &s[..open];
        if let Some(inner) = generic_inner(s) {
            let parts: Vec<String> = split_generic_args(inner).iter()
                .map(|p| normalize_generic_brackets(p))
                .collect();
            return format!("{}<{}>", base, parts.join(","));
        }
    }
    s.to_string()
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
        Type::Never(_) => HirType::Never,
        // `[T]` 即拥有堆数组（unique 已移除；借用写 `ref [T]`）
        Type::Array(inner, _) => HirType::Unique(Box::new(HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))))),
        Type::ArraySized(inner, n, _) => HirType::Unique(Box::new(HirType::ArraySized(
            Box::new(ast_type_to_hir(inner, interfaces)), *n))),
        Type::Generic(name, args, _) => {
            // Encode generic instantiation as a unique named type
            let args_str: Vec<String> = args.iter()
                .map(|a| type_to_string_generic(a, interfaces))
                .collect();
            HirType::Named(Symbol::intern(&format!("{}<{}>", name, args_str.join(","))))
        }
        Type::Named(s, _) => {
            let name = s.as_str();
            if name == "char" { HirType::Char }
            else if name == "void" { HirType::Void }
            else if name == "bool" { HirType::Bool }
            else if let Some(t) = fixed_width_type(&name) { t }
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
            // 接口签名中的 `Self`（实现类型哨兵）；impl 方法签名已由解析器替换为具体类型。
            // 匹配/替换见 hir/lower/body/iface_match.rs（T1）。
            HirType::Named(Symbol::intern("Self"))
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
        HirType::Never => "!".into(),
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
