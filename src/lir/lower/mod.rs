use std::collections::HashMap;

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::*;

use super::ir::*;


mod ctx;
mod fn_lower;
mod mir_expr;
mod mir_expr2;
mod mir_stmts;
mod names;
mod strings;
mod util;

use ctx::LowerCtx;
use fn_lower::{lower_items, lower_stmts};
use names::{collect_fn_names, mangle};
use strings::collect_strings;

pub fn lower_program(mir: &MirProgram) -> LirProgram {
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

    let functions: Vec<LirFn> = mir
        .items
        .iter()
        .flat_map(|item| lower_items(item, &str_map))
        .filter(|f| !(f.extern_c && f.blocks.is_empty()))
        .collect();

    let vtables: Vec<VtableDesc> = mir.vtables.iter().map(|ve| {
        let name = format!("vtable_{}_{}",
            ve.concrete_type.as_str().replace('<', "_lt_").replace('>', "_gt_").replace('[', "_lb_").replace(']', "_rb_"),
            ve.interface.as_str().replace('<', "_lt_").replace('>', "_gt_"));
        VtableDesc { name, fn_ids: ve.method_fn_ids.clone() }
    }).collect();

    let defined_ids: std::collections::HashSet<FnId> = functions.iter().map(|f| f.fn_id).collect();
    let imported_fn_ids: std::collections::HashSet<FnId> = fn_names.keys()
        .filter(|id| !defined_ids.contains(id))
        .copied()
        .collect();

    LirProgram {
        strings,
        fn_names,
        functions,
        vtables,
        struct_defs: mir.struct_defs.clone(),
        generic_struct_params: mir.generic_struct_params.clone(),
        imported_fn_ids,
    }
}
