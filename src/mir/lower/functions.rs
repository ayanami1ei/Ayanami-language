use super::*;
use super::mem::{action_to_stmt, strategy_for};

pub(super) fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Vec<MirItem> {
    match item {
        HirItem::Fn(f) => vec![MirItem::Fn(lower_fn(f, struct_defs))],
        HirItem::StructDef(def) => vec![MirItem::StructDef {
            name: def.name,
            fields: def.fields.iter().map(|f| (f.name, f.ty.clone())).collect(),
        }],
        HirItem::Namespace { name, items } => {
            let inner: Vec<MirItem> = items.iter().flat_map(|item| lower_item(item, struct_defs)).collect();
            vec![MirItem::Namespace { name: *name, items: inner }]
        }
        HirItem::InterfaceDef { .. } => vec![],
    }
}

fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> MirFn {
    if f.extern_c {
        return MirFn {
            fn_id: f.fn_id,
            name: f.name,
            is_inline: f.is_inline,
            extern_c: f.extern_c,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            locals: vec![],
            body: vec![],
        };
    }

    let mut ctx = Ctx::new(f, struct_defs);

    let mut body = Vec::new();
    for stmt in &f.body.stmts {
        let mut stmts = ctx.lower_stmt(stmt);
        body.append(&mut stmts);
    }

    let mut cleanup = Vec::new();
    let alive_snapshot: Vec<VarId> = ctx.alive.iter().copied().collect();
    for var in &alive_snapshot {
        if ctx.moved.contains(var) { continue; }
        let ty = ctx.var_types[var].clone();
        let strategy = strategy_for(&ty, struct_defs);
        for action in strategy.on_scope_end(*var, &ty) {
            cleanup.push(action_to_stmt(*var, &ty, &action));
        }
    }
    body.append(&mut cleanup);

    for var in &alive_snapshot {
        if ctx.moved.contains(var) { continue; }
        let ty = &ctx.var_types[var];
        let inner = match ty {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if let HirType::Named(type_name) = inner {
            if let Some(fields) = struct_defs.get(type_name) {
                for (_, field_ty) in fields {
                    match field_ty {
                        HirType::Unique(inner_field) => {}
                        HirType::Shared(inner_field) => {}
                        _ => {}
                    }
                }
            }
        }
    }

    MirFn {
        fn_id: f.fn_id,
        name: f.name,
        is_inline: f.is_inline,
        extern_c: f.extern_c,
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        locals: ctx.mir_locals,
        body,
    }
}
