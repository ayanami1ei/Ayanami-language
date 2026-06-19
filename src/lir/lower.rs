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

fn collect_fn_names(mir: &MirProgram) -> HashMap<FnId, String> {
    let mut map = HashMap::new();
    collect_fn_names_items(&mir.items, "", &mut map);
    map
}

fn collect_fn_names_items(items: &[MirItem], _prefix: &str, map: &mut HashMap<FnId, String>) {
    for item in items {
        match item {
            MirItem::Fn(f) => {
                let name = if f.extern_c {
                    f.name.as_str().to_string()
                } else {
                    mangle("", &f.name.as_str().replace('.', "__"), &f.params)
                };
                map.insert(f.fn_id, name);
            }
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { name: _name, items } => {
                collect_fn_names_items(items, "", map);
            }
        }
    }
}

fn mangle(prefix: &str, name: &str, params: &[(crate::intern::Symbol, HirType)]) -> String {
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
        HirType::Named(s) => s.as_str().replace('<', "_lt_").replace('>', "_gt_")
            .replace(',', "_c_").replace(' ', "_").replace('[', "_lb_").replace(']', "_rb_"),
        HirType::Unique(inner) => format!("unique_{}", type_to_mangle(inner)),
        HirType::Shared(inner) => format!("shared_{}", type_to_mangle(inner)),
        HirType::FnPtr(..) => "fnptr".into(),
        HirType::Weak(inner) => format!("weak_{}", type_to_mangle(inner)),
        HirType::FatPtr { name, .. } => format!("fatptr_{}", name.as_str().replace('<', "_lt_").replace('>', "_gt_")),
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

struct LowerCtx<'a> {
    tmp: u64,
    block_id: u64,
    current_label: String,
    current_insts: Vec<LirNodeBox>,
    blocks: Vec<LirBlock>,
    str_map: &'a HashMap<String, u64>,
    loop_stack: Vec<(String, String)>,
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
            loop_stack: Vec::new(),
        }
    }

    fn next_block_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}{}", prefix, id)
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

    fn finish(&mut self) -> Vec<LirBlock> {
        if !self.current_label.is_empty() {
            let label = std::mem::take(&mut self.current_label);
            let insts = std::mem::take(&mut self.current_insts);
            self.blocks.push(LirBlock { label, insts });
        }
        std::mem::take(&mut self.blocks)
    }
}

impl LirLowerCtx for LowerCtx<'_> {
    fn next_tmp(&mut self) -> u64 {
        let t = self.tmp;
        self.tmp += 1;
        t
    }
    fn emit(&mut self, inst: LirNodeBox) {
        self.current_insts.push(inst);
    }
    fn str_map(&self) -> &HashMap<String, u64> {
        self.str_map
    }
    fn loop_stack(&self) -> &Vec<(String, String)> {
        &self.loop_stack
    }
    fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.loop_stack
    }
    fn next_block_label(&mut self, prefix: &str) -> String {
        self.next_block_label(prefix)
    }
    fn set_current_block(&mut self, label: String) {
        self.set_current_block(label)
    }
}

fn lower_stmts(ctx: &mut LowerCtx, stmts: &[MirStmtBox]) {
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

fn lower_expr(ctx: &mut dyn LirLowerCtx, expr: &MirNodeBox) -> LirValue {
    expr.lower_to_lir(ctx)
}

fn default_ret_value(ty: &HirType) -> Option<(LirValue, HirType)> {
    match ty {
        HirType::Void => None,
        HirType::Int => Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), HirType::Int)),
        HirType::Float => Some((LirValue::Literal(HirLiteral::Float(0.0), HirType::Float), HirType::Float)),
        HirType::Char => Some((LirValue::Literal(HirLiteral::Char('\0'), HirType::Char), HirType::Char)),
        HirType::Bool => Some((LirValue::Literal(HirLiteral::Bool(false), HirType::Bool), HirType::Bool)),
        HirType::Named(_) | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_) | HirType::FatPtr { .. } | HirType::Array(_) | HirType::Ref(_, _) | HirType::FnPtr(..) => {
            Some((LirValue::Literal(HirLiteral::Int(0), HirType::Int), ty.clone()))
        }
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
        HirType::Ref(_, _) | HirType::FnPtr(..) => 8,
    }
}

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
            MirItem::Fn(f) => collect_strings_stmts(&f.body, out),
            MirItem::StructDef { .. } => {}
            MirItem::Namespace { items, .. } => collect_strings_items(items, out),
        }
    }
}

