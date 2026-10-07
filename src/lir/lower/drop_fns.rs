//! 按需合成 Named 类型 drop 函数（递归类型终止；弱链接跨模块去重）。
use super::*;

/// 为实际出现 drop 站点的 Named 类型合成 `__drop_<n>`，返回新函数 id（弱链接）。
pub(super) fn synthesize_named_drop_fns(
    functions: &mut Vec<LirFn>,
    fn_names: &mut HashMap<FnId, String>,
    struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>,
    next_fn_id: &mut usize,
) -> Vec<FnId> {
    let mut named_drop_ids: Vec<FnId> = Vec::new();
    let mut needed: std::collections::HashSet<Symbol> = std::collections::HashSet::new();
    {
        let collect = |ty: &HirType, needed: &mut std::collections::HashSet<Symbol>| {
            collect_named_drop_types(ty, struct_defs, needed);
        };
        for f in functions.iter() {
            for b in &f.blocks {
                for inst in &b.insts {
                    let any = inst.as_any();
                    if let Some(d) = any.downcast_ref::<SLirDropValue>() { collect(&d.ty, &mut needed); }
                    if let Some(d) = any.downcast_ref::<SLirDropPtr>() { collect(&d.ty, &mut needed); }
                    if let Some(d) = any.downcast_ref::<SLirDropArray>() { collect(&d.elem_ty, &mut needed); }
                }
            }
        }
        let mut work: Vec<Symbol> = needed.iter().copied().collect();
        while let Some(n) = work.pop() {
            let Some(fields) = struct_defs.get(&n).cloned() else { continue; };
            for (_, ft) in &fields {
                let before = needed.len();
                collect_named_drop_types(ft, struct_defs, &mut needed);
                if needed.len() > before {
                    let new_items: Vec<Symbol> = needed.iter().copied()
                        .filter(|x| !work.contains(x)).collect();
                    work.extend(new_items);
                }
            }
        }
    }
    for name in &needed {
        let ty = HirType::Named(*name);
        let fid = FnId(*next_fn_id);
        *next_fn_id += 1;
        let fname = format!("__drop_{}", crate::lir::ir::sanitize_name(&name.as_str()));
        fn_names.insert(fid, fname.clone());
        functions.push(LirFn {
            fn_id: fid,
            name: Symbol::intern(&fname),
            is_inline: false,
            extern_c: false,
            is_pub: false,
            params: vec![(Symbol::intern("data"), HirType::Ref(Box::new(ty.clone()), true))],
            return_type: HirType::Void,
            locals: Vec::new(),
            attrs: Vec::new(),
            param_attrs: Vec::new(),
            effects: Default::default(),
            blocks: vec![LirBlock {
                label: "entry".to_string(),
                insts: vec![
                    SLirDropPtr { ptr: LirValue::Param(0), ty: ty.clone() }.into(),
                    SLirRet { val: None }.into(),
                ],
            }],
            custom: Vec::new(),
        });
        named_drop_ids.push(fid);
    }
    named_drop_ids
}

/// 收集类型中的 Named 类型（用于 drop 函数按需合成；穿透 unique/数组/ref/闭包）。
fn collect_named_drop_types(
    ty: &HirType,
    defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>,
    out: &mut std::collections::HashSet<Symbol>,
) {
    match ty {
        HirType::Named(n) => {
            if crate::lir::ir::needs_drop(ty, defs) {
                out.insert(*n);
            }
        }
        HirType::Unique(inner) | HirType::Array(inner) | HirType::ArraySized(inner, _)
        | HirType::Ref(inner, _) => collect_named_drop_types(inner, defs, out),
        HirType::Closure(ps, ret, _, _) => {
            for p in ps { collect_named_drop_types(p, defs, out); }
            collect_named_drop_types(ret, defs, out);
        }
        HirType::FatPtr { .. } => {}
        _ => {}
    }
}
