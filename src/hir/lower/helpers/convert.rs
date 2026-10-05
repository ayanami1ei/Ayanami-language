use super::*;

pub(crate) fn hir_type_to_ast_type(ty: &HirType) -> Type {
    let s = Span::default();
    match ty {
        HirType::Int => Type::Int(s),
        HirType::Float => Type::Float(s),
        HirType::F32 => Type::Named(Symbol::intern("f32"), s),
        HirType::Char => Type::Char(s),
        HirType::Bool => Type::Bool(s),
        HirType::Void => Type::Void(s),
        HirType::IntN { bits, signed } => Type::Named(Symbol::intern(&crate::hir::lower::helpers::intn_name(*bits, *signed)), s),
        HirType::Named(n) => Type::Named(*n, s),
        HirType::Unique(inner) => Type::Unique(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FnPtr(..) => Type::Int(s),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => Type::Array(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FatPtr { name, kind } => {
            let inner = Type::Named(*name, s);
            match kind.as_ref() {
                HirType::Unique(_) => Type::Unique(Box::new(inner), s),
                _ => inner,
            }
        }
        HirType::Ref(inner, mutable) => Type::Ref(Box::new(hir_type_to_ast_type(inner)), *mutable, s),
    }
}

/// Given an AST param type and the corresponding HirType from the lowered arg,
/// extract bindings for **all** generic parameter names used by the param type.
pub(crate) fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Vec<(Symbol, HirType)> {
    match (param_ty, arg_ty) {
        (Type::Named(name, _), _) => vec![(*name, arg_ty.clone())],
        // Generic("Pair", [A, B]) vs Named("Pair<int,int>") → A = int, B = int
        (Type::Generic(name, params, _), _) => {
            let mut out = Vec::new();
            let base = name.as_str();
            // 剥离所有所有权包装，获取底层的 Named 类型名
            let stripped = strip_ownership_ref(arg_ty);
            let arg_name = match stripped {
                HirType::Named(n) => n.as_str(),
                _ => return out,
            };
            // 如果 arg 本身就是泛型参数（如 Named("T")），直接映射
            for gp in params.iter() {
                if let Type::Named(gp_name, _) = gp {
                    if arg_name == gp_name.as_str() {
                        out.push((*gp_name, arg_ty.clone()));
                        return out;
                    }
                }
            }
            // Check if arg_name is "Name<...>"（兼容 `Name[...]` 旧编码）
            if let Some(start) = arg_name.find('<').or_else(|| arg_name.find('[')) {
                if &arg_name[..start] == base {
                    let closer = if arg_name.as_bytes()[start] == b'<' { '>' } else { ']' };
                    let inner = arg_name[start + 1..].trim_end_matches(closer);
                    let inner_parts = split_generic_args(inner);
                    // Decode each inner part: "int" → Int, "String" → Named("String")
                    for (gp, inner_str) in params.iter().zip(inner_parts.iter()) {
                        if let Type::Named(gp_name, _) = gp {
                            out.push((*gp_name, sig_str_to_hir(inner_str.trim())));
                        }
                    }
                }
            }
            out
        }
        (Type::Unique(inner, _), HirType::Unique(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        // Param expects wrapper but arg is unwrapped (auto-wrap will handle)
        (Type::Unique(inner, _), _) => {
            infer_generic_from_param(inner, arg_ty)
        }
        // 借用形参（如 ref Box[T]）→ 继续推导内层
        (Type::Ref(inner, _, _), _) => infer_generic_from_param(inner, arg_ty),
        // 数组形参（[T]）→ 从拥有/借用数组的元素类型推导
        (Type::Array(inner, _), _) => match strip_ownership_ref(arg_ty) {
            HirType::Array(a) | HirType::ArraySized(a, _) => infer_generic_from_param(inner, a),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// 按顶层逗号切分泛型实参（忽略嵌套 `<>` / `[]` 内的逗号）
fn split_generic_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '[' => depth += 1,
            '>' | ']' => depth -= 1,
            ',' if depth == 0 => { out.push(&s[start..i]); start = i + 1; }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

pub(crate) fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType {
    match ty {
        HirType::Named(s) => {
            // 直接匹配泛型参数名（如 T → int）
            if let Some(replacement) = subst.get(s) {
                return replacement.clone();
            }
            // 处理编码的泛型类型名（如 "LinkedListNode<T>" 或 "LinkedListNode[T]"）
            let name_str = s.as_str();
            let open = name_str.find('<').or_else(|| name_str.find('['));
            if let Some(start) = open {
                let base = &name_str[..start];
                let inner = name_str[start..].trim_start_matches('<').trim_start_matches('[').trim_end_matches('>').trim_end_matches(']');
                let parts: Vec<&str> = inner.split(',').collect();
                let mut changed = false;
                let new_parts: Vec<String> = parts.iter().map(|p| {
                    let trimmed = p.trim();
                    let sym = Symbol::intern(trimmed);
                    if let Some(replacement) = subst.get(&sym) {
                        changed = true;
                        hir_type_display(replacement)
                    } else {
                        trimmed.to_string()
                    }
                }).collect();
                if changed {
                    let new_name = format!("{}<{}>", base, new_parts.join(","));
                    return HirType::Named(Symbol::intern(&new_name));
                }
            }
            ty.clone()
        }
        HirType::Unique(inner) => HirType::Unique(Box::new(substitute_hir_type(inner, subst))),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => HirType::Array(Box::new(substitute_hir_type(inner, subst))),
        HirType::Ref(inner, mutable) => HirType::Ref(Box::new(substitute_hir_type(inner, subst)), *mutable),
        HirType::FatPtr { name, kind } => HirType::FatPtr {
            name: *name,
            kind: Box::new(substitute_hir_type(kind, subst)),
        },
        _ => ty.clone(),
    }
}