fn collect_strings_stmts(stmts: &[MirStmtBox], out: &mut Vec<String>) {
    for stmt in stmts {
        stmt.for_each_child_expr(&mut |child| {
            collect_strings_dyn(child, out);
        });
    }
}

fn collect_strings_dyn(node: &dyn MirNode, out: &mut Vec<String>) {
    if let Some(s) = node.as_string_literal() {
        out.push(s.to_string());
    }
    node.for_each_child(&mut |child| {
        collect_strings_dyn(child, out);
    });
}

fn block_ends_with_ret(stmts: &[MirStmtBox]) -> bool {
    stmts.iter().any(|s| s.is_return())
}

// ═══════════════════════════════════════════════════════════════════
//  impl MirNode for all 23 SMir* types
// ═══════════════════════════════════════════════════════════════════

impl MirNode for SMirLocal {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let dest = ctx.next_tmp();
        ctx.emit(SLirLoad { dest, src: self.var, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let m = if self.moved { " [moved]" } else { "" };
        writeln!(w, "{:width$}Local(v{} : {}{})", "", self.var.0, display_hir_type(&self.ty), m, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn as_local(&self) -> Option<VarId> { Some(self.var) }
    fn collect_var_ids(&self, vars: &mut std::collections::HashSet<VarId>) { vars.insert(self.var); }
}

impl MirNode for SMirLiteral {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        match &self.val {
            HirLiteral::String(s) => {
                let idx = ctx.str_map()[s];
                let is_string_struct = match &self.ty {
                    HirType::Named(sym) => sym.as_str() == "String",
                    _ => false,
                };
                if is_string_struct {
                    let data_dest = ctx.next_tmp();
                    ctx.emit(SLirStrGlobal { dest: data_dest, str_idx: idx }.into());
                    let data_val = LirValue::Tmp(data_dest);
                    let struct_dest = ctx.next_tmp();
                    let alloca_tmp = ctx.next_tmp();
                    let data_gep = ctx.next_tmp();
                    let len_gep = ctx.next_tmp();
                    ctx.emit(SLirStructLit {
                        dest: struct_dest, alloca_tmp,
                        field_geps: vec![data_gep, len_gep],
                        fields: vec![
                            (data_val, HirType::Named(Symbol::intern("[char]"))),
                            (LirValue::Literal(HirLiteral::Int(s.len() as i64), HirType::Int), HirType::Int),
                        ],
                        struct_name: Symbol::intern("String"),
                        struct_ty: self.ty.clone(),
                    }.into());
                    LirValue::Tmp(struct_dest)
                } else {
                    let dest = ctx.next_tmp();
                    ctx.emit(SLirStrGlobal { dest, str_idx: idx }.into());
                    LirValue::Tmp(dest)
                }
            }
            _ => LirValue::Literal(self.val.clone(), self.ty.clone()),
        }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let s = match &self.val {
            HirLiteral::Int(n) => format!("Int({})", n),
            HirLiteral::Float(n) => format!("Float({})", n),
            HirLiteral::Char(c) => format!("Char('{}')", c),
            HirLiteral::String(s) => format!("String(\"{}\")", s),
            HirLiteral::Bool(b) => format!("Bool({})", b),
        };
        writeln!(w, "{:width$}Literal({})", "", s, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn as_string_literal(&self) -> Option<&str> {
        match &self.val { HirLiteral::String(s) => Some(s.as_str()), _ => None }
    }
}

impl MirNode for SMirBinary {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let lv = self.lhs.lower_to_lir(ctx);
        let rv = self.rhs.lower_to_lir(ctx);
        let dest = ctx.next_tmp();
        let result_ty = match self.op {
            BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => HirType::Bool,
            _ => self.ty.clone(),
        };
        ctx.emit(SLirBinOp { dest, op: self.op, lhs: lv, rhs: rv, ty: self.ty.clone(), result_ty }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Binary {{ op: {:?}, ty: {} }}", "", self.op, display_hir_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  lhs:", "", width = level * 2)?;
        self.lhs.display(level + 1, w)?;
        writeln!(w, "{:width$}  rhs:", "", width = level * 2)?;
        self.rhs.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.lhs); f(&*self.rhs); }
}

impl MirNode for SMirUnary {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let av = self.arg.lower_to_lir(ctx);
        let dest = ctx.next_tmp();
        ctx.emit(SLirUnaryOp { dest, op: self.op, src: av, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Unary {{ op: {:?}, ty: {} }}", "", self.op, display_hir_type(&self.ty), width = level * 2)?;
        self.arg.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.arg); }
}

impl MirNode for SMirCall {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let lowered_args: Vec<_> = self.args.iter().map(|a| {
            let val = a.lower_to_lir(ctx);
            let aty = a.expr_type();
            (val, aty)
        }).collect();
        let is_void = matches!(&self.ty, HirType::Void);
        let dest = if is_void { None } else { Some(ctx.next_tmp()) };
        ctx.emit(SLirCall { dest, fn_id: self.fn_id, args: lowered_args, ret_ty: self.ty.clone() }.into());
        if is_void { LirValue::Literal(HirLiteral::Int(0), HirType::Void) }
        else { LirValue::Tmp(dest.unwrap()) }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Call(fn{}, ty: {})", "", self.fn_id.0, display_hir_type(&self.ty), width = level * 2)?;
        for arg in &self.args { arg.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
}

impl MirNode for SMirMove {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Move(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirNode for SMirClone {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue { self.expr.lower_to_lir(ctx) }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Clone(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirNode for SMirToUnique {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let inner_val = self.expr.lower_to_lir(ctx);
        let inner_ty = match &self.ty { HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(), _ => &self.ty };
        if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
            let src = match inner_val {
                LirValue::Tmp(_) => inner_val,
                _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&inner_val), ty: self.expr.expr_type() }.into()); LirValue::Tmp(t) }
            };
            let dest = ctx.next_tmp(); let alloca_tmp = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
            ctx.emit(SLirConv { dest, alloca_tmp, malloc_tmp, src, kind: ConvKind::ToUnique, src_ty: self.expr.expr_type(), ty: self.ty.clone() }.into());
            LirValue::Tmp(dest)
        } else { inner_val }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToUnique(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirNode for SMirToShared {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let inner_val = self.expr.lower_to_lir(ctx);
        let inner_ty = match &self.ty { HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(), _ => &self.ty };
        if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
            let src = match inner_val {
                LirValue::Tmp(_) => inner_val,
                _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&inner_val), ty: self.expr.expr_type() }.into()); LirValue::Tmp(t) }
            };
            let dest = ctx.next_tmp(); let alloca_tmp = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
            ctx.emit(SLirConv { dest, alloca_tmp, malloc_tmp, src, kind: ConvKind::ToShared, src_ty: self.expr.expr_type(), ty: self.ty.clone() }.into());
            LirValue::Tmp(dest)
        } else { inner_val }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToShared(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirNode for SMirToWeak {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let inner_val = self.expr.lower_to_lir(ctx);
        let inner_ty = match &self.ty { HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(), _ => &self.ty };
        if matches!(inner_ty, HirType::Named(_) | HirType::FatPtr { .. }) {
            let src = match inner_val {
                LirValue::Tmp(_) => inner_val,
                _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&inner_val), ty: self.expr.expr_type() }.into()); LirValue::Tmp(t) }
            };
            let dest = ctx.next_tmp(); let alloca_tmp = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
            ctx.emit(SLirConv { dest, alloca_tmp, malloc_tmp, src, kind: ConvKind::ToWeak, src_ty: self.expr.expr_type(), ty: self.ty.clone() }.into());
            LirValue::Tmp(dest)
        } else { inner_val }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToWeak(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirNode for SMirVirtualCall {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let receiver_val = self.receiver.lower_to_lir(ctx);
        let receiver_tmp = match receiver_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&receiver_val), ty: self.receiver.expr_type() }.into()); t }
        };
        let lowered_args: Vec<_> = self.args.iter().map(|a| {
            let val = a.lower_to_lir(ctx); let aty = a.expr_type(); (val, aty)
        }).collect();
        let is_void = matches!(&self.ty, HirType::Void);
        let fn_dest = if is_void { None } else { Some(ctx.next_tmp()) };
        let data_tmp = ctx.next_tmp(); let vtable_tmp = ctx.next_tmp();
        let gep_tmp = ctx.next_tmp(); let fn_ptr_tmp = ctx.next_tmp();
        ctx.emit(SLirVirtualCall { fn_dest, receiver_tmp, data_tmp, vtable_tmp, gep_tmp, fn_ptr_tmp, method_index: self.method_index, args: lowered_args, ret_ty: self.ty.clone() }.into());
        if is_void { LirValue::Literal(HirLiteral::Int(0), HirType::Void) }
        else { LirValue::Tmp(fn_dest.unwrap()) }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VirtualCall iface={} method={} ty={}", "", self.interface, self.method_index, display_hir_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  receiver:", "", width = level * 2)?;
        self.receiver.display(level + 1, w)?;
        for arg in &self.args { arg.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.receiver); for a in &self.args { f(&**a); } }
}

