use super::*;
use super::mem::{action_to_stmt, strategy_for};
use crate::error::Result;

pub(super) fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<Vec<MirItem>> {
    match item {
        HirItem::Fn(f) => Ok(vec![MirItem::Fn(lower_fn(f, struct_defs)?)]),
        HirItem::StructDef(def) => Ok(vec![MirItem::StructDef {
            name: def.name,
            fields: def.fields.iter().map(|f| (f.name, f.ty.clone())).collect(),
        }]),
        HirItem::Namespace { name, items } => {
            let mut inner = Vec::new();
            for item in items {
                inner.extend(lower_item(item, struct_defs)?);
            }
            Ok(vec![MirItem::Namespace { name: *name, items: inner }])
        }
        HirItem::InterfaceDef { .. } => Ok(vec![]),
    }
}

fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Result<MirFn> {
    if f.extern_c {
        return Ok(MirFn {
            fn_id: f.fn_id,
            name: f.name,
            is_inline: f.is_inline,
            extern_c: f.extern_c,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            locals: vec![],
            body: vec![],
            attrs: f.attrs.clone(),
            param_attrs: f.param_attrs.clone(),
        });
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

    if let Some(e) = ctx.errors.into_iter().next() {
        return Err(e);
    }

    Ok(MirFn {
        fn_id: f.fn_id,
        name: f.name,
        is_inline: f.is_inline,
        extern_c: f.extern_c,
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        locals: ctx.mir_locals,
        body,
        attrs: f.attrs.clone(),
            param_attrs: f.param_attrs.clone(),
    })
}
