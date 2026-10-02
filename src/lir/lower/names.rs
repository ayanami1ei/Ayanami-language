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
                } else {
                    mangle("", &f.name.as_str().replace('.', "__"), &f.params)
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

pub(super) fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String {
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
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::Named(s) => s.as_str().replace('<', "_lt_").replace('>', "_gt_")
            .replace(',', "_c_").replace(' ', "_").replace('[', "_lb_").replace(']', "_rb_"),
        HirType::Unique(inner) => format!("unique_{}", type_to_mangle(inner)),
        HirType::FnPtr(..) => "fnptr".into(),
        HirType::FatPtr { name, .. } => format!("fatptr_{}", name.as_str().replace('<', "_lt_").replace('>', "_gt_")),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => format!("arr_{}", type_to_mangle(inner)),
        HirType::Ref(inner, _) => format!("ref_{}", type_to_mangle(inner)),
    }
}