impl MirNode for SMirMakeFatPtr {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let val = self.value.lower_to_lir(ctx);
        let value_src = match &val {
            LirValue::Tmp(_) => val,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&val), ty: self.value.expr_type() }.into()); LirValue::Tmp(t) }
        };
        let vtable_name = format!("vtable_{}_{}",
            self.concrete_type.as_str().replace('<', "_lt_").replace('>', "_gt_").replace('[', "_lb_").replace(']', "_rb_"),
            self.interface_name.as_str().replace('<', "_lt_").replace('>', "_gt_"));
        let dest = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
        let bc_tmp = ctx.next_tmp(); let vtable_gep_tmp = ctx.next_tmp(); let iv_tmp = ctx.next_tmp();
        ctx.emit(SLirMakeFatPtr { dest, malloc_tmp, bc_tmp, vtable_gep_tmp, iv_tmp, value_src, value_ty: self.value.expr_type(), vtable_name, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}MakeFatPtr {} -> {} ty={}", "", self.concrete_type, self.interface_name, display_hir_type(&self.ty), width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.value); }
}

impl MirNode for SMirEnumConstruct {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, _ctx: &mut dyn LirLowerCtx) -> LirValue {
        LirValue::Literal(HirLiteral::Int(0), HirType::Int)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}EnumConstruct {}.{}", "", self.enum_name, self.variant_name, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }
}

