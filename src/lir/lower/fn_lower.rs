use super::*;

/// MIR 标注 → LIR 标注
fn lir_attrs(f: &MirFn) -> Vec<LirAttr> {
    util::attrs_to_lir(&f.attrs)
}

/// MIR 形参标注 → LIR 形参标注（与 params 等长并行）
fn lir_param_attrs(f: &MirFn) -> Vec<Vec<LirAttr>> {
    f.param_attrs.iter().map(|v| util::attrs_to_lir(v)).collect()
}

pub(super) fn lower_items(item: &MirItem, str_map: &HashMap<String, u64>, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Vec<LirFn> {
    match item {
        MirItem::Fn(f) => vec![lower_fn(f, str_map, struct_defs)],
        MirItem::StructDef { .. } => vec![],
        MirItem::Namespace { items, .. } => {
            items.iter().flat_map(|child| lower_items(child, str_map, struct_defs)).collect()
        }
    }
}

pub(super) fn lower_fn(f: &MirFn, str_map: &HashMap<String, u64>, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> LirFn {
    let mut ctx = LowerCtx::new(str_map, struct_defs);

    if f.extern_c && f.body.is_empty() {
        return LirFn {
            fn_id: f.fn_id,
            name: f.name,
            is_inline: f.is_inline,
            extern_c: f.extern_c,
            is_pub: f.is_pub,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            locals: f.locals.clone(),
            attrs: lir_attrs(f),
        param_attrs: lir_param_attrs(f),
        effects: util::lir_effects(&f.effects, &f.inferred, !f.extern_c),
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
        is_pub: f.is_pub,
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        locals: f.locals.clone(),
        attrs: lir_attrs(f),
        param_attrs: lir_param_attrs(f),
        effects: util::lir_effects(&f.effects, &f.inferred, !f.extern_c),
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
        HirType::Void | HirType::Never => None,
        HirType::Int => Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), HirType::Int)),
        HirType::Float => Some((LirValue::Literal(HirLiteral::Float(0.0), HirType::Float), HirType::Float)),
        HirType::F32 => Some((LirValue::Literal(HirLiteral::Float(0.0), ty.clone()), ty.clone())),
        HirType::Char => Some((LirValue::Literal(HirLiteral::Char('\0'), HirType::Char), HirType::Char)),
        HirType::Bool => Some((LirValue::Literal(HirLiteral::Bool(false), HirType::Bool), HirType::Bool)),
        HirType::IntN { .. } => Some((LirValue::Literal(HirLiteral::Int(0), ty.clone()), ty.clone())),
        HirType::Named(_) | HirType::Unique(_) | HirType::FatPtr { .. } | HirType::Closure(..) | HirType::Array(_) | HirType::ArraySized(_, _) | HirType::Ref(_, _) | HirType::FnPtr(..) => {
            Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), ty.clone()))
        }
    }
}

pub(super) fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Unique(inner) => *inner,
        other => other,
    }
}

pub(super) fn type_size(ty: &HirType) -> u64 {
    match ty {
        HirType::Int | HirType::Float => 8,
        HirType::F32 => 4,
        HirType::IntN { bits, .. } => (*bits / 8) as u64,
        HirType::Char | HirType::Bool => 1,
        HirType::Void | HirType::Never => 0,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Closure(..) | HirType::Array(_) | HirType::ArraySized(_, _) => 16,
        HirType::Unique(inner) => type_size(inner),
        HirType::Ref(_, _) | HirType::FnPtr(..) => 8,
    }
}
