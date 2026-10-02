use super::*;

impl LirNode for SLirDropValue {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "DropValue" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        emit_drop_value(ctx, &format!("%v{}", self.var.0), &self.ty, &mut lines);
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    drop v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(8);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}



impl LirNode for SLirRefInst {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "RefInst" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> {
        vec![format!("%t{} = getelementptr i8, ptr %v{}, i32 0", self.dest, self.var_id.0)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let m = if self.mutable { "mut " } else { "" };
        writeln!(f, "    t{} = ref_{}v{} : {:?}", self.dest, m, self.var_id.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(23);
        put_u64(buf, self.dest);
        put_u32(buf, self.var_id.0 as u32);
        buf.push(if self.mutable { 1 } else { 0 });
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirRefTmp {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "RefTmp" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        // 对临时值取引用：先溢出到栈槽，再返回其地址
        let llvm = ctx.llvm_type(&self.ty);
        let src = ctx.value_ref(&self.src, &self.ty);
        vec![
            format!("%t{} = alloca {}, align 8", self.alloca_tmp, llvm),
            format!("store {} {}, ptr %t{}", llvm, src, self.alloca_tmp),
            format!("%t{} = getelementptr i8, ptr %t{}, i32 0", self.dest, self.alloca_tmp),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let m = if self.mutable { "mut " } else { "" };
        writeln!(f, "    t{} = ref_tmp_{} : {:?}", self.dest, m, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(27);
        put_u64(buf, self.dest);
        put_u64(buf, self.alloca_tmp);
        put_value(buf, &self.src);
        buf.push(if self.mutable { 1 } else { 0 });
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirIndexStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "IndexStore" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let elem_llvm = ctx.llvm_type(&self.elem_ty);
        let src_str = match &self.src {
            LirValue::Var(v) => {
                let t = ctx.tmp();
                lines.push(format!("%e{} = load {}, ptr %v{}, align 8", t, elem_llvm, v.0));
                format!("%e{}", t)
            }
            _ => ctx.value_ref(&self.src, &self.elem_ty),
        };
        let idx_str = match &self.index {
            LirValue::Var(v) => {
                let t = ctx.tmp();
                lines.push(format!("%e{} = load i64, ptr %v{}, align 8", t, v.0));
                format!("%e{}", t)
            }
            _ => ctx.value_ref(&self.index, &HirType::Int),
        };
        lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i64 {}", self.gep_tmp, elem_llvm, self.dest, idx_str));
        lines.push(format!("store {} {}, ptr %t{}", elem_llvm, src_str, self.gep_tmp));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = index_store elem_ty={:?}", self.dest, self.elem_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(22);
        put_u64(buf, self.dest); put_u64(buf, self.gep_tmp);
        put_value(buf, &self.src); put_value(buf, &self.index);
        put_type(buf, &self.elem_ty); put_type(buf, &self.array_ty);
    }
}