impl MirNode for SMirFnPtr {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let t = ctx.next_tmp();
        ctx.emit(SLirFnAddr { dest: t, fn_id: self.fn_id }.into());
        LirValue::Tmp(t)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}FnPtr(fn{})", "", self.fn_id.0, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl MirNode for SMirCallPtr {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let fn_val = self.fn_ptr.lower_to_lir(ctx);
        let lowered_args: Vec<(LirValue, HirType)> = self.args.iter()
            .map(|a| { let val = a.lower_to_lir(ctx); (val, a.expr_type()) }).collect();
        let dest = ctx.next_tmp();
        let fn_ptr = match &fn_val {
            LirValue::Tmp(t) => LirValue::Tmp(*t),
            LirValue::Var(v) => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: *v, ty: self.fn_ptr.expr_type() }.into()); LirValue::Tmp(t) }
            _ => LirValue::Tmp(ctx.next_tmp()),
        };
        ctx.emit(SLirCallPtr { dest, fn_ptr, args: lowered_args, ret_ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}CallPtr", "", width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.fn_ptr); for a in &self.args { f(&**a); } }
}

impl MirNode for SMirEnumMatch {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let val = self.value.lower_to_lir(ctx);
        let val_tmp = match val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&val), ty: self.value.expr_type() }.into()); t }
        };
        let tag_tmp = ctx.next_tmp(); let gep_tmp = ctx.next_tmp();
        ctx.emit(SLirFieldAccess { dest: tag_tmp, gep_tmp, src: LirValue::Tmp(val_tmp), field_index: 0, field_ty: HirType::Int, struct_ty: self.value.expr_type() }.into());
        if self.arms.is_empty() {
            LirValue::Literal(HirLiteral::Int(0), HirType::Int)
        } else {
            let result_id = VarId(ctx.next_tmp() as usize);
            ctx.emit(SLirAlloca { var: result_id, ty: self.ty.clone() }.into());
            let merge_lbl = format!("ematch{}", ctx.next_tmp());
            let cond_lbls: Vec<String> = (0..self.arms.len()).map(|i| format!("econd{}", i)).collect();
            let arm_lbls: Vec<String> = (0..self.arms.len()).map(|i| format!("earm{}", i)).collect();
            ctx.emit(SLirBr { label: cond_lbls[0].clone() }.into());
            for (i, (tag_val, arm_expr)) in self.arms.iter().enumerate() {
                // create a temporary LowerCtx-like block switch
                ctx.emit(SLirBr { label: cond_lbls[i].clone() }.into());
                let cmp_tmp = ctx.next_tmp();
                ctx.emit(SLirBinOp { dest: cmp_tmp, op: BinaryOp::Eq, lhs: LirValue::Tmp(tag_tmp), rhs: LirValue::Literal(HirLiteral::Int(*tag_val), HirType::Int), ty: HirType::Int, result_ty: HirType::Bool }.into());
                let false_target = if i + 1 < self.arms.len() { cond_lbls[i + 1].clone() } else { merge_lbl.clone() };
                ctx.emit(SLirBrCond { cond: LirValue::Tmp(cmp_tmp), true_block: arm_lbls[i].clone(), false_block: false_target }.into());
                let arm_val = arm_expr.lower_to_lir(ctx);
                ctx.emit(SLirStore { dest: result_id, src: arm_val, ty: self.ty.clone() }.into());
                ctx.emit(SLirBr { label: merge_lbl.clone() }.into());
            }
            let load_tmp = ctx.next_tmp();
            ctx.emit(SLirLoad { dest: load_tmp, src: result_id, ty: self.ty.clone() }.into());
            LirValue::Tmp(load_tmp)
        }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}EnumMatch", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        for (_, e) in &self.arms { e.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) {
        f(&*self.value);
        for (_, e) in &self.arms { f(&**e); }
    }
}

