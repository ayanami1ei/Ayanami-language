//! P3：裸指针算术 / volatile 读写的 MIR→LIR 降级。

use super::*;
use super::util::*;
use super::mir_ref::as_ptr_tmp;

impl MirNode for SMirPtrOffset {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.ptr); f(&mut self.index); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let ptr_ty = self.ptr.expr_type();
        let elem_ty = match &ptr_ty {
            HirType::RawPtr(inner) => inner.as_ref().clone(),
            other => super::fn_lower::strip_ownership(other.clone()),
        };
        let ptr_val = self.ptr.lower_to_lir(ctx);
        let arr_tmp = as_ptr_tmp(ctx, ptr_val, &ptr_ty);
        let index = self.index.lower_to_lir(ctx);
        let dest = ctx.next_tmp();
        ctx.emit(SLirIndexAddr { dest, arr_tmp, index, elem_ty }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}PtrOffset(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        self.index.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl MirNode for SMirVolatileLoad {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.ptr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let ptr_ty = self.ptr.expr_type();
        let ptr_val = self.ptr.lower_to_lir(ctx);
        let ptr_tmp = as_ptr_tmp(ctx, ptr_val, &ptr_ty);
        let dest = ctx.next_tmp();
        ctx.emit(SLirVolatileLoad { dest, src: LirValue::Tmp(ptr_tmp), ty: self.ty.clone() }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VolatileLoad(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl MirNode for SMirVolatileStore {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.ptr); f(&mut self.value); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let ptr_ty = self.ptr.expr_type();
        let ptr_val = self.ptr.lower_to_lir(ctx);
        let ptr_tmp = as_ptr_tmp(ctx, ptr_val, &ptr_ty);
        let val = self.value.lower_to_lir(ctx);
        ctx.emit(SLirVolatileStore {
            ptr: LirValue::Tmp(ptr_tmp), src: val, ty: self.value.expr_type(),
        }.into());
        LirValue::Literal(HirLiteral::Int(0), HirType::Void)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VolatileStore", "", width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}
