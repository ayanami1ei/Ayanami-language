use super::*;
use super::util::*;
use super::fn_lower::{strip_ownership, type_size};

impl MirNode for SMirEnumConstruct {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for a in &mut self.args { f(a); } }
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
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.fn_ptr); for a in &mut self.args { f(a); } }
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
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.fn_ptr); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }

}

impl MirNode for SMirEnumMatch {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.value); for (_, e) in &mut self.arms { f(e); } }
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
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); }
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
    fn as_field_access(&self) -> Option<(&MirNodeBox, usize)> { Some((&self.object, self.field_index)) }
}

impl MirNode for SMirStructLiteral {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for (_, e) in &mut self.fields { f(e); } }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn struct_literal_fields(&self) -> Option<&[(Symbol, MirNodeBox)]> { Some(&self.fields) }
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
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for a in &mut self.elems { f(a); } }
    fn is_alloc(&self) -> bool { true }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let lowered: Vec<_> = self.elems.iter().map(|e| {
            let val = e.lower_to_lir(ctx); let ety = e.expr_type(); (val, ety)
        }).collect();
        let dest = ctx.next_tmp(); let malloc_tmp = ctx.next_tmp();
        let elem_geps: Vec<u64> = lowered.iter().map(|_| ctx.next_tmp()).collect();
        // `[..]` 现在直接是 Unique(Array/ArraySized)：先剥拥有包装再取元素类型
        let elem_ty = match strip_ownership(self.ty.clone()) { HirType::Array(inner) | HirType::ArraySized(inner, _) => (*inner).clone(), _ => HirType::Int };
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
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.count); }
    fn is_alloc(&self) -> bool { true }
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

impl MirNode for SMirIndex {
    fn as_index(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.index)) }
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.object); f(&mut self.index); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let arr_val = self.object.lower_to_lir(ctx);
        let idx_val = self.index.lower_to_lir(ctx);
        let arr_tmp = match arr_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&arr_val), ty: self.object.expr_type() }.into()); t }
        };
        // `ref [T]`/`ref mut [T]`：先通过 ref 取出数组指针
        let arr_tmp = super::mir_ref::array_base_through_ref(ctx, arr_tmp, &self.object.expr_type());
        let dest = ctx.next_tmp(); let gep_tmp = ctx.next_tmp(); let load_tmp = ctx.next_tmp();
        let obj_ty = strip_ownership(self.object.expr_type());
        let elem_ty = super::mir_ref::array_elem_ty(&obj_ty).unwrap_or_else(|| self.ty.clone());
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
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { for (_, e) in &mut self.outputs { f(e); } for (_, e) in &mut self.inputs { f(e); } }
    fn is_asm(&self) -> bool { true }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let is_void = matches!(&self.ty, HirType::Void);
        let dest = if is_void { None } else { Some(ctx.next_tmp()) };
        let input_vals: Vec<_> = self.inputs.iter()
            .map(|(c, e)| { let v = e.lower_to_lir(ctx); (v, (c.clone(), e.expr_type())) }).collect();
        let output_operands: Vec<(LirValue, HirType, Option<VarId>)> = self.outputs.iter()
            .map(|(_c, e)| { let v = e.lower_to_lir(ctx); (v, e.expr_type(), e.as_local()) }).collect();
        let output_constraints: Vec<String> = self.outputs.iter().map(|(c, _)| c.clone()).collect();
        let input_constraints: Vec<String> = input_vals.iter().map(|(_, (c, _))| c.clone()).collect();
        let input_operands: Vec<(LirValue, HirType)> = input_vals.into_iter().map(|(v, (_, t))| (v, t)).collect();
        ctx.emit(SLirAsm { dest, template: self.template.clone(), output_constraints, output_operands, input_operands, input_constraints, ret_ty: self.ty.clone() }.into());
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