impl MirNode for SMirFieldAccess {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let obj_val = self.object.lower_to_lir(ctx);
        let obj_tmp = match obj_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&obj_val), ty: self.object.expr_type() }.into()); t }
        };
        let dest = ctx.next_tmp(); let gep_tmp = ctx.next_tmp();
        ctx.emit(SLirFieldAccess { dest, gep_tmp, src: LirValue::Tmp(obj_tmp), field_index: self.field_index, field_ty: self.ty.clone(), struct_ty: self.object.expr_type() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}FieldAccess {} ty={}", "", self.field, display_hir_type(&self.ty), width = level * 2)?;
        self.object.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); }
}

impl MirNode for SMirStructLiteral {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let lowered_fields: Vec<_> = self.fields.iter().map(|(_, e)| {
            let val = e.lower_to_lir(ctx); let fty = e.expr_type(); (val, fty)
        }).collect();
        let dest = ctx.next_tmp(); let alloca_tmp = ctx.next_tmp();
        let field_geps: Vec<u64> = lowered_fields.iter().map(|_| ctx.next_tmp()).collect();
        ctx.emit(SLirStructLit { dest, alloca_tmp, field_geps, fields: lowered_fields, struct_name: self.type_name, struct_ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}StructLiteral {} fields={} ty={}", "", self.type_name, self.fields.len(), display_hir_type(&self.ty), width = level * 2)?;
        for (name, e) in &self.fields {
            writeln!(w, "{:width$}  {}:", "", name, width = level * 2)?;
            e.display(level + 1, w)?;
        }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for (_, e) in &self.fields { f(&**e); } }
}

impl MirNode for SMirArrayLiteral {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let lowered: Vec<_> = self.elems.iter().map(|e| {
            let val = e.lower_to_lir(ctx); let ety = e.expr_type(); (val, ety)
        }).collect();
        let dest = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
        let elem_geps: Vec<u64> = lowered.iter().map(|_| ctx.next_tmp()).collect();
        let elem_ty = match &self.ty { HirType::Array(inner) => *inner.clone(), _ => HirType::Int };
        ctx.emit(SLirArrayLit { dest, malloc_tmp, elem_geps, elems: lowered, elem_ty, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ArrayLiteral len={} ty={}", "", self.elems.len(), display_hir_type(&self.ty), width = level * 2)?;
        for e in &self.elems { e.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for e in &self.elems { f(&**e); } }
}

impl MirNode for SMirArraySized {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let dest = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
        let count_tmp = ctx.next_tmp(); let size_tmp = ctx.next_tmp();
        let elem_count = self.count.lower_to_lir(ctx);
        let elem_size = type_size(&self.elem_ty);
        ctx.emit(SLirArraySized { dest, malloc_tmp, count_tmp, size_tmp, elem_count, elem_size, elem_ty: self.elem_ty.clone(), ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ArraySized {{ elem_ty: {}, ty: {} }}", "", display_hir_type(&self.elem_ty), display_hir_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  count:", "", width = level * 2)?;
        self.count.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.count); }
}

impl MirNode for SMirRef {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let var_id = match self.expr.as_local() {
            Some(id) => id,
            None => { let _ = self.expr.lower_to_lir(ctx); panic!("ref target must be a variable"); }
        };
        let dest = ctx.next_tmp();
        ctx.emit(SLirRefInst { dest, var_id, mutable: self.mutable, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let m = if self.mutable { "mut " } else { "" };
        writeln!(w, "{:width$}Ref({}ty: {})", "", m, display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
    fn as_ref(&self) -> Option<(VarId, bool)> {
        self.expr.as_local().map(|var| (var, self.mutable))
    }
}

impl MirNode for SMirIndex {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let arr_val = self.object.lower_to_lir(ctx);
        let idx_val = self.index.lower_to_lir(ctx);
        let arr_tmp = match arr_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&arr_val), ty: self.object.expr_type() }.into()); t }
        };
        let dest = ctx.next_tmp(); let gep_tmp = ctx.next_tmp(); let load_tmp = ctx.next_tmp();
        let obj_ty = strip_ownership(self.object.expr_type());
        let elem_ty = match &obj_ty { HirType::Array(inner) => *inner.clone(), _ => self.ty.clone() };
        ctx.emit(SLirIndexAccess { dest, gep_tmp, load_tmp, arr: LirValue::Tmp(arr_tmp), index: idx_val, elem_ty, ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Index ty={}", "", display_hir_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  object:", "", width = level * 2)?;
        self.object.display(level + 1, w)?;
        writeln!(w, "{:width$}  index:", "", width = level * 2)?;
        self.index.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); }
}

