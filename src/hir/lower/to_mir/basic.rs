use super::*;

impl HirNode for SVar {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirLocal { var: self.var, ty: self.ty.clone(), moved: moved.contains(&self.var) }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Local(v{} : {})", "", self.var.0, crate::hir::display::display_type(&self.ty), width = level * 2)
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn as_local(&self) -> Option<VarId> { Some(self.var) }
    fn collect_var_ids(&self, vars: &mut HashSet<VarId>) { vars.insert(self.var); }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        
    }
}

impl HirNode for SConst {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, _moved: &HashSet<VarId>) -> MirNodeBox {
        SMirLiteral { val: self.val.clone(), ty: self.ty.clone() }.into()
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
    fn as_const(&self) -> Option<&HirLiteral> { Some(&self.val) }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        
    }
}

impl HirNode for SBin {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirBinary {
            op: self.op,
            lhs: self.lhs.lower_to_mir(moved),
            rhs: self.rhs.lower_to_mir(moved),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Binary {{ op: {:?}, ty: {} }}", "", self.op, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        writeln!(w, "{:width$}  lhs:", "", width = level * 2)?;
        self.lhs.display(level + 1, w)?;
        writeln!(w, "{:width$}  rhs:", "", width = level * 2)?;
        self.rhs.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.lhs);
        f(&*self.rhs);
    }
}

impl HirNode for SUn {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirUnary { op: self.op, arg: self.arg.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Unary {{ op: {:?}, ty: {} }}", "", self.op, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.arg.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.arg);
    }
}

impl HirNode for SCall {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirCall {
            fn_id: self.fn_id,
            args: self.args.iter().map(|a| a.lower_to_mir(moved)).collect(),
            ty: self.ty.clone(),
        }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Call(fn{}, ty: {})", "", self.fn_id.0, crate::hir::display::display_type(&self.ty), width = level * 2)?;
        for arg in &self.args { arg.display(level + 1, w)?; }
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        for a in &self.args { f(&**a); }
    }
}

impl HirNode for SMove {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirMove { expr: self.expr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Move(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn is_move_or_clone(&self) -> bool { true }
    fn as_move(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        // Copy 类型不因转换/移动而失效
        if !self.expr.expr_type().is_copy() {
            if let Some(id) = self.expr.as_local() { moved.insert(id); }
        }
        self.expr.record_moves(moved);
    }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.expr);
    }
}

impl HirNode for SClone {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirClone { expr: self.expr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}Clone(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn is_move_or_clone(&self) -> bool { true }
    fn as_clone(&self) -> Option<&HirNodeBox> { Some(&self.expr) }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.expr);
    }
}

impl HirNode for SToUnique {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirToUnique { expr: self.expr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToUnique(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
    fn for_each_child(&self, f: &mut dyn FnMut(&dyn HirNode)) {
        f(&*self.expr);
    }
    fn record_moves(&self, moved: &mut HashSet<VarId>) {
        // Copy 类型不因转换/移动而失效
        if !self.expr.expr_type().is_copy() {
            if let Some(id) = self.expr.as_local() { moved.insert(id); }
        }
        self.expr.record_moves(moved);
    }
}


