use super::*;
use super::util::*;

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
                let idx = match ctx.str_map().get(s) {
                    Some(i) => *i,
                    None => {
                        eprintln!("DEBUG: missing string in str_map: {:?} (len {})", s, s.len());
                        // Fallback: add it dynamically (shouldn't happen)
                        0
                    }
                };
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
    fn call_fn_id(&self) -> Option<crate::hir::ty::FnId> { Some(self.fn_id) }
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
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }

}

impl MirNode for SMirMove {
    fn move_expr(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
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
        let inner_ty = match &self.ty { HirType::Unique(i) => i.as_ref(), _ => &self.ty };
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
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.receiver); for a in &self.args { f(&**a); } }    fn is_call(&self) -> bool { true }

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
