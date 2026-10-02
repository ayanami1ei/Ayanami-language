use super::*;

impl LirNode for SLirAsm {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Asm" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        let constraint_str = {
            let mut all = self.output_constraints.clone();
            all.extend(self.input_constraints.iter().cloned());
            all.join(",")
        };
        let args_str: Vec<String> = self.input_operands.iter()
            .map(|(v, t)| format!("{} {}", ctx.llvm_type(t), ctx.value_ref(v, t)))
            .collect();
        if let Some(d) = self.dest {
            vec![format!("%t{} = call {} asm sideeffect \"{}\", \"{}\"({})", d, ret_llvm, self.template, constraint_str, args_str.join(", "))]
        } else {
            let args = if args_str.is_empty() { String::from("()") } else { format!("({})", args_str.join(", ")) };
            vec![format!("call void asm sideeffect \"{}\", \"{}\"{}", self.template, constraint_str, args)]
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    asm \"{}\"", self.template)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(24);
        put_u32(buf, self.dest.map_or(0xFFFFFFFF, |d| d as u32));
        put_str(buf, &self.template);
        put_u32(buf, self.output_constraints.len() as u32);
        for c in &self.output_constraints { put_str(buf, c); }
        put_u32(buf, self.input_operands.len() as u32);
        for (v, t) in &self.input_operands { put_value(buf, v); put_type(buf, t); }
        put_u32(buf, self.input_constraints.len() as u32);
        for c in &self.input_constraints { put_str(buf, c); }
        put_type(buf, &self.ret_ty);
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

impl LirNode for SLirArraySized {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "ArraySized" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let count_str = ctx.value_ref(&self.elem_count, &HirType::Int);
        let alloc_fn = "__ayanami_unique_alloc";
        vec![
            format!("%t{} = add i64 0, {}", self.count_tmp, count_str),
            format!("%t{} = mul i64 %t{}, {}", self.size_tmp, self.count_tmp, self.elem_size),
            format!("%t{} = call i8* @{}(i64 %t{})", self.malloc_tmp, alloc_fn, self.size_tmp),
            format!("%t{} = bitcast i8* %t{} to ptr", self.dest, self.malloc_tmp),
            format!("call void @llvm.memset.p0.i64(ptr %t{}, i8 0, i64 %t{}, i1 false)", self.dest, self.size_tmp),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = array_sized (count={:?}, elem_ty={:?})", self.dest, self.elem_count, self.elem_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(20);
        put_u64(buf, self.dest); put_u64(buf, self.malloc_tmp);
        put_u64(buf, self.count_tmp); put_u64(buf, self.size_tmp);
        put_value(buf, &self.elem_count); put_u64(buf, self.elem_size);
        put_type(buf, &self.elem_ty); put_type(buf, &self.ty);
    }
}

impl LirNode for SLirArrayLit {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "ArrayLit" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let num_elems = self.elems.len();
        let elem_llvm = ctx.llvm_type(&self.elem_ty);
        let elem_size_val = llvm_type_size(&self.elem_ty).parse::<u64>().unwrap_or(8);
        let total_size = num_elems as u64 * elem_size_val;
        let alloc_fn = "__ayanami_unique_alloc";
        lines.push(format!("%t{} = call i8* @{}(i64 {})", self.malloc_tmp, alloc_fn, total_size));
        lines.push(format!("%t{} = bitcast i8* %t{} to ptr", self.dest, self.malloc_tmp));
        for (i, ((val, _fty), gep_tmp)) in self.elems.iter().zip(self.elem_geps.iter()).enumerate() {
            let val_str = ctx.value_ref(val, &self.elem_ty);
            lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i64 {}", gep_tmp, elem_llvm, self.dest, i));
            lines.push(format!("store {} {}, ptr %t{}", elem_llvm, val_str, gep_tmp));
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = array_lit ({} elems, elem_ty={:?})", self.dest, self.elems.len(), self.elem_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(18);
        put_u64(buf, self.dest); put_u64(buf, self.malloc_tmp);
        put_u32(buf, self.elem_geps.len() as u32);
        for g in &self.elem_geps { put_u64(buf, *g); }
        put_u32(buf, self.elems.len() as u32);
        for (v, t) in &self.elems { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.elem_ty); put_type(buf, &self.ty);
    }
}

impl LirNode for SLirIndexAccess {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "IndexAccess" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let elem_llvm = ctx.llvm_type(&self.elem_ty);
        let arr_str = ctx.value_ref(&self.arr, &self.ty);
        let idx_str = ctx.value_ref(&self.index, &HirType::Int);
        vec![
            format!("%t{} = getelementptr {}, ptr {}, i64 {}", self.gep_tmp, elem_llvm, arr_str, idx_str),
            format!("%t{} = load {}, ptr %t{}", self.load_tmp, elem_llvm, self.gep_tmp),
            format!("%t{} = bitcast {} %t{} to {}", self.dest, elem_llvm, self.load_tmp, ctx.llvm_type(&self.ty)),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = index_access elem_ty={:?}", self.dest, self.elem_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(19);
        put_u64(buf, self.dest); put_u64(buf, self.gep_tmp); put_u64(buf, self.load_tmp);
        put_value(buf, &self.arr); put_value(buf, &self.index);
        put_type(buf, &self.elem_ty); put_type(buf, &self.ty);
    }
}

