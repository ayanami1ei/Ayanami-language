use super::*;

impl HirNode for SField {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirFieldAccess {
            object: self.object.lower_to_mir(moved),
            field: self.field,
            field_index: self.field_index,
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}FieldAccess {} ty={}", "", self.field, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.object.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SStruct {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirStructLiteral {
            type_name: self.type_name,
            fields: self.fields.iter().map(|(n, e)| (*n, e.lower_to_mir(moved))).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}StructLiteral {} fields={} ty={}", "", self.type_name, self.fields.len(), crate::hir::display::display_type(&self.ty), width = level * 2)?;
        for (name, e) in &self.fields {
            writeln!(w, "{:width$}  {}:", "", name, width = level * 2)?;
            e.display(level + 1, w)?;
        }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SArrLit {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirArrayLiteral {
            elems: self.elems.iter().map(|e| e.lower_to_mir(moved)).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ArrayLiteral len={} ty={}", "", self.elems.len(), crate::hir::display::display_type(&self.ty), width = level * 2)?;
        for e in &self.elems { e.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SArrSz {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirArraySized {
            count: self.count.lower_to_mir(moved),
            elem_ty: self.elem_ty.clone(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ArraySized {{ elem_ty: {}, ty: {} }}", "", crate::hir::display::display_type(&self.elem_ty), crate::hir::display::display_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  count:", "", width = level * 2)?;
        self.count.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SRef {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirRef { expr: self.expr.lower_to_mir(moved), mutable: self.mutable, ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let m = if self.mutable { "mut " } else { "" };
        writeln!(w, "{:width$}Ref({}ty: {})", "", m, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SIdx {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirIndex {
            object: self.object.lower_to_mir(moved),
            index: self.index.lower_to_mir(moved),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Index ty={}", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  object:", "", width = level * 2)?;
        self.object.display(level + 1, w)?;
        writeln!(w, "{:width$}  index:", "", width = level * 2)?;
        self.index.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}
