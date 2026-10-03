use super::*;
use super::util::*;
use super::fn_lower::strip_ownership;


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
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }    fn assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.target, &self.value)) }

}

impl MirStmtNode for SMirFieldAssignStmt {
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let obj_ty = self.object.expr_type();
        let var_id = match self.object.as_local() {
            Some(id) => {
                let is_value = !matches!(obj_ty, HirType::Unique(_) | HirType::Ref(..));
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
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.value); }    fn field_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.value)) }

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
        let elem_ty = match &obj_ty { HirType::Array(inner) | HirType::ArraySized(inner, _) => *inner.clone(), _ => HirType::Int };
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
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.object); f(&*self.index); f(&*self.value); }    fn index_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox, &MirNodeBox)> { Some((&self.object, &self.index, &self.value)) }

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
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) {
        if let Some(v) = &self.value { f(&**v); }
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
    }    fn as_if(&self) -> Option<IfParts<'_>> { Some((&self.cond, &self.then_block, &self.elifs, &self.else_block)) }

}

pub(super) fn lower_elifs(ctx: &mut dyn LirLowerCtx, elifs: &[(MirNodeBox, Vec<MirStmtBox>)], else_block: &Option<Vec<MirStmtBox>>, merge_lbl: &str) {
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
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.body { f(&**s); } }    fn as_while(&self) -> Option<WhileParts<'_>> { Some((&self.cond, &self.body)) }

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
    }    fn is_break(&self) -> bool { true }

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
    }    fn is_continue(&self) -> bool { true }

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
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }    fn expr_part(&self) -> Option<&MirNodeBox> { Some(&self.expr) }

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
    fn for_each_child_stmt(&self, f: &mut dyn FnMut(&dyn MirStmtNode)) { for s in &self.stmts { f(&**s); } }    fn as_block(&self) -> Option<&[MirStmtBox]> { Some(&self.stmts) }

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