impl MirNode for SMirAsm {
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let is_void = matches!(&self.ty, HirType::Void);
        let dest = if is_void { None } else { Some(ctx.next_tmp()) };
        let input_vals: Vec<_> = self.inputs.iter()
            .map(|(c, e)| { let v = e.lower_to_lir(ctx); (v, (c.clone(), e.expr_type())) }).collect();
        let output_constraints: Vec<String> = self.outputs.iter().map(|(c, _)| format!("={}", c)).collect();
        let input_constraints: Vec<String> = input_vals.iter().map(|(_, (c, _))| c.clone()).collect();
        let input_operands: Vec<(LirValue, HirType)> = input_vals.into_iter().map(|(v, (_, t))| (v, t)).collect();
        ctx.emit(SLirAsm { dest, template: self.template.clone(), output_constraints, input_operands, input_constraints, ret_ty: self.ty.clone() }.into());
        if is_void { LirValue::Literal(HirLiteral::Int(0), HirType::Void) }
        else { LirValue::Tmp(dest.unwrap()) }
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Asm template=\"{}\" outputs={} inputs={}", "", self.template, self.outputs.len(), self.inputs.len(), width = level * 2)?;
        for (i, (c, e)) in self.outputs.iter().enumerate() {
            writeln!(w, "{:width$}  out[{}] constraint={}:", "", i, c, width = level * 2)?;
            e.display(level + 1, w)?;
        }
        for (i, (c, e)) in self.inputs.iter().enumerate() {
            writeln!(w, "{:width$}  in[{}] constraint={}:", "", i, c, width = level * 2)?;
            e.display(level + 1, w)?;
        }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) {
        for (_, e) in &self.outputs { f(&**e); }
        for (_, e) in &self.inputs { f(&**e); }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  impl MirStmtNode for all 13 SMir*Stmt types
// ═══════════════════════════════════════════════════════════════════

impl MirStmtNode for SMirAssignStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let src = self.value.lower_to_lir(ctx);
        if let Some(id) = self.target.as_local() {
            ctx.emit(SLirStore { dest: id, src, ty: self.target.expr_type() }.into());
        }
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Assign", "", width = level * 2)?;
        writeln!(w, "{:width$}  target:", "", width = level * 2)?;
        self.target.display(level + 1, w)?;
        writeln!(w, "{:width$}  value:", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }
}

impl MirStmtNode for SMirFieldAssignStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let obj_ty = self.object.expr_type();
        let var_id = match self.object.as_local() {
            Some(id) => {
                let is_value = !matches!(obj_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_));
                if is_value { Some(id) } else { None }
            }
            None => None,
        };
        let obj_val = self.object.lower_to_lir(ctx);
        let obj_tmp = match obj_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&obj_val), ty: self.object.expr_type() }.into()); t }
        };
        let src_val = self.value.lower_to_lir(ctx);
        let gep_tmp = ctx.next_tmp(); let iv_tmp = ctx.next_tmp();
        ctx.emit(SLirFieldStore { dest: obj_tmp, var_id, gep_tmp, iv_tmp, src: src_val, field_index: self.field_index, field_ty: self.field_ty.clone(), struct_ty: obj_ty }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}FieldAssign field={} index={} ty={}", "", self.field, self.field_index, display_hir_type(&self.field_ty), width = level * 2)?;
        writeln!(w, "{:width$}  object:", "", width = level * 2)?;
        self.object.display(level + 1, w)?;
        writeln!(w, "{:width$}  value:", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.value); }
}

