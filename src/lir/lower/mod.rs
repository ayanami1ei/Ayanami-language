use std::collections::HashMap;

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::*;

use super::ir::*;


mod ctx;
mod fn_lower;
mod mir_bool;
mod mir_expr;
mod mir_expr2;
mod mir_contract;
mod mir_stmts;
mod mir_stmts_loops;
mod mir_ref;
pub(crate) mod names;
mod strings;
mod util;

/// A3c：递归收集定义函数的效应摘要。
fn collect_effect_summaries(items: &[MirItem], out: &mut HashMap<String, crate::hir::effects::EffectSummary>) {
    for item in items {
        match item {
            MirItem::Fn(f) => {
                out.insert(f.name.as_str(), crate::hir::effects::EffectSummary {
                    declared: f.effects.clone(),
                    inferred: f.inferred.clone(),
                });
            }
            MirItem::Namespace { items, .. } => collect_effect_summaries(items, out),
            _ => {}
        }
    }
}

/// 收集泛型特化函数 id（含 namespace 递归）
fn collect_specialized_fns(items: &[MirItem], out: &mut std::collections::HashSet<FnId>) {
    for item in items {
        match item {
            MirItem::Fn(f) => { if f.is_specialized { out.insert(f.fn_id); } }
            MirItem::Namespace { items, .. } => collect_specialized_fns(items, out),
            _ => {}
        }
    }
}

use ctx::LowerCtx;
use fn_lower::lower_items;
use names::{collect_fn_names, mangle};
use strings::collect_strings;

