//! 转换类 MIR 节点的 LIR 降级（Cast / Reinterpret；从 mir_expr.rs 拆出）。

use super::*;
use super::util::*;

impl MirNode for SMirReinterpret {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let src = self.expr.lower_to_lir(ctx);
        let dest = ctx.next_tmp();
        ctx.emit(SLirConv {
            dest,
            alloca_tmp: 0,
            malloc_tmp: 0,
            src,
            kind: ConvKind::Reinterpret,
            src_ty: self.expr.expr_type(),
            ty: self.ty.clone(),
        }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Reinterpret(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl MirNode for SMirCast {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let src = self.expr.lower_to_lir(ctx);
        let dest = ctx.next_tmp();
        ctx.emit(SLirConv {
            dest,
            alloca_tmp: 0,
            malloc_tmp: 0,
            src,
            kind: ConvKind::Cast,
            src_ty: self.expr.expr_type(),
            ty: self.ty.clone(),
        }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Cast(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}



