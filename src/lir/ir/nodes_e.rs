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

impl LirNode for SLirLoadPtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "LoadPtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        let src = ctx.value_ref(&self.src, &self.ty);
        vec![format!("%t{} = load {}, ptr {}, align 8", self.dest, llvm_ty, src)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = load_ptr {:?} : {:?}", self.dest, self.src, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(34);
        put_u64(buf, self.dest);
        put_value(buf, &self.src);
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

impl LirNode for SLirIndexAddr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "IndexAddr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let elem_llvm = ctx.llvm_type(&self.elem_ty);
        let idx_str = match &self.index {
            LirValue::Var(v) => {
                let t = ctx.tmp();
                lines.push(format!("%e{} = load i64, ptr %v{}, align 8", t, v.0));
                format!("%e{}", t)
            }
            _ => ctx.value_ref(&self.index, &HirType::Int),
        };
        lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i64 {}", self.dest, elem_llvm, self.arr_tmp, idx_str));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = index_addr elem_ty={:?}", self.dest, self.elem_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(33);
        put_u64(buf, self.dest); put_u64(buf, self.arr_tmp);
        put_value(buf, &self.index); put_type(buf, &self.elem_ty);
    }
}

impl LirNode for SLirStrGlobal {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "StrGlobal" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        // 字符串字面量复制到堆上：String 值拥有其 data 缓冲区，作用域结束会释放
        let len = ctx.prog.strings.get(self.str_idx as usize).map(|s| s.len()).unwrap_or(0);
        let l = ctx.tmp();
        vec![
            // 多分配 1 字节并写入 NUL，保证可传给 C 字符串函数（strlen 等）
            format!("%l{} = call i8* @__ayanami_unique_alloc(i64 {})", l, len + 1),
            format!("call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr @__str_{}, i64 {}, i1 false)", l, self.str_idx, len),
            format!("%e{} = getelementptr i8, ptr %l{}, i64 {}", l, l, len),
            format!("store i8 0, ptr %e{}", l),
            format!("%t{} = bitcast i8* %l{} to ptr", self.dest, l),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = str_global @__str_{}", self.dest, self.str_idx)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(6);
        put_u64(buf, self.dest);
        put_u64(buf, self.str_idx);
    }
}

impl LirNode for SLirFieldStorePtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FieldStorePtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let inner = match &self.struct_ty {
            HirType::Unique(inner) | HirType::Ref(inner, _) => inner.as_ref(),
            other => other,
        };
        let struct_name = match inner {
            HirType::Named(n) => n,
            _ => unreachable!(),
        };
        let struct_llvm = ctx.struct_llvm_name(struct_name)
            .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
        let obj_str = ctx.value_ref(&self.obj, &self.struct_ty);
        let field_llvm = ctx.llvm_type(&self.field_ty);
        let mut out = Vec::new();
        let src_str = match &self.src {
            LirValue::Var(v) => {
                let t = ctx.tmp();
                out.push(format!("%e{} = load {}, ptr %v{}, align 8", t, field_llvm, v.0));
                format!("%e{}", t)
            }
            _ => ctx.value_ref(&self.src, &self.field_ty),
        };
        out.push(format!("%t{} = getelementptr {}, ptr {}, i32 0, i32 {}", self.gep_tmp, struct_llvm, obj_str, self.field_index));
        out.push(format!("store {} {}, ptr %t{}", field_llvm, src_str, self.gep_tmp));
        out
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    field_store_ptr field={}", self.field_index)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(31);
        put_u64(buf, self.gep_tmp);
        put_value(buf, &self.obj);
        put_u32(buf, self.field_index as u32);
        put_type(buf, &self.field_ty);
        put_value(buf, &self.src);
        put_type(buf, &self.struct_ty);
    }
}

impl LirNode for SLirGlobalAddr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "GlobalAddr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> {
        vec![format!("%t{} = getelementptr i8, ptr @{}, i64 0", self.dest, self.name.as_str())]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = global_addr @{}", self.dest, self.name.as_str())
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(32);
        put_u64(buf, self.dest);
        put_str(buf, &self.name.as_str());
    }
}

impl LirNode for SLirFieldAddr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FieldAddr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let inner = match &self.struct_ty {
            HirType::Unique(inner) | HirType::Ref(inner, _) => inner.as_ref(),
            other => other,
        };
        let struct_name = match inner {
            HirType::Named(n) => n,
            _ => unreachable!(),
        };
        let struct_llvm = ctx.struct_llvm_name(struct_name)
            .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
        let obj_str = ctx.value_ref(&self.obj, &self.struct_ty);
        vec![format!("%t{} = getelementptr {}, ptr {}, i32 0, i32 {}", self.dest, struct_llvm, obj_str, self.field_index)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = field_addr field={}", self.dest, self.field_index)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(30);
        put_u64(buf, self.dest);
        put_value(buf, &self.obj);
        put_u32(buf, self.field_index as u32);
        put_type(buf, &self.struct_ty);
    }
}
