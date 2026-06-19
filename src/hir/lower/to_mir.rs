use std::collections::{HashSet, HashMap};
use crate::intern::Symbol;
use crate::mir::ir::*;
use crate::hir::*;
use crate::parser::ast::BinaryOp;

// ═══════════════════════════════════════════════════════════════════
//  impl HirNode for all 23 HIR struct types
// ═══════════════════════════════════════════════════════════════════

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
        if let Some(id) = self.expr.as_local() { moved.insert(id); }
        self.expr.record_moves(moved);
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
}

impl HirNode for SToShared {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirToShared { expr: self.expr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToShared(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

impl HirNode for SToWeak {
    fn clone_node(&self) -> Box<dyn HirNode> { Box::new(self.clone()) }
    fn lower_to_mir(&self, moved: &HashSet<VarId>) -> MirNodeBox {
        SMirToWeak { expr: self.expr.lower_to_mir(moved), ty: self.ty.clone() }.into()
    }
    fn display(&self, level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        writeln!(w, "{:width$}ToWeak(ty: {})", "", crate::hir::display::display_type(&self.ty), width = level * 2)?;
        self.expr.display(level + 1, w)?;
        Ok(())
    }
    fn expr_type(&self) -> HirType { self.ty.clone() }
}

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
}

impl HirNode for SEnumC {
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
}
