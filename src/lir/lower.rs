use std::collections::HashMap;

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::*;

use super::ir::*;

pub fn lower_program(mir: &MirProgram) -> LirProgram {
    let strings = collect_strings(mir);
    let str_map: HashMap<String, u64> = strings
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u64))
        .collect();
    let mut fn_names = collect_fn_names(mir);

    // Add names for imported functions (not defined in any MirItem)
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
        let name = format!("vtable_{}_{}", ve.concrete_type, ve.interface);
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
        imported_fn_ids,
    }
}

fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String> {
    let mut map = HashMap::new();
    collect_fn_names_items(&mir.items, "", &mut map);
    map
}

fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>) {
    for item in items {
        match item {
            MirItem::Fn(f) => {
                // Function name already includes namespace from HIR (e.g., "PointStatic.new")
                let name = if f.extern_c {
                    // Extern C: use unmangled name
                    f.name.as_str().to_string()
                } else {
                    mangle("", &f.name.as_str().replace('.', "__"), &f.params)
                };
                map.insert(f.fn_id, name);
            }
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { name: _name, items } => {
                // Namespace already handled by HIR dotted naming; just recurse
                collect_fn_names_items(items, "", map);
            }
        }
    }
}

fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String {
    // Replace dots with __ for LLVM identifier compatibility (namespace paths)
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

fn type_to_mangle(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::Named(s) => format!("{}", s.as_str()),
        HirType::Unique(inner) => format!("unique_{}", type_to_mangle(inner)),
        HirType::Shared(inner) => format!("shared_{}", type_to_mangle(inner)),
        HirType::Weak(inner) => format!("weak_{}", type_to_mangle(inner)),
        HirType::FatPtr { name, .. } => format!("fatptr_{}", name),
        HirType::Array(inner) => format!("arr_{}", type_to_mangle(inner)),
        HirType::Ref(inner, _) => format!("ref_{}", type_to_mangle(inner)),
    }
}

fn lower_items(item: &MirItem, str_map: &HashMap<String, u64>) -> Vec<LirFn> {
    match item {
        MirItem::Fn(f) => vec![lower_fn(f, str_map)],
        MirItem::StructDef { .. } => vec![],
        MirItem::Namespace { items, .. } => {
            items.iter().flat_map(|child| lower_items(child, str_map)).collect()
        }
    }
}

fn lower_fn(f: &MirFn, str_map: &HashMap<String, u64>) -> LirFn {
    let mut ctx = LowerCtx::new(str_map);

    // Extern declarations have no body — return early
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
        };
    }

    for (i, local) in f.locals.iter().enumerate() {
        ctx.emit(LirInst::Alloca(VarId(i), local.ty.clone()));
    }

    for (i, (_, ty)) in f.params.iter().enumerate() {
        let vid = VarId(i);
        ctx.emit(LirInst::Store {
            dest: vid,
            src: LirValue::Param(i as u64),
            ty: ty.clone(),
        });
    }

    lower_stmts(&mut ctx, &f.body);

    // If body didn't end with ret, add a default return
    let needs_ret = ctx
        .current_insts
        .last()
        .map_or(true, |i| !matches!(i, LirInst::Ret(_)));
    if needs_ret {
        let ret = default_ret_value(&f.return_type);
        ctx.emit(LirInst::Ret(ret));
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
    }
}

// ----------------------------------------------------------------
//  LowerCtx
// ----------------------------------------------------------------

struct LowerCtx<'a> {
    tmp: u64,
    block_id: u64,
    current_label: String,
    current_insts: Vec<LirInst>,
    blocks: Vec<LirBlock>,
    str_map: &'a HashMap<String, u64>,
}

impl<'a> LowerCtx<'a> {
    fn new(str_map: &'a HashMap<String, u64>) -> Self {
        Self {
            tmp: 0,
            block_id: 0,
            current_label: "entry".into(),
            current_insts: Vec::new(),
            blocks: Vec::new(),
            str_map,
        }
    }

    fn next_tmp(&mut self) -> u64 {
        let t = self.tmp;
        self.tmp += 1;
        t
    }

