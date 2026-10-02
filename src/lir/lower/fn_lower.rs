use super::*;

pub(super) fn lower_items(item: &MirItem, str_map: &HashMap<String, u64>) -> Vec<LirFn> {
    match item {
        MirItem::Fn(f) => vec![lower_fn(f, str_map)],
        MirItem::StructDef { .. } => vec![],
        MirItem::Namespace { items, .. } => {
            items.iter().flat_map(|child| lower_items(child, str_map)).collect()
        }
    }
}

pub(super) fn lower_fn(f: &MirFn, str_map: &HashMap<String, u64>) -> LirFn {
    let mut ctx = LowerCtx::new(str_map);

    if f.extern_c && f.body.is_empty() {
        return LirFn {
            fn_id: f.fn_id,
            name: f.name,
            is_inline: f.is_inline,
            extern_c: f.extern_c,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            locals: f.locals.clone(),
            blocks: vec![],
            custom: Vec::new(),
        };
    }

    for (i, local) in f.locals.iter().enumerate() {
        ctx.emit(SLirAlloca { var: VarId(i), ty: local.ty.clone() }.into());
    }

    for (i, (_, ty)) in f.params.iter().enumerate() {
        let vid = VarId(i);
        ctx.emit(SLirStore {
            dest: vid,
            src: LirValue::Param(i as u64),
            ty: ty.clone(),
        }.into());
    }

    lower_stmts(&mut ctx, &f.body);

    let needs_ret = ctx
        .current_insts
        .last()
        .map_or(true, |i| i.kind() != "Ret");
    if needs_ret {
        let ret = default_ret_value(&f.return_type);
        ctx.emit(SLirRet { val: ret }.into());
    }

    let blocks = ctx.finish();

    LirFn {
        fn_id: f.fn_id,
        name: f.name,
        is_inline: f.is_inline,
        extern_c: f.extern_c,
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        locals: f.locals.clone(),
        blocks,
        custom: Vec::new(),
    }
}

pub(super) fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmtBox]) {
    let mut i = 0;
    while i < stmts.len() {
        if stmts[i].is_return() {
            let ret_val = stmts[i].return_value().and_then(|v| {
                let val = lower_expr(ctx, v);
                let ty = v.expr_type();
                Some((val, ty))
            });
            i += 1;
            while i < stmts.len() {
                if let Some((id, ty)) = stmts[i].as_drop() {
                    ctx.emit(SLirDropValue { var: id, ty: ty.clone() }.into());
                    i += 1;
                } else if let Some((id, ty)) = stmts[i].as_retain() {
                    ctx.emit(SLirRetainValue { var: id, ty: ty.clone() }.into());
                    i += 1;
                } else if let Some((id, ty)) = stmts[i].as_release() {
                    ctx.emit(SLirReleaseValue { var: id, ty: ty.clone() }.into());
                    i += 1;
                } else {
                    break;
                }
            }
            ctx.emit(SLirRet { val: ret_val }.into());
            continue;
        }
        stmts[i].lower_to_lir_stmt(ctx);
        i += 1;
    }
}

pub(super) fn lower_expr(ctx: &mut dyn LirLowerCtx, expr: &MirNodeBox) -> LirValue {
    expr.lower_to_lir(ctx)
}

pub(super) fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)> {
    match ty {
        HirType::Void => None,
        HirType::Int => Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), HirType::Int)),
        HirType::Float => Some((LirValue::Literal(HirLiteral::Float(0.0), HirType::Float), HirType::Float)),
        HirType::Char => Some((LirValue::Literal(HirLiteral::Char('\0'), HirType::Char), HirType::Char)),
        HirType::Bool => Some((LirValue::Literal(HirLiteral::Bool(false), HirType::Bool), HirType::Bool)),
        HirType::Named(_) | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_) | HirType::FatPtr { .. } | HirType::Array(_) | HirType::ArraySized(_, _) | HirType::Ref(_, _) | HirType::FnPtr(..) => {
            Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), ty.clone()))
        }
    }
}

pub(super) fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => *inner,
        other => other,
    }
}

pub(super) fn type_size(ty: &HirType) -> u64 {
    match ty {
        HirType::Int | HirType::Float => 8,
        HirType::Char | HirType::Bool => 1,
        HirType::Void => 0,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_) | HirType::ArraySized(_, _) => 16,
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => type_size(inner),
        HirType::Ref(_, _) | HirType::FnPtr(..) => 8,
    }
}