impl MirStmtNode for SMirIndexAssignStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let obj_val = self.object.lower_to_lir(ctx);
        let obj_tmp = match obj_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&obj_val), ty: self.object.expr_type() }.into()); t }
        };
        let idx_val = self.index.lower_to_lir(ctx);
        let src_val = self.value.lower_to_lir(ctx);
        let gep_tmp = ctx.next_tmp();
        let obj_ty = strip_ownership(self.object.expr_type());
        let elem_ty = match &obj_ty { HirType::Array(inner) => *inner.clone(), _ => HirType::Int };
        ctx.emit(SLirIndexStore { dest: obj_tmp, gep_tmp, src: src_val, index: idx_val, elem_ty, array_ty: self.object.expr_type() }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}IndexAssign", "", width = level * 2)?;
        writeln!(w, "{:width$}  object:", "", width = level * 2)?;
        self.object.display(level + 1, w)?;
        writeln!(w, "{:width$}  index:", "", width = level * 2)?;
        self.index.display(level + 1, w)?;
        writeln!(w, "{:width$}  value:", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); f(&*self.value); }
}

impl MirStmtNode for SMirReturnStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let ret = self.value.as_ref().map(|v| {
            let val = v.lower_to_lir(ctx);
            let ty = v.expr_type();
            (val, ty)
        });
        ctx.emit(SLirRet { val: ret }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Return", "", width = level * 2)?;
        if let Some(v) = &self.value { v.display(level + 1, w)?; }
        else { writeln!(w, "{:width$}  (none)", "", width = level * 2)?; }
        Ok(())
    }
    fn is_return(&self) -> bool { true }
    fn return_value(&self) -> Option<&MirNodeBox> { self.value.as_ref() }
}

impl MirStmtNode for SMirIfStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let then_lbl = ctx.next_block_label("then");
        let else_lbl = ctx.next_block_label("else");
        let merge_lbl = ctx.next_block_label("ifcont");

        let cond_val = self.cond.lower_to_lir(ctx);
        ctx.emit(SLirBrCond { cond: cond_val, true_block: then_lbl.clone(), false_block: else_lbl.clone() }.into());

        ctx.set_current_block(then_lbl);
        for stmt in &self.then_block { stmt.lower_to_lir_stmt(ctx); }
        ctx.emit(SLirBr { label: merge_lbl.clone() }.into());

        ctx.set_current_block(else_lbl);
        lower_elifs(ctx, &self.elifs, &self.else_block, &merge_lbl);

        ctx.set_current_block(merge_lbl);
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}If", "", width = level * 2)?;
        writeln!(w, "{:width$}  cond:", "", width = level * 2)?;
        self.cond.display(level + 1, w)?;
        writeln!(w, "{:width$}  then:", "", width = level * 2)?;
        write_stmt_block(&self.then_block, level + 1, w)?;
        for (i, (c, b)) in self.elifs.iter().enumerate() {
            writeln!(w, "{:width$}  elif[{}]:", "", i, width = level * 2)?;
            writeln!(w, "{:width$}    cond:", "", width = level * 2)?;
            c.display(level + 2, w)?;
            write_stmt_block(b, level + 1, w)?;
        }
        if let Some(b) = &self.else_block {
            writeln!(w, "{:width$}  else:", "", width = level * 2)?;
            write_stmt_block(b, level + 1, w)?;
        }
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) {
        for s in &self.then_block { f(&**s); }
        for (_, b) in &self.elifs { for s in b { f(&**s); } }
        if let Some(b) = &self.else_block { for s in b { f(&**s); } }
    }
}