    fn next_block_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}{}", prefix, id)
    }

    fn emit(&mut self, inst: LirInst) {
        self.current_insts.push(inst);
    }

    fn finish(&mut self) -> Vec<LirBlock> {
        if !self.current_label.is_empty() {
            let label = std::mem::take(&mut self.current_label);
            let insts = std::mem::take(&mut self.current_insts);
            self.blocks.push(LirBlock { label, insts });
        }
        std::mem::take(&mut self.blocks)
    }

    fn set_current_block(&mut self, label: String) {
        if !self.current_label.is_empty() {
            let old_label = std::mem::replace(&mut self.current_label, label);
            let insts = std::mem::take(&mut self.current_insts);
            if !insts.is_empty() || !self.blocks.is_empty() {
                self.blocks.push(LirBlock {
                    label: old_label,
                    insts,
                });
            }
        } else {
            self.current_label = label;
        }
    }
}

// ----------------------------------------------------------------
//  Statement lowering
// ----------------------------------------------------------------

fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmt]) {
    let mut i = 0;
    while i < stmts.len() {
        // Special handling: when we see Return, lower the expression first,
        // then process cleanup (Drop/Release/Retain) before emitting Ret
        if let MirStmt::Return { value } = &stmts[i] {
            let ret = value.as_ref().map(|v| {
                let val = lower_expr(ctx, v);
                let ty = expr_mir_type(v);
                (val, ty)
            });
            // Process subsequent cleanup statements before Ret
            i += 1;
            while i < stmts.len() {
                match &stmts[i] {
                    MirStmt::Drop(id, ty) => {
                        ctx.emit(LirInst::DropValue(*id, ty.clone()));
                        i += 1;
                    }
                    MirStmt::Retain(id, ty) => {
                        ctx.emit(LirInst::RetainValue(*id, ty.clone()));
                        i += 1;
                    }
                    MirStmt::Release(id, ty) => {
                        ctx.emit(LirInst::ReleaseValue(*id, ty.clone()));
                        i += 1;
                    }
                    _ => break,
                }
            }
            ctx.emit(LirInst::Ret(ret));
            continue;
        }
        lower_stmt(ctx, &stmts[i]);
        i += 1;
    }
}

fn lower_stmt(ctx: &mut LowerCtx, stmt: &MirStmt) {
    match stmt {
        MirStmt::Assign { target, value } => {
            let src = lower_expr(ctx, value);
            if let MirExpr::Local(id, ty, _) = target {
                ctx.emit(LirInst::Store {
                    dest: *id,
                    src,
                    ty: ty.clone(),
                });
            }
        }
        MirStmt::FieldAssign { object, field: _, field_index, field_ty, value } => {
            let obj_ty = expr_mir_type(object);
            // Extract var_id BEFORE lowering (lower_expr loads into Tmp)
            let var_id = match object.as_ref() {
                MirExpr::Local(id, _, _) => {
                    let is_value = !matches!(obj_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_));
                    if is_value { Some(*id) } else { None }
                }
                _ => None,
            };
            let obj_val = lower_expr(ctx, object);
            let obj_tmp = match obj_val {
                LirValue::Tmp(t) => t,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&obj_val), ty: obj_ty.clone() });
                    t
                }
            };
            let src_val = lower_expr(ctx, value);
            let gep_tmp = ctx.next_tmp();
            let iv_tmp = ctx.next_tmp();
            ctx.emit(LirInst::FieldStore {
                dest: obj_tmp, var_id, gep_tmp, iv_tmp,
                src: src_val,
                field_index: *field_index,
                field_ty: field_ty.clone(),
                struct_ty: obj_ty,
            });
        }
        MirStmt::IndexAssign { object, index, value } => {
            let obj_val = lower_expr(ctx, object);
            let obj_tmp = match obj_val {
                LirValue::Tmp(t) => t,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&obj_val), ty: expr_mir_type(object) });
                    t
                }
            };
            let idx_val = lower_expr(ctx, index);
            let src_val = lower_expr(ctx, value);
            let gep_tmp = ctx.next_tmp();
            let obj_ty = strip_ownership(expr_mir_type(object));
            let elem_ty = match &obj_ty {
                HirType::Array(inner) => *inner.clone(),
                _ => HirType::Int,
            };
            ctx.emit(LirInst::IndexStore {
                dest: obj_tmp,
                gep_tmp,
                src: src_val,
                index: idx_val,
                elem_ty,
                array_ty: expr_mir_type(object),
            });
        }
        MirStmt::Return { value } => {
            let ret = value.as_ref().map(|v| {
                let val = lower_expr(ctx, v);
                let ty = expr_mir_type(v);
                (val, ty)
            });
            ctx.emit(LirInst::Ret(ret));
        }
        MirStmt::Expr(expr) => {
            lower_expr(ctx, expr);
        }
        MirStmt::Block(stmts) => {
            lower_stmts(ctx, stmts);
        }
        MirStmt::If {
            cond,
            then_block,
            elifs,
            else_block,
        } => lower_if(ctx, cond, then_block, elifs, else_block),
        MirStmt::While { cond, body } => lower_while(ctx, cond, body),
        MirStmt::Drop(id, ty) => {
            ctx.emit(LirInst::DropValue(*id, ty.clone()));
        }
        MirStmt::Retain(id, ty) => {
            ctx.emit(LirInst::RetainValue(*id, ty.clone()));
        }
        MirStmt::Release(id, ty) => {
            ctx.emit(LirInst::ReleaseValue(*id, ty.clone()));
        }
    }
}

