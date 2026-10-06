use super::*;

impl LirNode for SLirConv {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Conv" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let src_val = ctx.value_ref(&self.src, &self.src_ty);
        let _src_is_heap_ptr = matches!(&self.src_ty,
            HirType::Unique(_)
        );
        match &self.kind {
            ConvKind::ToUnique => {
                // 数组缓冲区已是堆指针：所有权直接转移，无需复制
                if matches!(&self.src_ty, HirType::Array(_) | HirType::ArraySized(_, _)) {
                    lines.push(format!("%t{} = bitcast ptr {} to ptr", self.dest, src_val));
                    return lines;
                }
                let inner_ty = match &self.ty {
                    HirType::Unique(i) => i.as_ref(),
                    _ => &self.ty,
                };
                let size = struct_llvm_size(inner_ty, &ctx.prog.struct_defs);
                let src_ptr = if matches!(&self.src_ty, HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. }) {
                    let src_llvm = ctx.llvm_type(&self.src_ty);
                    let alloca = format!("%t{}", self.alloca_tmp);
                    lines.push(format!("{} = alloca {}, align 8", alloca, src_llvm));
                    lines.push(format!("store {} {}, ptr {}", src_llvm, src_val, alloca));
                    alloca
                } else if matches!(&self.src_ty, HirType::Named(s) if ctx.prog.struct_defs.contains_key(s)) {
                    let src_llvm = ctx.llvm_type(&self.src_ty);
                    let alloca = format!("%t{}", self.alloca_tmp);
                    lines.push(format!("{} = alloca {}, align 8", alloca, src_llvm));
                    lines.push(format!("store {} {}, ptr {}", src_llvm, src_val, alloca));
                    alloca
                } else {
                    src_val.clone()
                };
                lines.push(format!("%l{} = call i8* @__ayanami_unique_alloc(i64 {})", self.malloc_tmp, size));
                lines.push(format!("call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr {}, i64 {}, i1 false)", self.malloc_tmp, src_ptr, size));
                lines.push(format!("%t{} = bitcast i8* %l{} to {}", self.dest, self.malloc_tmp, ctx.llvm_type(&self.ty)));
            }
            ConvKind::Cast => {
                let from = ctx.llvm_type(&self.src_ty);
                let to = ctx.llvm_type(&self.ty);
                match (int_info(&self.src_ty), int_info(&self.ty)) {
                    (Some((sb, _)), Some((db, _))) if sb == db => {
                        lines.push(format!("%t{} = bitcast {} {} to {}", self.dest, from, src_val, to));
                    }
                    (Some((sb, ss)), Some((db, _))) if sb < db => {
                        let op = if ss { "sext" } else { "zext" };
                        lines.push(format!("%t{} = {} {} {} to {}", self.dest, op, from, src_val, to));
                    }
                    (Some(_), Some(_)) => {
                        lines.push(format!("%t{} = trunc {} {} to {}", self.dest, from, src_val, to));
                    }
                    (Some((_, ss)), None) if matches!(self.ty, HirType::Float | HirType::F32) => {
                        let op = if ss { "sitofp" } else { "uitofp" };
                        lines.push(format!("%t{} = {} {} {} to {}", self.dest, op, from, src_val, to));
                    }
                    (None, Some((_, ds))) if matches!(self.src_ty, HirType::Float | HirType::F32) => {
                        // Rust 语义：浮点 → 整数为饱和转换
                        let op = if ds { "fptosi.sat" } else { "fptoui.sat" };
                        lines.push(format!("%t{} = call {} @llvm.{}.{}.{}({} {})", self.dest, to, op, to, from, from, src_val));
                    }
                    (None, None) => {
                        // float ↔ f32
                        if from == to {
                            lines.push(format!("%t{} = bitcast {} {} to {}", self.dest, from, src_val, to));
                        } else {
                            let op = if to == "float" { "fptrunc" } else { "fpext" };
                            lines.push(format!("%t{} = {} {} {} to {}", self.dest, op, from, src_val, to));
                        }
                    }
                    _ => {
                        lines.push(format!("%t{} = bitcast {} {} to {}", self.dest, from, src_val, to));
                    }
                }
            }
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = conv {:?} -> {:?} : {:?}", self.dest, self.kind, self.src, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(7);
        put_u64(buf, self.dest); put_u64(buf, self.alloca_tmp); put_u64(buf, self.malloc_tmp);
        put_value(buf, &self.src); put_u32(buf, self.kind as u32); put_type(buf, &self.src_ty); put_type(buf, &self.ty);
    }
}

impl LirNode for SLirBr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Br" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> {
        vec![format!("br label %{}", self.label)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    br %{}", self.label)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(11);
        put_str(buf, &self.label);
    }
}

impl LirNode for SLirBrCond {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "BrCond" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let c = ctx.value_ref(&self.cond, &HirType::Bool);
        vec![format!("br i1 {}, label %{}, label %{}", c, self.true_block, self.false_block)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    br_cond {:?} %{} %{}", self.cond, self.true_block, self.false_block)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(12);
        put_value(buf, &self.cond);
        put_str(buf, &self.true_block);
        put_str(buf, &self.false_block);
    }
}

impl LirNode for SLirAssume {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Assume" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let c = ctx.value_ref(&self.cond, &HirType::Bool);
        vec![format!("call void @llvm.assume(i1 {})", c)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    assume {:?}", self.cond)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(28);
        put_value(buf, &self.cond);
    }
}

