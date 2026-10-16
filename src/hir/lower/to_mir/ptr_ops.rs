//! P3：裸指针算术 / volatile 读写的 HIR→MIR 降级。

use super::*;

impl HirNode for SPtrOffset {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirPtrOffset {
            ptr: self.ptr.lower_to_mir(moved),
            index: self.index.lower_to_mir(moved),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}PtrOffset(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        self.index.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.ptr);
        f(&*self.index);
    }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        self.ptr.record_moves(moved);
        self.index.record_moves(moved);
    }
}

impl HirNode for SVolatileLoad {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirVolatileLoad { ptr: self.ptr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VolatileLoad(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.ptr);
    }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        self.ptr.record_moves(moved);
    }
}

impl HirNode for SVolatileStore {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirVolatileStore {
            ptr: self.ptr.lower_to_mir(moved),
            value: self.value.lower_to_mir(moved),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VolatileStore(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.ptr.display(level + 1, w)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.ptr);
        f(&*self.value);
    }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        self.ptr.record_moves(moved);
        self.value.record_moves(moved);
    }
}
