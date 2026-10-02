use crate::intern::Symbol;
use std::collections::{HashMap, HashSet};

use crate::hir::ir::*;
use crate::mir::ir::*;
use crate::mir::mem::*;


mod ctx;
mod functions;
mod mem;

use functions::lower_item;

pub fn lower_program(hir: &HirProgram) -> MirProgram {
    let struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>> = hir.struct_defs.iter().map(|(name, fields)| {
        (*name, fields.iter().map(|f| (f.name, f.ty.clone())).collect())
    }).collect();
    MirProgram {
        items: hir.items.iter().flat_map(|item| lower_item(item, &struct_defs)).collect(),
        vtables: hir.vtables.clone(),
        struct_defs: struct_defs.clone(),
        generic_struct_params: hir.generic_struct_params.clone(),
        imported_fns: hir.imported_fns.iter().map(|f| crate::hir::ir::ImportedFnSig {
            fn_id: f.fn_id,
            name: f.name,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
        }).collect(),
    }
}

struct Ctx {
    mir_locals: Vec<MirLocal>,
    var_types: HashMap<VarId, HirType>,
    alive: HashSet<VarId>,
    moved: HashSet<VarId>,
    struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
}
