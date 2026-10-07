//! ref 相关 MIR 节点的 LIR 降级：取引用 / 解引用 / 穿透引用写入。
use super::*;
use super::util::*;
use super::place::{array_base_through_ref, place_ptr};

impl MirNode for SMirRef {
    fn ref_expr(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
    fn ref_expr_mut(&mut self) -> Option<&mut MirNodeBox> { Some(&mut self.expr) }
    fn refs_global(&self) -> Option<Symbol> {
        self.expr.as_global()
            .or_else(|| self.expr.as_deref().and_then(|i| i.as_global()))
    }
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let dest = ctx.next_tmp();
        if let Some(var_id) = self.expr.as_local() {
            ctx.emit(SLirRefInst { dest, var_id, mutable: self.mutable, ty: self.ty.clone() }.into());
        } else if let Some(ptr) = place_ptr(&self.expr, ctx) {
            // 字段（含嵌套）：直接取字段地址，避免复制到临时栈槽
            return ptr;
        } else {
            // 临时值（如函数返回值）：溢出到栈槽后取引用
            let pointee_ty = self.expr.expr_type();
            let src = self.expr.lower_to_lir(ctx);
            let alloca_tmp = ctx.next_tmp();
            ctx.emit(SLirRefTmp { dest, alloca_tmp, src, mutable: self.mutable, ty: pointee_ty }.into());
        }
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

impl MirNode for SMirGlobal {
    fn as_global(&self) -> Option<Symbol> { Some(self.name) }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let dest = ctx.next_tmp();
        ctx.emit(SLirGlobalAddr { dest, name: self.name }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Global({}, mut: {})", "", self.name.as_str(), self.mutable, width = level * 2)
    }
    fn expr_type(&self) -> HirType { HirType::Ref(Box::new(self.ty.clone()), self.mutable) }
    fn for_each_child(&self, _f: &mut dyn FnMut(&dyn MirNode)) {}
}

impl MirNode for SMirDeref {
    fn as_deref(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let ptr_val = self.expr.lower_to_lir(ctx);
        let ptr_tmp = as_ptr_tmp(ctx, ptr_val, &self.expr.expr_type());
        let dest = ctx.next_tmp(); let gep_tmp = ctx.next_tmp(); let load_tmp = ctx.next_tmp();
        // 对任意指针按 index 0 解引用（getelementptr T, ptr p, i64 0 + load）
        ctx.emit(SLirIndexAccess {
            dest, gep_tmp, load_tmp,
            arr: LirValue::Tmp(ptr_tmp),
            index: LirValue::Literal(HirLiteral::Int(0), HirType::Int),
            elem_ty: self.ty.clone(),
            ty: self.ty.clone(),
        }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Deref ty={}", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

impl MirStmtNode for SMirDerefAssignStmt { fn span(&self) -> crate::span::Span { self.span }
    fn for_each_child_expr_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.target); f(&mut self.value); }
    fn clone_stmt(&self) -> Box<dyn MirStmtNode> { Box::new(self.clone()) }
    fn lower_to_lir_stmt(&self, ctx: &mut dyn LirLowerCtx) {
        let ptr_val = self.target.lower_to_lir(ctx);
        let ptr_tmp = as_ptr_tmp(ctx, ptr_val, &self.target.expr_type());
        let src_val = self.value.lower_to_lir(ctx);
        let gep_tmp = ctx.next_tmp();
        let elem_ty = match self.target.expr_type() {
            HirType::Ref(inner, _) => *inner,
            _ => self.value.expr_type(),
        };
        // 对任意指针按 index 0 写入
        ctx.emit(SLirIndexStore {
            dest: ptr_tmp, gep_tmp, src: src_val,
            index: LirValue::Literal(HirLiteral::Int(0), HirType::Int),
            elem_ty, array_ty: self.target.expr_type(),
        }.into());
    }
    fn display_stmt(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}DerefAssign", "", width = level * 2)?;
        writeln!(w, "{:width$}  target:", "", width = level * 2)?;
        self.target.display(level + 1, w)?;
        writeln!(w, "{:width$}  value:", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn for_each_child_expr(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.target); f(&*self.value); }
    fn deref_assign_parts(&self) -> Option<(&MirNodeBox, &MirNodeBox)> { Some((&self.target, &self.value)) }
}

/// 递归求字段/局部变量地址（字段移出清零用）；不可寻址返回 None。
pub(super) fn lower_field_addr_of(ctx: &mut dyn LirLowerCtx, node: &dyn MirNode) -> Option<LirValue> {
    // 解引用位置：内层为指针（ref/unique）时，指针值即地址
    // （match ref 接收者时字段移出清零原值，避免浅拷贝副本悬挂）
    if let Some(inner) = node.as_deref() {
        if matches!(inner.expr_type(), HirType::Ref(..) | HirType::Unique(_)) {
            return Some(inner.lower_to_lir(ctx));
        }
        return lower_field_addr_of(ctx, &**inner);
    }
    if let Some((obj, idx)) = node.as_field_access() {
        let base = lower_field_addr_of(ctx, &**obj)?;
        let dest = ctx.next_tmp();
        ctx.emit(SLirFieldAddr { dest, obj: base, field_index: idx, struct_ty: obj.expr_type() }.into());
        return Some(LirValue::Tmp(dest));
    }
    let ty = node.expr_type();
    if let Some(var) = node.as_local() {
        if matches!(ty, HirType::Ref(..) | HirType::Unique(_)) {
            // 指针型局部：load 出的指针值即地址
            return Some(node.lower_to_lir(ctx));
        }
        let dest = ctx.next_tmp();
        ctx.emit(SLirRefInst { dest, var_id: var, mutable: true, ty }.into());
        return Some(LirValue::Tmp(dest));
    }
    if matches!(ty, HirType::Ref(..) | HirType::Unique(_)) {
        return Some(node.lower_to_lir(ctx));
    }
    None
}

impl MirNode for SMirMove {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn move_expr(&self) -> Option<&MirNodeBox> { Some(&self.expr) }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        // 字段移出：取地址 → load → 源字段清零（源结构 drop 不再重复释放）
        if let Some((obj, idx)) = self.expr.as_field_access() {
            if let Some(base) = lower_field_addr_of(ctx, &**obj) {
                let dest = ctx.next_tmp(); let gep = ctx.next_tmp();
                ctx.emit(SLirFieldTake {
                    dest, gep_tmp: gep, obj: base, field_index: idx,
                    field_ty: self.expr.expr_type(), struct_ty: obj.expr_type(),
                }.into());
                return LirValue::Tmp(dest);
            }
        }
        // 数组元素读取：非破坏性深拷贝（clone）——源元素保留，集合 drop 释放原值。
        // SMove(Index) 仅由 implicit_move 对非 Copy 元素生成。
        if let Some((obj, idx)) = self.expr.as_index() {
            let elem_ty = self.expr.expr_type();
            if !elem_ty.is_copy() {
                let arr_val = obj.lower_to_lir(ctx);
                let idx_val = idx.lower_to_lir(ctx);
                let arr_tmp = as_ptr_tmp(ctx, arr_val, &obj.expr_type());
                let arr_tmp = array_base_through_ref(ctx, arr_tmp, &obj.expr_type());
                // 可深拷贝元素 → 非破坏性 clone；否则（闭包/拥有胖指针/裸动态数组）
                // → 移出（load + 源槽清零），避免浅拷贝别名双释放
                if crate::lir::ir::is_cloneable(&elem_ty, &ctx.struct_defs()) {
                    let addr = ctx.next_tmp();
                    ctx.emit(SLirIndexAddr { dest: addr, arr_tmp, index: idx_val, elem_ty: elem_ty.clone() }.into());
                    let dest = ctx.next_tmp(); let alloca = ctx.next_tmp();
                    ctx.emit(SLirClone { dest, alloca_tmp: alloca, src: LirValue::Tmp(addr), ty: elem_ty }.into());
                    return LirValue::Tmp(dest);
                }
                let dest = ctx.next_tmp(); let gep = ctx.next_tmp();
                ctx.emit(SLirIndexTake {
                    dest, gep_tmp: gep, obj: LirValue::Tmp(arr_tmp), index: idx_val, elem_ty,
                }.into());
                return LirValue::Tmp(dest);
            }
        }
        // 局部变量移出：load + 源清零（循环回边重复移动时避免双重释放）
        if let Some(id) = self.expr.as_local() {
            if !self.expr.expr_type().is_copy() {
                let dest = ctx.next_tmp();
                ctx.emit(SLirLocalTake { dest, var: id, ty: self.expr.expr_type() }.into());
                return LirValue::Tmp(dest);
            }
        }
        self.expr.lower_to_lir(ctx)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Move(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}

/// 取指针临时量：已是 Tmp 直接用；否则 load 到新临时量。
pub(super) fn as_ptr_tmp(ctx: &mut dyn LirLowerCtx, val: LirValue, ty: &HirType) -> u64 {
    match val {
        LirValue::Tmp(t) => t,
        _ => {
            let t = ctx.next_tmp();
            ctx.emit(SLirLoad { dest: t, src: extract_var(&val), ty: ty.clone() }.into());
            t
        }
    }
}