fn lower_elifs(ctx: &mut dyn LirLowerCtx, elifs: &[(MirNodeBox, Vec<MirStmtBox>)], else_block: &Option<Vec<MirStmtBox>>, merge_lbl: &str) {
    if elifs.is_empty() {
        if let Some(stmts) = else_block {
            for s in stmts { s.lower_to_lir_stmt(ctx); }
        }
        ctx.emit(SLirBr { label: merge_lbl.to_string() }.into());
        return;
    }
    let (cond, body) = &elifs[0];
    let rest = &elifs[1..];
    let then_lbl = ctx.next_block_label("elif.then");
    let else_lbl = ctx.next_block_label("elif.else");
    let cond_val = cond.lower_to_lir(ctx);
    ctx.emit(SLirBrCond { cond: cond_val, true_block: then_lbl.clone(), false_block: else_lbl.clone() }.into());
    ctx.set_current_block(then_lbl);
    for s in body { s.lower_to_lir_stmt(ctx); }
    ctx.emit(SLirBr { label: merge_lbl.to_string() }.into());
    ctx.set_current_block(else_lbl);
    lower_elifs(ctx, rest, else_block, merge_lbl);
}

impl MirStmtNode for SMirWhileStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let cond_lbl = ctx.next_block_label("while.cond");
        let body_lbl = ctx.next_block_label("while.body");
        let end_lbl = ctx.next_block_label("while.end");

        let cond_lbl2 = cond_lbl.clone();
        ctx.loop_stack_mut().push((cond_lbl.clone(), end_lbl.clone()));
        ctx.emit(SLirBr { label: cond_lbl.clone() }.into());

        ctx.set_current_block(cond_lbl2);
        let cond_val = self.cond.lower_to_lir(ctx);
        ctx.emit(SLirBrCond { cond: cond_val, true_block: body_lbl.clone(), false_block: end_lbl.clone() }.into());

        ctx.set_current_block(body_lbl);
        for s in &self.body { s.lower_to_lir_stmt(ctx); }
        ctx.emit(SLirBr { label: cond_lbl }.into());

        ctx.loop_stack_mut().pop();
        ctx.set_current_block(end_lbl);
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}While", "", width = level * 2)?;
        writeln!(w, "{:width$}  cond:", "", width = level * 2)?;
        self.cond.display(level + 1, w)?;
        writeln!(w, "{:width$}  body:", "", width = level * 2)?;
        write_stmt_block(&self.body, level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.cond); }
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.body { f(&**s); } }
}

impl MirStmtNode for SMirBreakStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        if let Some((_, end_lbl)) = ctx.loop_stack().last() {
            ctx.emit(SLirBr { label: end_lbl.clone() }.into());
        }
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Break", "", width = level * 2)
    }
}

impl MirStmtNode for SMirContinueStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        if let Some((cond_lbl, _)) = ctx.loop_stack().last() {
            ctx.emit(SLirBr { label: cond_lbl.clone() }.into());
        }
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Continue", "", width = level * 2)
    }
}

impl MirStmtNode for SMirExprStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        self.expr.lower_to_lir(ctx);
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Expr", "", width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirStmtNode for SMirBlockStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        for s in &self.stmts { s.lower_to_lir_stmt(ctx); }
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Block {{", "", width = level * 2)?;
        for s in &self.stmts { s.display_stmt(level + 1, w)?; }
        writeln!(w, "{:width$}}}", "", width = level * 2)?;
        Ok(())
    }
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.stmts { f(&**s); } }
}

impl MirStmtNode for SMirDropStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        ctx.emit(SLirDropValue { var: self.var, ty: self.ty.clone() }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Drop(v{} : {})", "", self.var.0, display_hir_type(&self.ty), width = level * 2)
    }
    fn as_drop(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
}

impl MirStmtNode for SMirRetainStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        ctx.emit(SLirRetainValue { var: self.var, ty: self.ty.clone() }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Retain(v{} : {})", "", self.var.0, display_hir_type(&self.ty), width = level * 2)
    }
    fn as_retain(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
}

impl MirStmtNode for SMirReleaseStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        ctx.emit(SLirReleaseValue { var: self.var, ty: self.ty.clone() }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Release(v{} : {})", "", self.var.0, display_hir_type(&self.ty), width = level * 2)
    }
    fn as_release(&self) -> Option<(VarId, &HirType)> { Some((self.var, &self.ty)) }
}

fn write_stmt_block(stmts: &[MirStmtBox], level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    writeln!(w, "{:width$}Block {{", "", width = level * 2)?;
    for s in stmts { s.display_stmt(level + 1, w)?; }
    writeln!(w, "{:width$}}}", "", width = level * 2)?;
    Ok(())
}

fn display_hir_type(ty: &HirType) -> String {
    crate::hir::display::display_type(ty)
}

fn extract_var(val: &LirValue) -> VarId {
    match val {
        LirValue::Var(v) => *v,
        _ => panic!("expected Var, got {:?}", val),
    }
}


