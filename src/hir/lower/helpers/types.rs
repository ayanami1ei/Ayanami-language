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
        Type::Closure(ps, ret, _, once) => format!("{}({})->{}",
            if *once { "FnOnce" } else { "Fn" },
            ps.iter().map(|p| type_to_string_generic(p, interfaces)).collect::<Vec<_>>().join(","),
            type_to_string_generic(ret, interfaces)),
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
        Type::Closure(ps, ret, _, once) => HirType::Closure(
            ps.iter().map(|p| ast_type_to_hir(p, interfaces)).collect(),
            Box::new(ast_type_to_hir(ret, interfaces)), true, *once),
        // `[T]` 即拥有堆数组（unique 已移除；借用写 `ref [T]`）
        Type::Array(inner, _) => HirType::Unique(Box::new(HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))))),
        Type::ArraySized(inner, n, _) => HirType::Unique(Box::new(HirType::ArraySized(Box::new(ast_type_to_hir(inner, interfaces)), *n))),
        Type::Generic(name, args, _) => {
            // M4：`Ptr[T]` 拥有指针（内部 Unique(T)）；接口内层 → 拥有胖指针
            if name.as_str() == "Ptr" && args.len() == 1 {
                let inner = ast_type_to_hir(&args[0], interfaces);
                if is_iface_type(&inner, interfaces) {
                    return HirType::FatPtr {
                        name: *extract_named(&inner).unwrap(),
                        kind: Box::new(HirType::Unique(Box::new(HirType::Void))),
                    };
                }
                return HirType::Unique(Box::new(inner));
            }
            // 泛型实例编码为命名类型；泛型接口实例 → 拥有所有权的胖指针（调用点按需特化）
            let args_str: Vec<String> = args.iter().map(|a| type_to_string_generic(a, interfaces)).collect();
            let sym = Symbol::intern(&format!("{}<{}>", name, args_str.join(",")));
            if interfaces.contains_key(name) {
                HirType::FatPtr { name: sym, kind: Box::new(HirType::Unique(Box::new(HirType::Void))) }
            } else { HirType::Named(sym) }
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
            } else { HirType::Unique(Box::new(inner_hir)) }
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
        Type::FnPtr(params, ret, _) => HirType::Closure(
            params.iter().map(|p| ast_type_to_hir(p, interfaces)).collect(),
            Box::new(ast_type_to_hir(ret, interfaces)),
            true,
            false,
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
        // `[T]`/`[T; n]` 即拥有数组（内部 Unique(Array)）；其余 Unique 为 `Ptr<T>`
        // （HIR 泛型名统一角括号风格，与 `type_to_string_generic` 一致）
        HirType::Unique(inner) => match inner.as_ref() {
            HirType::Array(e) => format!("[{}]", hir_type_display(e)),
            HirType::ArraySized(e, n) => format!("[{}; {}]", hir_type_display(e), n),
            other => format!("Ptr<{}>", hir_type_display(other)),
        },
        HirType::FnPtr(..) => "fn(...)".into(),
        HirType::Closure(ps, ret, _, once) => format!("{}({}) -> {}",
            if *once { "FnOnce" } else { "Fn" },
            ps.iter().map(hir_type_display).collect::<Vec<_>>().join(", "),
            hir_type_display(ret)),
        HirType::FatPtr { name, kind } => match kind.as_ref() {
            // 拥有接口（`Shape`）与借用接口（`ref Shape`）的用户语法
            HirType::Unique(inner) if matches!(inner.as_ref(), HirType::Void) => name.as_str().to_string(),
            _ => format!("{} {}", hir_type_display(kind), name.as_str()),
        },
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
