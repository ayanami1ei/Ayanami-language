use crate::intern::Symbol;
use std::collections::{HashMap, HashSet};

use crate::hir::ir::*;
use crate::mir::ir::*;
use crate::mir::mem::*;


mod assign;
mod assign_field;
mod checks;
mod control;
mod ctx;
mod functions;
mod mem;
mod temps;

use functions::lower_item;

pub fn lower_program(hir: &HirProgram) -> crate::error::Result<MirProgram> {
    let struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>> = hir.struct_defs.iter().map(|(name, fields)| {
        (*name, fields.iter().map(|f| (f.name, f.ty.clone())).collect())
    }).collect();
    let mut items = Vec::new();
    for item in &hir.items {
        items.extend(lower_item(item, &struct_defs)?);
    }
    Ok(MirProgram {
        items,
        vtables: hir.vtables.clone(),
        struct_defs: struct_defs.clone(),
        generic_struct_params: hir.generic_struct_params.clone(),
        statics: hir.statics.clone(),
        imported_fns: hir.imported_fns.iter().map(|f| crate::hir::ir::ImportedFnSig {
            fn_id: f.fn_id,
            name: f.name,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            attrs: f.attrs.clone(),
            effects: f.effects.clone(),
            inferred: f.inferred.clone(),
            extern_c: f.extern_c,
        }).collect(),
    })
}

struct Ctx {
    mir_locals: Vec<MirLocal>,
    var_types: HashMap<VarId, HirType>,
    alive: HashSet<VarId>,
    moved: HashSet<VarId>,
    /// match/if 表达式结果变量：条件赋值，覆盖/块末不 drop
    result_vars: HashSet<VarId>,
    struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    errors: Vec<crate::error::Error>,
    return_type: HirType,
}
