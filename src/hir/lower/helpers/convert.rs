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
        HirType::Never => Type::Never(s),
        HirType::IntN { bits, signed } => Type::Named(Symbol::intern(&crate::hir::lower::helpers::intn_name(*bits, *signed)), s),
        HirType::Named(n) => Type::Named(*n, s),
        HirType::Unique(inner) => Type::Unique(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FnPtr(..) => Type::Int(s),
        HirType::Closure(ps, ret, _, once) => Type::Closure(
            ps.iter().map(hir_type_to_ast_type).collect(),
            Box::new(hir_type_to_ast_type(ret)), s, *once),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => Type::Array(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FatPtr { name, kind } => {
            let inner = Type::Named(*name, s);
            match kind.as_ref() {
                HirType::Unique(_) => Type::Unique(Box::new(inner), s),
                _ => inner,
            }
        }
        HirType::Ref(inner, mutable) => Type::Ref(Box::new(hir_type_to_ast_type(inner)), *mutable, s),
        HirType::RawPtr(inner) => Type::RawPtr(Box::new(hir_type_to_ast_type(inner)), s),
    }
}

/// 给定 AST 形参类型与实参 HirType，收集全部泛型参数绑定。
/// P4c：AST 形参编码为 HirType 后交给统一 `unify`（helpers/unify.rs）——嵌套泛型匹配
/// 的唯一实现；wrapper（Unique/Ref/数组/闭包）穿透由 unify 承担。
pub(crate) fn infer_generic_from_param(
    param_ty: &Type,
    arg_ty: &HirType,
    interfaces: &HashMap<Symbol, InterfaceReg>,
) -> Vec<(Symbol, HirType)> {
    let mut subst = HashMap::new();
    let _ = unify(&ast_type_to_hir(param_ty, interfaces), arg_ty, &mut subst);
    subst.into_iter().collect()
}

/// 提取 `Base<...>` / `Base[...]` 的外层参数串（匹配首个开括号的配对闭括号，尊重嵌套）。
pub(crate) fn generic_inner(s: &str) -> Option<&str> {
    let open = s.find(['<', '['])?;
    let mut depth = 0i32;
    for (i, c) in s.char_indices().skip(open) {
        match c {
            '<' | '[' => depth += 1,
            '>' | ']' => {
                // `->` 的 `>` 不是括号闭合
                if c == '>' && i > 0 && s.as_bytes()[i - 1] == b'-' { continue; }
                depth -= 1;
                if depth == 0 { return Some(&s[open + 1..i]); }
            }
            _ => {}
        }
    }
    None
}

/// 按顶层逗号切分泛型实参（忽略嵌套 `<>` / `[]` / `()` 内的逗号）
pub(crate) fn split_generic_args(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '<' | '[' | '(' => depth += 1,
            '>' | ']' | ')' => {
                // `->` 的 `>` 不是括号闭合（闭包签名 Fn(T)->U 会出现在实参里）
                if c == '>' && i > 0 && s.as_bytes()[i - 1] == b'-' { continue; }
                depth -= 1;
            }
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
                if let Some(inner) = generic_inner(&name_str) {
                    let mut changed = false;
                    let new_parts: Vec<String> = split_generic_args(inner).iter().map(|p| {
                        let orig = sig_str_to_hir(p.trim());
                        let sub = substitute_hir_type(&orig, subst);
                        if sub != orig { changed = true; }
                        hir_type_display(&sub)
                    }).collect();
                    if changed {
                        let new_name = format!("{}<{}>", base, new_parts.join(","));
                        return HirType::Named(Symbol::intern(&new_name));
                    }
                }
            }
            ty.clone()
        }
        HirType::Unique(inner) => HirType::Unique(Box::new(substitute_hir_type(inner, subst))),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => HirType::Array(Box::new(substitute_hir_type(inner, subst))),
        HirType::Ref(inner, mutable) => HirType::Ref(Box::new(substitute_hir_type(inner, subst)), *mutable),
        // #164：闭包/裸函数指针签名内的泛型实参同样替换（字段闭包返回 U → String）
        HirType::Closure(params, ret, owns, once) => HirType::Closure(
            params.iter().map(|p| substitute_hir_type(p, subst)).collect(),
            Box::new(substitute_hir_type(ret, subst)),
            *owns, *once,
        ),
        HirType::FnPtr(params, ret) => HirType::FnPtr(
            params.iter().map(|p| substitute_hir_type(p, subst)).collect(),
            Box::new(substitute_hir_type(ret, subst)),
        ),
        HirType::FatPtr { name, kind } => {
            // #161：接口名里的泛型实参同样要替换（Iterator<T> + T=int → Iterator<int>）
            let new_name = match substitute_hir_type(&HirType::Named(*name), subst) {
                HirType::Named(n) => n,
                _ => *name,
            };
            HirType::FatPtr {
                name: new_name,
                kind: Box::new(substitute_hir_type(kind, subst)),
            }
        }
        _ => ty.clone(),
    }
}

