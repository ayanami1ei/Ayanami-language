//! 签名串类型解析（`.lcl`/泛型名回解析）：`sig_str_to_hir` 与括号归一化。
use super::*;

pub(crate) fn sig_str_to_hir(s: &str) -> HirType {
    let s = s.trim();
    // M2：闭包签名 `Fn(T1,T2)->R`（.lcl 导出格式；见 type_to_string_generic）
    if let Some((rest, once)) = s.strip_prefix("FnOnce(").map(|r| (r, true)).or_else(|| s.strip_prefix("Fn(").map(|r| (r, false))) {
        let mut depth = 1i32;
        let mut close = None;
        for (i, c) in rest.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 { close = Some(i); break; }
                }
                _ => {}
            }
        }
        if let Some(i) = close {
            let after = rest[i + 1..].trim_start();
            if let Some(ret_str) = after.strip_prefix("->") {
                let params_str = &rest[..i];
                let params = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    split_generic_args(params_str).iter().map(|p| sig_str_to_hir(p.trim())).collect()
                };
                return HirType::Closure(params, Box::new(sig_str_to_hir(ret_str.trim())), true, once);
            }
        }
    }
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
    } else if s.split(['[', '<']).next().map(|b| b.trim() == "Ptr").unwrap_or(false) {
        // M4：`Ptr[T]` 拥有指针（签名串/泛型实参回解析）
        match generic_inner(s) {
            Some(inner) => HirType::Unique(Box::new(sig_str_to_hir(inner))),
            None => HirType::Unique(Box::new(HirType::Void)),
        }
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