impl LirNode for SLirContractCheck {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "ContractCheck" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let c = ctx.value_ref(&self.cond, &HirType::Bool);
        let t = ctx.tmp();
        vec![
            format!("br i1 {}, label %contract_ok_{}, label %contract_fail_{}", c, t, t),
            format!("contract_fail_{}:", t),
            format!("call void @{}(i64 {}, i64 {})", self.kind.runtime_fn(), self.line, self.col),
            "unreachable".to_string(),
            format!("contract_ok_{}:", t),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    {}({}:{}) {:?}", self.kind.label(), self.line, self.col, self.cond)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(29);
        buf.push(match self.kind {
            crate::hir::ContractKind::Require => 0,
            crate::hir::ContractKind::Ensure => 1,
            crate::hir::ContractKind::Invariant => 2,
        });
        put_value(buf, &self.cond);
        put_u64(buf, self.line);
        put_u64(buf, self.col);
    }
}

impl LirNode for SLirRet {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Ret" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        match &self.val {
            // M1.9：`return <!>`（发散表达式）→ 后续不可达
            Some((_, HirType::Never)) => vec!["unreachable".into()],
            Some((v, ty)) => {
                let s = ctx.value_ref(v, ty);
                let llvm_ty = ctx.llvm_type(&ctx.current_fn_ret_ty);
                vec![format!("ret {} {}", llvm_ty, s)]
            }
            // `-> !` 函数末尾：无 ret，仅 unreachable
            None if matches!(ctx.current_fn_ret_ty, HirType::Never) => vec!["unreachable".into()],
            None => vec!["ret void".into()],
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        match &self.val {
            Some((v, _)) => writeln!(f, "    ret {:?}", v),
            None => writeln!(f, "    ret void"),
        }
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(13);
        match &self.val {
            Some((v, t)) => { buf.push(1); put_value(buf, v); put_type(buf, t); }
            None => { buf.push(0); }
        }
    }
}

impl LirNode for SLirMakeFatPtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "MakeFatPtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let kind = match &self.ty {
            HirType::FatPtr { kind, .. } => kind.as_ref(),
            _ => &HirType::Void,
        };
        let is_ptr_type = matches!(&self.value_ty, HirType::Unique(_) | HirType::Ref(..));
        let data_ptr = if is_ptr_type || matches!(kind, HirType::Ref(..)) {
            // 已是指针或借用胖指针：直接使用（ref 的 LIR 值就是指向值的地址）
            ctx.value_ref(&self.value_src, &self.value_ty)
        } else {
            let alloc_fn = "__ayanami_unique_alloc";
            let size = struct_llvm_size(&self.value_ty, &ctx.prog.struct_defs);
            lines.push(format!("%t{} = call i8* @{}(i64 {})", self.malloc_tmp, alloc_fn, size));
            lines.push(format!("%t{} = bitcast i8* %t{} to ptr", self.bc_tmp, self.malloc_tmp));
            let val_llvm = ctx.llvm_type(&self.value_ty);
            let src_str = ctx.value_ref(&self.value_src, &self.value_ty);
            lines.push(format!("store {} {}, ptr %t{}, align 8", val_llvm, src_str, self.bc_tmp));
            format!("%t{}", self.malloc_tmp)
        };
        let vtable_elem_count = ctx.prog.vtables.iter()
            .find(|v| v.name == self.vtable_name)
            .map(|v| v.fn_ids.len())
            .unwrap_or(1);
        lines.push(format!("%t{} = getelementptr [{} x ptr], ptr @{}, i64 0, i64 0", self.vtable_gep_tmp, vtable_elem_count, self.vtable_name));
        lines.push(format!("%t{} = insertvalue {{ ptr, ptr }} zeroinitializer, ptr {}, 0", self.iv_tmp, data_ptr));
        lines.push(format!("%t{} = insertvalue {{ ptr, ptr }} %t{}, ptr %t{}, 1", self.dest, self.iv_tmp, self.vtable_gep_tmp));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = make_fatptr vtable={}", self.dest, self.vtable_name)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(14);
        put_u64(buf, self.dest); put_u64(buf, self.malloc_tmp); put_u64(buf, self.bc_tmp);
        put_u64(buf, self.vtable_gep_tmp); put_u64(buf, self.iv_tmp);
        put_value(buf, &self.value_src); put_type(buf, &self.value_ty); put_str(buf, &self.vtable_name); put_type(buf, &self.ty);
    }
}

impl LirNode for SLirFieldAccess {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FieldAccess" }
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
        let src_str = ctx.value_ref(&self.src, &self.struct_ty);
        if matches!(&self.struct_ty, HirType::Unique(_) | HirType::Ref(..)) {
            vec![
                format!("%t{} = getelementptr {}, ptr {}, i32 0, i32 {}", self.gep_tmp, struct_llvm, src_str, self.field_index),
                format!("%t{} = load {}, ptr %t{}", self.dest, ctx.llvm_type(&self.field_ty), self.gep_tmp),
            ]
        } else {
            vec![format!("%t{} = extractvalue {} {}, {}", self.dest, struct_llvm, src_str, self.field_index)]
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = field_access field={} : {:?}", self.dest, self.field_index, self.field_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(16);
        put_u64(buf, self.dest); put_u64(buf, self.gep_tmp);
        put_value(buf, &self.src); put_u32(buf, self.field_index as u32);
        put_type(buf, &self.field_ty); put_type(buf, &self.struct_ty);
    }
}
