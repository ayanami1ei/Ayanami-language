use super::*;

impl HirNode for SAsm {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirAsm {
            template: self.template.clone(),
            outputs: self.outputs.iter().map(|(c, e)| (c.clone(), e.lower_to_mir(moved))).collect(),
            inputs: self.inputs.iter().map(|(c, e)| (c.clone(), e.lower_to_mir(moved))).collect(),
            ty: self.ty.clone(),
        }.into()
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
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        for (_, e) in &self.outputs { f(&**e); }
        for (_, e) in &self.inputs { f(&**e); }
    }
}

impl HirNode for SVCall {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirVirtualCall {
            receiver: self.receiver.lower_to_mir(moved),
            interface: self.interface,
            method_index: self.method_index,
            args: self.args.iter().map(|a| a.lower_to_mir(moved)).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}VirtualCall iface={} method={} ty={}", "", self.interface, self.method_index, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  receiver:", "", width = level * 2)?;
        self.receiver.display(level + 1, w)?;
        for arg in &self.args { arg.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.receiver);
        for a in &self.args { f(&**a); }
    }
}

impl HirNode for SMFP {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirMakeFatPtr {
            value: self.value.lower_to_mir(moved),
            concrete_type: self.concrete_type,
            interface_name: self.interface_name,
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}MakeFatPtr {} -> {} ty={}", "", self.concrete_type, self.interface_name, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.value.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.value);
    }
}

impl HirNode for SFnPtr {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox {
        SMirFnPtr { fn_id: self.fn_id, ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}FnPtr(fn{})", "", self.fn_id.0, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, _f: &mut dyn FnMut(&dyn HirNode)) {
    }
}

impl HirNode for SCallP {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirCallPtr {
            fn_ptr: self.fn_ptr.lower_to_mir(moved),
            args: self.args.iter().map(|a| a.lower_to_mir(moved)).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}CallPtr", "", width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.fn_ptr);
        for a in &self.args { f(&**a); }
    }
}

impl HirNode for SEnumC {
    fn enum_variant(&self) -> Option<Symbol> { Some(self.variant_name) }
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirEnumConstruct {
            enum_name: self.enum_name,
            variant_name: self.variant_name,
            variant_struct: self.variant_struct,
            args: self.args.iter().map(|a| a.lower_to_mir(moved)).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}EnumConstruct {}.{}", "", self.enum_name, self.variant_name, width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        for a in &self.args { f(&**a); }
    }
}

impl HirNode for SEnumM {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirEnumMatch {
            value: self.value.lower_to_mir(moved),
            arms: self.arms.iter().map(|(tag, e)| (*tag, e.lower_to_mir(moved))).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}EnumMatch", "", width = level * 2)?;
        self.value.display(level + 1, w)?;
        for (_, e) in &self.arms { e.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.value);
        for (_, e) in &self.arms { f(&**e); }
    }
}