pub fn lower_program(mir: &MirProgram) -> LirProgram {
    let globals: Vec<LirGlobal> = mir.statics.iter()
        .filter(|s| !s.is_external)
        .map(|s| LirGlobal { name: s.name, ty: s.ty.clone(), value: s.value.clone(), is_pub: s.is_pub })
        .collect();
    let mut extern_globals: Vec<(Symbol, HirType)> = mir.statics.iter()
        .filter(|s| s.is_external)
        .map(|s| (s.name, s.ty.clone()))
        .collect();
    extern_globals.sort_by(|a, b| a.0.as_str().cmp(&b.0.as_str()));
    extern_globals.dedup_by(|a, b| a.0 == b.0);
    let strings = collect_strings(mir);
    let str_map: HashMap<String, u64> = strings
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u64))
        .collect();
    let mut fn_names = collect_fn_names(mir);

    for imp in &mir.imported_fns {
        if !fn_names.contains_key(&imp.fn_id) {
            let name = mangle("", &imp.name.as_str(), &imp.params);
            fn_names.insert(imp.fn_id, name);
        }
    }

    let all_fns: Vec<LirFn> = mir
        .items
        .iter()
        .flat_map(|item| lower_items(item, &str_map))
        .collect();

    let functions: Vec<LirFn> = all_fns
        .iter()
        .filter(|f| !(f.extern_c && f.blocks.is_empty()))
        .cloned()
        .collect();
    let defined_ids: std::collections::HashSet<FnId> = functions.iter().map(|f| f.fn_id).collect();

    // 外部声明：源码 extern "C" 声明（带标注）+ 未定义的包导入函数（真实签名）
    let mut extern_decls: Vec<ExternDecl> = Vec::new();
    for f in &all_fns {
        if f.extern_c && f.blocks.is_empty() {
            if let Some(name) = fn_names.get(&f.fn_id) {
                extern_decls.push(ExternDecl {
                    name: name.clone(),
                    params: f.params.iter().map(|(_, t)| t.clone()).collect(),
                    return_type: f.return_type.clone(),
                    attrs: f.attrs.clone(),
                    param_attrs: f.param_attrs.clone(),
                    effects: f.effects,
                    extern_c: true,
                });
            }
        }
    }
    for imp in &mir.imported_fns {
        if defined_ids.contains(&imp.fn_id) { continue; }
        let name = fn_names.get(&imp.fn_id).cloned().unwrap_or_else(|| mangle("", &imp.name.as_str(), &imp.params));
        if extern_decls.iter().any(|d| d.name == name) { continue; }
        extern_decls.push(ExternDecl {
            name,
            params: imp.params.iter().map(|(_, t)| t.clone()).collect(),
            return_type: imp.return_type.clone(),
            attrs: util::attrs_to_lir(&imp.attrs),
            param_attrs: Vec::new(), // 包导入暂不携带形参标注
            effects: util::lir_effects(&imp.effects, &Default::default(), false), // 导入：仅信任承诺
            extern_c: false,
        });
    }

    let mut vtables: Vec<VtableDesc> = mir.vtables.iter().map(|ve| {
        let name = format!("vtable_{}_{}",
            crate::lir::ir::sanitize_name(&ve.concrete_type.as_str()),
            crate::lir::ir::sanitize_name(&ve.interface.as_str()));
        VtableDesc { name, fn_ids: ve.method_fn_ids.clone() }
    }).collect();

    // 接口箱（拥有胖指针）递归 drop：为需要释放的具体类型合成 vtable[0] drop 函数，
    // 箱释放前调用（否则箱内拥有字段泄漏）。
    let mut functions = functions;
    let mut fn_names = fn_names;
    let mut drop_fns: HashMap<Symbol, FnId> = HashMap::new();
    let mut iface_drop_ids: Vec<FnId> = Vec::new();
    // 新 fn_id 必须避开所有已登记 id（含 extern/导入函数的 fn_names）
    let mut next_fn_id = fn_names.keys().map(|f| f.0)
        .chain(functions.iter().map(|f| f.fn_id.0))
        .max().unwrap_or(0) + 1;
    for (i, ve) in mir.vtables.iter().enumerate() {
        if ve.method_fn_ids.first() != Some(&FnId(usize::MAX)) {
            continue;
        }
        let concrete_ty = HirType::Named(ve.concrete_type);
        if !crate::lir::ir::needs_drop(&concrete_ty, &mir.struct_defs) {
            continue;
        }
        let fid = match drop_fns.get(&ve.concrete_type) {
            Some(&id) => id,
            None => {
                let id = FnId(next_fn_id);
                next_fn_id += 1;
                drop_fns.insert(ve.concrete_type, id);
                iface_drop_ids.push(id);
                let name = format!("__iface_drop_{}", crate::lir::ir::sanitize_name(&ve.concrete_type.as_str()));
                fn_names.insert(id, name.clone());
                functions.push(LirFn {
                    fn_id: id,
                    name: Symbol::intern(&name),
                    is_inline: false,
                    extern_c: false,
                    is_pub: false,
                    params: vec![(Symbol::intern("data"), HirType::Ref(Box::new(concrete_ty.clone()), true))],
                    return_type: HirType::Void,
                    locals: Vec::new(),
                    attrs: Vec::new(),
                    param_attrs: Vec::new(),
                    effects: Default::default(),
                    blocks: vec![LirBlock {
                        label: "entry".to_string(),
                        insts: vec![
                            SLirDropPtr { ptr: LirValue::Param(0), ty: concrete_ty.clone() }.into(),
                            SLirRet { val: None }.into(),
                        ],
                    }],
                    custom: Vec::new(),
                });
                id
            }
        };
        if let Some(vt) = vtables.get_mut(i) {
            if !vt.fn_ids.is_empty() {
                vt.fn_ids[0] = fid;
            }
        }
    }

    // A3c：收集本节函数的效应摘要（打包导出用；不参与序列化）
    let mut effect_summaries: HashMap<String, crate::hir::effects::EffectSummary> = HashMap::new();
    collect_effect_summaries(&mir.items, &mut effect_summaries);
    // 泛型特化集合（发射 linkonce_odr）
    let mut specialized_fns: std::collections::HashSet<FnId> = std::collections::HashSet::new();
    collect_specialized_fns(&mir.items, &mut specialized_fns);
    // 接口箱 drop 函数弱链接（包与使用方都会合成，跨模块去重）
    specialized_fns.extend(iface_drop_ids.iter().copied());

    LirProgram {
        strings,
        globals,
        extern_globals,
        fn_names,
        functions,
        vtables,
        struct_defs: mir.struct_defs.clone(),
        generic_struct_params: mir.generic_struct_params.clone(),
        extern_decls,
        effect_summaries,
        specialized_fns,
    }
}