// ----------------------------------------------------------------
//  Expression lowering
// ----------------------------------------------------------------

fn lower_expr(ctx: &mut LowerCtx, expr: &MirExpr) -> LirValue {
    match expr {
        MirExpr::Literal(HirLiteral::String(s), ty) => {
            let idx = ctx.str_map[s];
            let is_string_struct = matches!(ty, HirType::Named(sym) if sym.as_str() == "String");
            
            if is_string_struct {
                // Build String struct: { data: ptr, len: i64 }
                let data_dest = ctx.next_tmp();
                ctx.emit(LirInst::StrGlobal { dest: data_dest, str_idx: idx });
                let data_val = LirValue::Tmp(data_dest);
                
                let len_val = LirValue::Literal(HirLiteral::Int(s.len() as i64), HirType::Int);
                
                let struct_dest = ctx.next_tmp();
                let alloca_tmp = ctx.next_tmp();
                ctx.emit(LirInst::StructLit {
                    dest: struct_dest,
                    alloca_tmp,
                    field_geps: vec![],
                    fields: vec![
                        (data_val, HirType::Named(Symbol::intern("[char]"))),
                        (LirValue::Literal(HirLiteral::Int(s.len() as i64), HirType::Int), HirType::Int),
                    ],
                    struct_name: Symbol::intern("String"),
                    struct_ty: ty.clone(),
                });
                LirValue::Tmp(struct_dest)
            } else {
                let dest = ctx.next_tmp();
                ctx.emit(LirInst::StrGlobal { dest, str_idx: idx });
                LirValue::Tmp(dest)
            }
        }
        MirExpr::Literal(lit, ty) => LirValue::Literal(lit.clone(), ty.clone()),
        MirExpr::Local(id, ty, _) => {
            let dest = ctx.next_tmp();
            ctx.emit(LirInst::Load {
                dest,
                src: *id,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::Binary { op, lhs, rhs, ty } => {
            let lv = lower_expr(ctx, lhs);
            let rv = lower_expr(ctx, rhs);
            let dest = ctx.next_tmp();
            ctx.emit(LirInst::BinOp {
                dest,
                op: *op,
                lhs: lv,
                rhs: rv,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::Unary { op, arg, ty } => {
            let av = lower_expr(ctx, arg);
            let dest = ctx.next_tmp();
            ctx.emit(LirInst::UnaryOp {
                dest,
                op: *op,
                src: av,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::Call { fn_id, args, ty } => {
            let lowered_args: Vec<_> = args
                .iter()
                .map(|a| {
                    let val = lower_expr(ctx, a);
                    let aty = expr_mir_type(a);
                    (val, aty)
                })
                .collect();
            let is_void = matches!(ty, HirType::Void);
            let dest = if is_void { None } else { Some(ctx.next_tmp()) };
            ctx.emit(LirInst::Call {
                dest,
                fn_id: *fn_id,
                args: lowered_args,
                ret_ty: ty.clone(),
            });
            if is_void {
                LirValue::Literal(HirLiteral::Int(0), HirType::Void)
            } else {
                LirValue::Tmp(dest.unwrap())
            }
        }
        MirExpr::Move(inner, _) => lower_expr(ctx, inner),
        MirExpr::Clone(inner, _) => lower_expr(ctx, inner),
        MirExpr::ToUnique(inner, ty) => {
            let inner_val = lower_expr(ctx, inner);
            let inner_ty = match &ty {
                HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                _ => &ty,
            };
            if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
                let src = match inner_val {
                    LirValue::Tmp(_) => inner_val,
                    _ => {
                        let t = ctx.next_tmp();
                        ctx.emit(LirInst::Load {
                            dest: t, src: extract_var(&inner_val),
                            ty: expr_mir_type(inner),
                        });
                        LirValue::Tmp(t)
                    }
                };
                let dest = ctx.next_tmp();
                let alloca_tmp = ctx.next_tmp();
                let malloc_tmp = ctx.next_tmp();
                ctx.emit(LirInst::Conv {
                    dest,
                    alloca_tmp,
                    malloc_tmp,
                    src,
                    kind: ConvKind::ToUnique,
                    src_ty: expr_mir_type(inner),
                    ty: ty.clone(),
                });
                LirValue::Tmp(dest)
            } else {
                inner_val
            }
        }
        MirExpr::ToShared(inner, ty) => {
            let inner_val = lower_expr(ctx, inner);
            let inner_ty = match &ty {
                HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                _ => &ty,
            };
            if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
                let src = match inner_val {
                    LirValue::Tmp(_) => inner_val,
                    _ => {
                        let t = ctx.next_tmp();
                        ctx.emit(LirInst::Load {
                            dest: t, src: extract_var(&inner_val),
                            ty: expr_mir_type(inner),
                        });
                        LirValue::Tmp(t)
                    }
                };
                let dest = ctx.next_tmp();
                let alloca_tmp = ctx.next_tmp();
                let malloc_tmp = ctx.next_tmp();
                ctx.emit(LirInst::Conv {
                    dest,
                    alloca_tmp,
                    malloc_tmp,
                    src,
                    kind: ConvKind::ToShared,
                    src_ty: expr_mir_type(inner),
                    ty: ty.clone(),
                });
                LirValue::Tmp(dest)
            } else {
                inner_val
            }
        }
        MirExpr::ToWeak(inner, ty) => {
            let inner_val = lower_expr(ctx, inner);
            let inner_ty = match &ty {
                HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                _ => &ty,
            };
            if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
                let src = match inner_val {
                    LirValue::Tmp(_) => inner_val,
                    _ => {
                        let t = ctx.next_tmp();
                        ctx.emit(LirInst::Load {
                            dest: t, src: extract_var(&inner_val),
                            ty: expr_mir_type(inner),
                        });
                        LirValue::Tmp(t)
                    }
                };
                let dest = ctx.next_tmp();
                let alloca_tmp = ctx.next_tmp();
                let malloc_tmp = ctx.next_tmp();
                ctx.emit(LirInst::Conv {
                    dest,
                    alloca_tmp,
                    malloc_tmp,
                    src,
                    kind: ConvKind::ToWeak,
                    src_ty: expr_mir_type(inner),
                    ty: ty.clone(),
                });
                LirValue::Tmp(dest)
            } else {
                inner_val
            }
        }
        MirExpr::VirtualCall { receiver, interface: _, method_index, args, ty } => {
            let receiver_val = lower_expr(ctx, receiver);
            let receiver_tmp = match receiver_val {
                LirValue::Tmp(t) => t,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&receiver_val), ty: expr_mir_type(receiver) });
                    t
                }
            };
            let lowered_args: Vec<_> = args.iter()
                .map(|a| {
                    let val = lower_expr(ctx, a);
                    let aty = expr_mir_type(a);
                    (val, aty)
                })
                .collect();
            let is_void = matches!(ty, HirType::Void);
            let fn_dest = if is_void { None } else { Some(ctx.next_tmp()) };
            let data_tmp = ctx.next_tmp();
            let vtable_tmp = ctx.next_tmp();
            let gep_tmp = ctx.next_tmp();
            let fn_ptr_tmp = ctx.next_tmp();
            ctx.emit(LirInst::VirtualCall {
                fn_dest,
                receiver_tmp,
                data_tmp,
                vtable_tmp,
                gep_tmp,
                fn_ptr_tmp,
                method_index: *method_index,
                args: lowered_args,
                ret_ty: ty.clone(),
            });
            if is_void {
                LirValue::Literal(HirLiteral::Int(0), HirType::Void)
            } else {
                LirValue::Tmp(fn_dest.unwrap())
            }
        }
        MirExpr::MakeFatPtr { value, concrete_type, interface_name, ty } => {
            let val = lower_expr(ctx, value);
            let value_src = match &val {
                LirValue::Tmp(_) => val,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&val), ty: expr_mir_type(value) });
                    LirValue::Tmp(t)
                }
            };
            let vtable_name = format!("vtable_{}_{}", concrete_type, interface_name);
            let dest = ctx.next_tmp();
            let malloc_tmp = ctx.next_tmp();
            let bc_tmp = ctx.next_tmp();
            let vtable_gep_tmp = ctx.next_tmp();
            let iv_tmp = ctx.next_tmp();
            ctx.emit(LirInst::MakeFatPtr {
                dest,
                malloc_tmp,
                bc_tmp,
                vtable_gep_tmp,
                iv_tmp,
                value_src,
                value_ty: expr_mir_type(value),
                vtable_name,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::FieldAccess { object, field_index, ty, .. } => {
            let obj_val = lower_expr(ctx, object);
            let obj_tmp = match obj_val {
                LirValue::Tmp(t) => t,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&obj_val), ty: expr_mir_type(object) });
                    t
                }
            };
            let dest = ctx.next_tmp();
            let gep_tmp = ctx.next_tmp();
            ctx.emit(LirInst::FieldAccess {
                dest,
                gep_tmp,
                src: LirValue::Tmp(obj_tmp),
                field_index: *field_index,
                field_ty: ty.clone(),
                struct_ty: expr_mir_type(object),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::StructLiteral { type_name, fields, ty } => {
            let lowered_fields: Vec<_> = fields.iter()
                .map(|(_, e)| {
                    let val = lower_expr(ctx, e);
                    let fty = expr_mir_type(e);
                    (val, fty)
                })
                .collect();
            let dest = ctx.next_tmp();
            let alloca_tmp = ctx.next_tmp();
            let field_geps: Vec<u64> = lowered_fields.iter().map(|_| ctx.next_tmp()).collect();
            ctx.emit(LirInst::StructLit {
                dest,
                alloca_tmp,
                field_geps,
                fields: lowered_fields,
                struct_name: *type_name,
                struct_ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::ArraySized { count, elem_ty, ty } => {
            let dest = ctx.next_tmp();
            let malloc_tmp = ctx.next_tmp();
            let count_tmp = ctx.next_tmp();
            let size_tmp = ctx.next_tmp();
            let elem_count = lower_expr(ctx, count);
            let elem_size = type_size(elem_ty);
            ctx.emit(LirInst::ArraySized {
                dest, malloc_tmp, count_tmp, size_tmp,
                elem_count,
                elem_size,
                elem_ty: elem_ty.clone(),
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::ArrayLiteral(elems, ty) => {
            let lowered_elems: Vec<_> = elems.iter()
                .map(|e| {
                    let val = lower_expr(ctx, e);
                    let ety = expr_mir_type(e);
                    (val, ety)
                })
                .collect();
            let dest = ctx.next_tmp();
            let malloc_tmp = ctx.next_tmp();
            let elem_geps: Vec<u64> = lowered_elems.iter().map(|_| ctx.next_tmp()).collect();
            let elem_ty = match ty {
                HirType::Array(inner) => *inner.clone(),
                _ => HirType::Int,
            };
            ctx.emit(LirInst::ArrayLit {
                dest,
                malloc_tmp,
                elem_geps,
                elems: lowered_elems,
                elem_ty,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
        MirExpr::Ref { expr, mutable, ty } => {
            // Extract VarId directly from the MirExpr (before lowering)
            let var_id = match expr.as_ref() {
                MirExpr::Local(id, _, _) => *id,
                _ => { let _ = lower_expr(ctx, expr); panic!("ref target must be a variable"); }
            };
            let dest = ctx.next_tmp();
            ctx.emit(LirInst::RefInst { dest, var_id, mutable: *mutable, ty: ty.clone() });
            LirValue::Tmp(dest)
        }
        MirExpr::Asm { template, outputs, inputs, ty } => {
            let is_void = matches!(ty, HirType::Void);
            let dest = if is_void { None } else { Some(ctx.next_tmp()) };
            let input_operands: Vec<_> = inputs.iter()
                .map(|(c, e)| { let v = lower_expr(ctx, e); (v, (c.clone(), expr_mir_type(e))) })
                .collect();
            let output_constraints: Vec<String> = outputs.iter().map(|(c, _)| format!("={}", c)).collect();
            let input_constraints: Vec<String> = input_operands.iter().map(|(_, (c, _))| c.clone()).collect();
            let input_vals: Vec<(LirValue, HirType)> = input_operands.into_iter().map(|(v, (_, t))| (v, t)).collect();
            // Outputs become additional temps
            if !outputs.is_empty() {
                // For now, only support single output
            }
            ctx.emit(LirInst::Asm {
                dest,
                template: template.clone(),
                output_constraints,
                input_operands: input_vals,
                input_constraints,
                ret_ty: ty.clone(),
            });
            if is_void {
                LirValue::Literal(HirLiteral::Int(0), HirType::Void)
            } else {
                LirValue::Tmp(dest.unwrap())
            }
        }
        MirExpr::Index { object, index, ty } => {
            let arr_val = lower_expr(ctx, object);
            let idx_val = lower_expr(ctx, index);
            let arr_tmp = match arr_val {
                LirValue::Tmp(t) => t,
                _ => {
                    let t = ctx.next_tmp();
                    ctx.emit(LirInst::Load { dest: t, src: extract_var(&arr_val), ty: expr_mir_type(object) });
                    t
                }
            };
            let dest = ctx.next_tmp();
            let gep_tmp = ctx.next_tmp();
            let load_tmp = ctx.next_tmp();
            let obj_ty = strip_ownership(expr_mir_type(object));
            let elem_ty = match obj_ty {
                HirType::Array(inner) => *inner,
                _ => ty.clone(),
            };
            ctx.emit(LirInst::IndexAccess {
                dest,
                gep_tmp,
                load_tmp,
                arr: LirValue::Tmp(arr_tmp),
                index: idx_val,
                elem_ty,
                ty: ty.clone(),
            });
            LirValue::Tmp(dest)
        }
    }
}

// ----------------------------------------------------------------
//  If / elif / else
// ----------------------------------------------------------------

fn lower_if(
    ctx: &mut LowerCtx,
    cond: &MirExpr,
    then_block: &[MirStmt],
    elifs: &[(MirExpr, Vec<MirStmt>)],
    else_block: &Option<Vec<MirStmt>>,
) {
    let then_lbl = ctx.next_block_label("then");
    let else_lbl = ctx.next_block_label("else");
    let merge_lbl = ctx.next_block_label("ifcont");

    let cond_val = lower_expr(ctx, cond);
    ctx.emit(LirInst::BrCond {
        cond: cond_val,
        true_block: then_lbl.clone(),
        false_block: else_lbl.clone(),
    });

    // Then block
    ctx.set_current_block(then_lbl);
    lower_stmts(ctx, then_block);
    if !block_ends_with_ret(then_block) {
        ctx.emit(LirInst::Br(merge_lbl.clone()));
    }

    // Else / elif chain
    ctx.set_current_block(else_lbl);
    lower_elifs(ctx, elifs, else_block, &merge_lbl);

    // Merge
    ctx.set_current_block(merge_lbl);
}

fn lower_elifs(
    ctx: &mut LowerCtx,
    elifs: &[(MirExpr, Vec<MirStmt>)],
    else_block: &Option<Vec<MirStmt>>,
    merge_lbl: &str,
) {
    if elifs.is_empty() {
        if let Some(stmts) = else_block {
            lower_stmts(ctx, stmts);
        }
        ctx.emit(LirInst::Br(merge_lbl.to_string()));
        return;
    }

    let (cond, body) = &elifs[0];
    let rest = &elifs[1..];

    let then_lbl = ctx.next_block_label("elif.then");
    let next_lbl = ctx.next_block_label("elif.next");

    let cond_val = lower_expr(ctx, cond);
    ctx.emit(LirInst::BrCond {
        cond: cond_val,
        true_block: then_lbl.clone(),
        false_block: next_lbl.clone(),
    });

    ctx.set_current_block(then_lbl);
    lower_stmts(ctx, body);
    if !block_ends_with_ret(body) {
        ctx.emit(LirInst::Br(merge_lbl.to_string()));
    }

    ctx.set_current_block(next_lbl);
    lower_elifs(ctx, rest, else_block, merge_lbl);
}

// ----------------------------------------------------------------
//  While
// ----------------------------------------------------------------

fn lower_while(ctx: &mut LowerCtx, cond: &MirExpr, body: &[MirStmt]) {
    let cond_lbl = ctx.next_block_label("while.cond");
    let body_lbl = ctx.next_block_label("while.body");
    let end_lbl = ctx.next_block_label("while.end");

    ctx.emit(LirInst::Br(cond_lbl.clone()));

    ctx.set_current_block(cond_lbl.clone());
    let cond_val = lower_expr(ctx, cond);
    ctx.emit(LirInst::BrCond {
        cond: cond_val,
        true_block: body_lbl.clone(),
        false_block: end_lbl.clone(),
    });

    ctx.set_current_block(body_lbl);
    lower_stmts(ctx, body);
    ctx.emit(LirInst::Br(cond_lbl));

    ctx.set_current_block(end_lbl);
}

// ----------------------------------------------------------------
//  Helpers
// ----------------------------------------------------------------

fn block_ends_with_ret(stmts: &[MirStmt]) -> bool {
    stmts.iter().any(|s| matches!(s, MirStmt::Return { .. }))
}

fn expr_mir_type(expr: &MirExpr) -> HirType {
    match expr {
        MirExpr::Literal(_, ty)
        | MirExpr::Local(_, ty, _)
        | MirExpr::Binary { ty, .. }
        | MirExpr::Unary { ty, .. }
        | MirExpr::Call { ty, .. }
        | MirExpr::Move(_, ty)
        | MirExpr::Clone(_, ty)
        | MirExpr::ToUnique(_, ty)
        | MirExpr::ToShared(_, ty)
        | MirExpr::ToWeak(_, ty)
        | MirExpr::VirtualCall { ty, .. }
        | MirExpr::MakeFatPtr { ty, .. }
        | MirExpr::FieldAccess { ty, .. }
        | MirExpr::StructLiteral { ty, .. }
        | MirExpr::ArraySized { ty, .. }
        | MirExpr::ArrayLiteral(_, ty)
        | MirExpr::Index { ty, .. }
        | MirExpr::Ref { ty, .. }
        | MirExpr::Asm { ty, .. } => ty.clone(),
    }
}

fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => *inner,
        other => other,
    }
}

fn type_size(ty: &HirType) -> u64 {
    match ty {
        HirType::Int | HirType::Float => 8,
        HirType::Char | HirType::Bool => 1,
        HirType::Void => 0,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_) => 16,
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => type_size(inner),
        HirType::Ref(_, _) => 8,
    }
}

fn extract_var(val: &LirValue) -> VarId {
    match val {
        LirValue::Var(v) => *v,
        _ => panic!("expected Var, got {:?}", val),
    }
}

// ----------------------------------------------------------------
//  String collection (reused from emit.rs)
// ----------------------------------------------------------------

fn collect_strings(mir: &MirProgram) -> Vec<String> {
    let mut strings = Vec::new();
    collect_strings_items(&mir.items, &mut strings);
    let mut seen = std::collections::HashSet::new();
    strings.retain(|s| seen.insert(s.clone()));
    strings
}

fn collect_strings_items(items: &[MirItem], out: &mut Vec<String>) {
    for item in items {
        match item {
            MirItem::Fn(f) => collect_strings_stmt(&f.body, out),
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { items, .. } => collect_strings_items(items, out),
        }
    }
}

fn collect_strings_stmt(stmts: &[MirStmt], out: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            MirStmt::Assign { target, value, .. } => {
                collect_strings_expr(target, out);
                collect_strings_expr(value, out);
            }
            MirStmt::FieldAssign { object, value, .. } => {
                collect_strings_expr(object, out);
                collect_strings_expr(value, out);
            }
            MirStmt::IndexAssign { object, index, value, .. } => {
                collect_strings_expr(object, out);
                collect_strings_expr(index, out);
                collect_strings_expr(value, out);
            }
            MirStmt::Return { value: Some(v) } => collect_strings_expr(v, out),
            MirStmt::If {
                then_block,
                elifs,
                else_block,
                ..
            } => {
                collect_strings_stmt(then_block, out);
                for (_, b) in elifs {
                    collect_strings_stmt(b, out);
                }
                if let Some(b) = else_block {
                    collect_strings_stmt(b, out);
                }
            }
            MirStmt::While { body, .. } => collect_strings_stmt(body, out),
            MirStmt::Expr(e) => collect_strings_expr(e, out),
            MirStmt::Block(stmts) => collect_strings_stmt(stmts, out),
            _ => {}
        }
    }
}

fn collect_strings_expr(expr: &MirExpr, out: &mut Vec<String>) {
    match expr {
        MirExpr::Literal(HirLiteral::String(s), _) => out.push(s.clone()),
        MirExpr::Binary { lhs, rhs, .. } => {
            collect_strings_expr(lhs, out);
            collect_strings_expr(rhs, out);
        }
        MirExpr::Unary { arg, .. } => collect_strings_expr(arg, out),
        MirExpr::Call { args, .. } => {
            for a in args {
                collect_strings_expr(a, out);
            }
        }
        MirExpr::Move(inner, _) | MirExpr::Clone(inner, _)
            | MirExpr::ToUnique(inner, _) | MirExpr::ToShared(inner, _) | MirExpr::ToWeak(inner, _)
            => collect_strings_expr(inner, out),
        MirExpr::VirtualCall { receiver, args, .. } => {
            collect_strings_expr(receiver, out);
            for a in args {
                collect_strings_expr(a, out);
            }
        }
        MirExpr::MakeFatPtr { value, .. } => collect_strings_expr(value, out),
        MirExpr::FieldAccess { object, .. } => collect_strings_expr(object, out),
        MirExpr::StructLiteral { fields, .. } => {
            for (_, e) in fields {
                collect_strings_expr(e, out);
            }
        }
        MirExpr::ArraySized { count, .. } => {
            collect_strings_expr(count, out);
        }
        MirExpr::ArrayLiteral(elems, _) => {
            for e in elems {
                collect_strings_expr(e, out);
            }
        }
        MirExpr::Ref { expr, .. } => {
            collect_strings_expr(expr, out);
        }
        MirExpr::Index { object, index, .. } => {
            collect_strings_expr(object, out);
            collect_strings_expr(index, out);
        }
        MirExpr::Asm { outputs, inputs, .. } => {
            for (_, e) in outputs { collect_strings_expr(e, out); }
            for (_, e) in inputs { collect_strings_expr(e, out); }
        }
        _ => {}
    }
}

fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)> {
    match ty {
        HirType::Void => None,
        HirType::Int => Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), HirType::Int)),
        HirType::Float => Some((LirValue::Literal(HirLiteral::Float(0.0), HirType::Float), HirType::Float)),
        HirType::Char => Some((LirValue::Literal(HirLiteral::Char('\0'), HirType::Char), HirType::Char)),
        HirType::Bool => Some((LirValue::Literal(HirLiteral::Bool(false), HirType::Bool), HirType::Bool)),
        HirType::Named(_) | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_) | HirType::FatPtr { .. } | HirType::Array(_) | HirType::Ref(_, _) => {
            Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), ty.clone()))
        }
    }
}
