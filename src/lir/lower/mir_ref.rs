//! ref 相关 MIR 节点的 LIR 降级：取引用 / 解引用 / 穿透引用写入。
use super::*;
use super::util::*;
use super::fn_lower::strip_ownership;

/// #91：递归计算「可写位置」的地址（局部 / 引用值 / 字段，含嵌套字段）
pub(super) fn place_ptr(expr: &MirNodeBox, ctx: &mut dyn LirLowerCtx) -> Option<LirValue> {
    // 引用值本身就是指针（局部需从栈槽加载指针）
    if matches!(expr.expr_type(), HirType::Ref(..)) {
        let val = expr.lower_to_lir(ctx);
        return Some(match val {
            LirValue::Tmp(_) => val,
            other => {
                let t = ctx.next_tmp();
                ctx.emit(SLirLoad { dest: t, src: extract_var(&other), ty: expr.expr_type() }.into());
                LirValue::Tmp(t)
            }
        });
    }
    if let Some(v) = expr.as_local() {
        return Some(LirValue::Var(v));
    }
    // M6.2：全局地址（直接）与「解引用全局」（自动借用 `f(STATIC)`）都取全局指针
    if let Some(name) = expr.as_global() {
        let dest = ctx.next_tmp();
        ctx.emit(SLirGlobalAddr { dest, name }.into());
        return Some(LirValue::Tmp(dest));
    }
    if let Some(inner) = expr.as_deref() {
        if let Some(name) = inner.as_global() {
            let dest = ctx.next_tmp();
            ctx.emit(SLirGlobalAddr { dest, name }.into());
            return Some(LirValue::Tmp(dest));
        }
    }
    // #130：数组元素地址（`o.xs[0].a = v` 的对象是索引表达式）
    if let Some((obj, index)) = expr.as_index() {
        let obj_val = obj.lower_to_lir(ctx);
        let arr_tmp = match obj_val {
            LirValue::Tmp(t) => t,
            other => {
                let t = ctx.next_tmp();
                ctx.emit(SLirLoad { dest: t, src: extract_var(&other), ty: obj.expr_type() }.into());
                t
            }
        };
        let idx_val = index.lower_to_lir(ctx);
        let arr_tmp = array_base_through_ref(ctx, arr_tmp, &obj.expr_type());
        let obj_ty = strip_ownership(obj.expr_type());
        let elem_ty = array_elem_ty(&obj_ty).unwrap_or(HirType::Int);
        let dest = ctx.next_tmp();
        ctx.emit(SLirIndexAddr { dest, arr_tmp, index: idx_val, elem_ty }.into());
        return Some(LirValue::Tmp(dest));
    }
    if let Some((obj, field_index)) = expr.as_field_access() {
        let obj_ptr = place_ptr(obj, ctx)?;
        let dest = ctx.next_tmp();
        ctx.emit(SLirFieldAddr { dest, obj: obj_ptr, field_index, struct_ty: obj.expr_type() }.into());
        return Some(LirValue::Tmp(dest));
    }
    None
}

/// 从（可能带 `ref` / 拥有包装的）数组类型取元素类型。
pub(super) fn array_elem_ty(ty: &HirType) -> Option<HirType> {
    let base = match ty {
        HirType::Ref(inner, _) => strip_ownership((**inner).clone()),
        other => strip_ownership(other.clone()),
    };
    match base {
        HirType::Array(inner) | HirType::ArraySized(inner, _) => Some((*inner).clone()),
        _ => None,
    }
}

/// `ref [T]` / `ref mut [T]` 索引：ref 值是调用方数组变量槽的地址（ptr 的地址），
/// 索引前需先取出数组指针；普通拥有数组/数组值原样返回。
pub(super) fn array_base_through_ref(ctx: &mut dyn LirLowerCtx, base_tmp: u64, ty: &HirType) -> u64 {
    if let HirType::Ref(inner, _) = ty {
        if matches!(strip_ownership((**inner).clone()), HirType::Array(_) | HirType::ArraySized(_, _)) {
            let t = ctx.next_tmp();
            ctx.emit(SLirLoadPtr { dest: t, src: LirValue::Tmp(base_tmp), ty: (**inner).clone() }.into());
            return t;
        }
    }
    base_tmp
}

impl MirNode for SMirRef {
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
        let ptr_tmp = match ptr_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&ptr_val), ty: self.expr.expr_type() }.into()); t }
        };
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
        let ptr_tmp = match ptr_val {
            LirValue::Tmp(t) => t,
            _ => { let t = ctx.next_tmp(); ctx.emit(SLirLoad { dest: t, src: extract_var(&ptr_val), ty: self.target.expr_type() }.into()); t }
        };
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
