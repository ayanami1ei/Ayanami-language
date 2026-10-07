//! #155：比较表达式（i1）→ 数值类型的显式转换（zext / uitofp）。
use super::*;
use super::util::*;

impl MirNode for SMirBoolToNum {
    fn for_each_child_mut(&mut self, f: &mut dyn FnMut(&mut MirNodeBox)) { f(&mut self.expr); }
    fn clone_node(&self) -> Box<dyn MirNode> { Box::new(self.clone()) }
    fn lower_to_lir(&self, ctx: &mut dyn LirLowerCtx) -> LirValue {
        let src = self.expr.lower_to_lir(ctx);
        let src = match src {
            LirValue::Tmp(_) => src,
            _ => {
                let t = ctx.next_tmp();
                ctx.emit(SLirLoad { dest: t, src: extract_var(&src), ty: HirType::Bool }.into());
                LirValue::Tmp(t)
            }
        };
        let dest = ctx.next_tmp();
        ctx.emit(SLirConv {
            dest, alloca_tmp: 0, malloc_tmp: 0,
            src, kind: ConvKind::Cast, src_ty: HirType::Bool, ty: self.ty.clone(),
        }.into());
        LirValue::Tmp(dest)
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}BoolToNum(ty: {})", "", display_hir_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn MirNode)) { f(&*self.expr); }
}