impl LirNode for SLirStructLit {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "StructLit" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let struct_llvm = format!("%struct.{}", sanitize_name(&self.struct_name.as_str()));
        lines.push(format!("%t{} = alloca {}, align 8", self.alloca_tmp, struct_llvm));
        for (i, ((val, fty), gep_tmp)) in self.fields.iter().zip(self.field_geps.iter()).enumerate() {
            let val_str = ctx.value_ref(val, fty);
            let field_llvm = ctx.llvm_type(fty);
            lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i32 0, i32 {}", gep_tmp, struct_llvm, self.alloca_tmp, i));
            lines.push(format!("store {} {}, ptr %t{}", field_llvm, val_str, gep_tmp));
        }
        lines.push(format!("%t{} = load {}, ptr %t{}", self.dest, struct_llvm, self.alloca_tmp));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = struct_lit {} ({} fields)", self.dest, self.struct_name, self.fields.len())
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(17);
        put_u64(buf, self.dest); put_u64(buf, self.alloca_tmp);
        put_u32(buf, self.field_geps.len() as u32);
        for g in &self.field_geps { put_u64(buf, *g); }
        put_u32(buf, self.fields.len() as u32);
        for (v, t) in &self.fields { put_value(buf, v); put_type(buf, t); }
        put_str(buf, &self.struct_name.as_str()); put_type(buf, &self.struct_ty);
    }
}

impl LirNode for SLirVirtualCall {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "VirtualCall" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("%t{} = extractvalue {{ ptr, ptr }} %t{}, 0", self.data_tmp, self.receiver_tmp));
        lines.push(format!("%t{} = extractvalue {{ ptr, ptr }} %t{}, 1", self.vtable_tmp, self.receiver_tmp));
        let slot_idx = 1 + self.method_index;
        lines.push(format!("%t{} = getelementptr ptr, ptr %t{}, i32 {}", self.gep_tmp, self.vtable_tmp, slot_idx));
        lines.push(format!("%t{} = load ptr, ptr %t{}", self.fn_ptr_tmp, self.gep_tmp));
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        let mut call_args = vec![format!("ptr %t{}", self.data_tmp)];
        for (val, aty) in &self.args {
            let llvm_ty = ctx.llvm_type(aty);
            let val_str = ctx.value_ref(val, aty);
            call_args.push(format!("{} {}", llvm_ty, val_str));
        }
        let dest_str = match (&self.fn_dest, &self.ret_ty) {
            (Some(_), HirType::Void) | (None, _) => String::new(),
            (Some(d), _) => format!("%t{} = ", d),
        };
        lines.push(format!("{}call {} %t{}({})", dest_str, ret_llvm, self.fn_ptr_tmp, call_args.join(", ")));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let d = self.fn_dest.map(|d| format!("t{}", d)).unwrap_or("_".into());
        writeln!(f, "    {} = virtual_call [receiver=t{}, slot={}]", d, self.receiver_tmp, 1 + self.method_index)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(15);
        put_u32(buf, self.fn_dest.map_or(0xFFFFFFFF, |d| d as u32));
        put_u64(buf, self.receiver_tmp); put_u64(buf, self.data_tmp); put_u64(buf, self.vtable_tmp);
        put_u64(buf, self.gep_tmp); put_u64(buf, self.fn_ptr_tmp);
        put_u32(buf, self.method_index as u32);
        put_u32(buf, self.args.len() as u32);
        for (v, t) in &self.args { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.ret_ty);
    }
}

impl LirNode for SLirFieldStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FieldStore" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let inner = match &self.struct_ty {
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) | HirType::Ref(inner, _) => inner.as_ref(),
            other => other,
        };
        let struct_name = match inner {
            HirType::Named(n) => n,
            _ => unreachable!(),
        };
        let struct_llvm = ctx.struct_llvm_name(struct_name)
            .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
        let src_str = ctx.value_ref(&self.src, &self.field_ty);
        let field_llvm = ctx.llvm_type(&self.field_ty);
        if matches!(&self.struct_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_) | HirType::Ref(..)) {
            lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i32 0, i32 {}", self.gep_tmp, struct_llvm, self.dest, self.field_index));
            // Retain the new value before storing (shared pointer field)
            if matches!(&self.field_ty, HirType::Shared(_)) {
                lines.push(format!("call void @__ayanami_shared_retain(i8* {})", src_str));
            }
            lines.push(format!("store {} {}, ptr %t{}", field_llvm, src_str, self.gep_tmp));
        } else {
            let var_ty = ctx.llvm_type(&self.struct_ty);
            lines.push(format!("%t{} = insertvalue {} %t{}, {} {}, {}", self.iv_tmp, struct_llvm, self.dest, field_llvm, src_str, self.field_index));
            let store_var = self.var_id.expect("FieldStore: value type needs var_id");
            lines.push(format!("store {} %t{}, ptr %v{}, align 8", var_ty, self.iv_tmp, store_var.0));
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = field_store field={} : {:?}", self.dest, self.field_index, self.field_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(21);
        put_u64(buf, self.dest);
        put_u32(buf, self.var_id.map_or(0xFFFFFFFF, |v| v.0 as u32));
        put_u64(buf, self.gep_tmp); put_u64(buf, self.iv_tmp);
        put_value(buf, &self.src); put_u32(buf, self.field_index as u32);
        put_type(buf, &self.field_ty); put_type(buf, &self.struct_ty);
    }
}

impl LirNode for SLirIndexStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "IndexStore" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let elem_llvm = ctx.llvm_type(&self.elem_ty);
        let src_str = ctx.value_ref(&self.src, &self.elem_ty);
        let idx_str = ctx.value_ref(&self.index, &HirType::Int);
        vec![
            format!("%t{} = getelementptr {}, ptr %t{}, i64 {}", self.gep_tmp, elem_llvm, self.dest, idx_str),
            format!("store {} {}, ptr %t{}", elem_llvm, src_str, self.gep_tmp),
        ]
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
