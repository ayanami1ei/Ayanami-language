use super::*;

pub(super) fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String> {
    let mut map = HashMap::new();
    collect_fn_names_items(&mir.items, "", &mut map);
    map
}

pub(super) fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>) {
    for item in items {
        match item {
            MirItem::Fn(f) => {
                let name = if f.extern_c {
                    f.name.as_str().to_string()
                } else if f.is_macro {
                    // A5b：宏函数符号加保留前缀，避免与宏展开产物重名
                    format!("__ayanami_macro_{}", mangle("", &f.name.as_str().replace('.', "__"), &f.params))
                } else {
                    let base = mangle("", &f.name.as_str().replace('.', "__"), &f.params);
                    unique_fn_name(&base, f, map)
                };
                map.insert(f.fn_id, name);
            }
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { name: _name, items } => {
                collect_fn_names_items(items, "", map);
            }
        }
    }
}

/// 同名同参数、仅返回类型不同的特化（如零参构造函数 `ArrayList.new`）
/// 追加返回类型/ID 消歧，避免 LLVM 重复定义
fn unique_fn_name(base: &str, f: &MirFn, map: &HashMap<FnId, String>) -> String {
    let used = |n: &str| map.values().any(|v| v == n);
    if !used(base) {
        return base.to_string();
    }
    let with_ret = format!("{}_{}", base, type_to_mangle(&f.return_type));
    if !used(&with_ret) {
        return with_ret;
    }
    format!("{}_{}", with_ret, f.fn_id.0)
}

pub(crate) fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String {
    let safe_name = name.replace('.', "__");
    let safe_prefix = prefix.replace('.', "__");
    let base = if safe_prefix.is_empty() {
        safe_name
    } else {
        format!("{}__{}", safe_prefix, safe_name)
    };
    if params.is_empty() {
        base
    } else {
        let suffix: String = params.iter()
            .map(|(_, t)| format!("_{}", type_to_mangle(t)))
            .collect();
        format!("{}{}", base, suffix)
    }
}

pub(super) fn type_to_mangle(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::F32 => "f32".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::IntN { bits, signed } => crate::hir::lower::helpers::intn_name(*bits, *signed),
        HirType::Named(s) => s.as_str().replace('<', "_lt_").replace('>', "_gt_")
            .replace(',', "_c_").replace(' ', "_").replace('[', "_lb_").replace(']', "_rb_"),
        // `[T]` 在 HIR 中是 Unique(Array(T))，导入签名字符串里是 Array(T)：
        // 二者按同一名字 mangle，保证定义与调用链接一致
        HirType::Unique(inner) if matches!(&**inner, HirType::Array(_) | HirType::ArraySized(_, _)) => type_to_mangle(inner),
        HirType::Unique(inner) => format!("unique_{}", type_to_mangle(inner)),
        HirType::FnPtr(..) => "fnptr".into(),
        HirType::FatPtr { name, .. } => format!("fatptr_{}", name.as_str().replace('<', "_lt_").replace('>', "_gt_")),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => format!("arr_{}", type_to_mangle(inner)),
        HirType::Ref(inner, _) => format!("ref_{}", type_to_mangle(inner)),
    }
}
