//! `SLirConv`（数值/指针转换）发射；从 nodes_b.rs 拆出，保持文件 ≤300 行。

use super::*;

impl LirNode for SLirConv {
    fn alloca_lines(&self, ctx: &LirEmitCtx) -> Vec<String> {
        if !matches!(self.kind, ConvKind::ToUnique) || matches!(&self.src_ty, HirType::Array(_) | HirType::ArraySized(_, _)) { return Vec::new(); }
        let scalar = matches!(&self.src_ty,
            HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. });
        let named = matches!(&self.src_ty, HirType::Named(s) if ctx.prog.struct_defs.contains_key(s));
        // #202：闭包/胖指针同为聚合值（16 字节），需要栈槽中转
        let aggregate = matches!(&self.src_ty, HirType::Closure(..) | HirType::FatPtr { .. });
        if scalar || named || aggregate {
            let src_llvm = ctx.llvm_type(&self.src_ty);
            vec![format!("%t{} = alloca {}, align 8", self.alloca_tmp, src_llvm)]
        } else {
            Vec::new()
        }
    }
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Conv" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let src_val = ctx.value_ref(&self.src, &self.src_ty);
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
                // #158：sizeof = GEP 到元素末尾 + ptrtoint（含对齐/嵌套结构体）
                let inner_llvm = ctx.llvm_type(inner_ty); let sz_gep = ctx.tmp(); let sz_tmp = ctx.tmp();
                lines.push(format!("%z{} = getelementptr {}, ptr null, i32 1\n%z{} = ptrtoint ptr %z{} to i64", sz_gep, inner_llvm, sz_tmp, sz_gep));
                let scalar = matches!(&self.src_ty, HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. });
                let named = matches!(&self.src_ty, HirType::Named(s) if ctx.prog.struct_defs.contains_key(s));
                // #202：闭包/胖指针为 16 字节聚合值，必须走“栈槽 + memcpy”装箱路径
                let aggregate = matches!(&self.src_ty, HirType::Closure(..) | HirType::FatPtr { .. });
                let src_ptr = if scalar || named || aggregate {
                    let src_llvm = ctx.llvm_type(&self.src_ty);
                    let alloca = format!("%t{}", self.alloca_tmp);
                    lines.push(format!("store {} {}, ptr {}", src_llvm, src_val, alloca));
                    alloca
                } else { src_val.clone() };
                lines.push(format!("%l{} = call i8* @__ayanami_unique_alloc(i64 %z{})", self.malloc_tmp, sz_tmp));
                lines.push(format!("call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr {}, i64 %z{}, i1 false)", self.malloc_tmp, src_ptr, sz_tmp));
                lines.push(format!("%t{} = bitcast i8* %l{} to {}", self.dest, self.malloc_tmp, ctx.llvm_type(&self.ty)));
            }
            // P3：位级重解释（同尺寸；尺寸由前端校验，LLVM 指令按类型选择）
            ConvKind::Reinterpret => {
                let from = ctx.llvm_type(&self.src_ty);
                let to = ctx.llvm_type(&self.ty);
                let src_ptr = matches!(&self.src_ty,
                    HirType::RawPtr(_) | HirType::Ref(..) | HirType::FnPtr(..) | HirType::Unique(_));
                let dst_ptr = matches!(&self.ty,
                    HirType::RawPtr(_) | HirType::Ref(..) | HirType::FnPtr(..) | HirType::Unique(_));
                if dst_ptr && !src_ptr {
                    lines.push(format!("%t{} = inttoptr {} {} to {}", self.dest, from, src_val, to));
                } else if src_ptr && !dst_ptr {
                    lines.push(format!("%t{} = ptrtoint {} {} to {}", self.dest, from, src_val, to));
                } else {
                    lines.push(format!("%t{} = bitcast {} {} to {}", self.dest, from, src_val, to));
                }
            }
            ConvKind::Cast => {
                let from = ctx.llvm_type(&self.src_ty);
                let to = ctx.llvm_type(&self.ty);
                // M5 P2：裸指针转换（int↔ptr / ptr↔ptr）
                let src_is_ptr = matches!(&self.src_ty,
                    HirType::RawPtr(_) | HirType::Ref(..) | HirType::FnPtr(..) | HirType::Unique(_));
                let dst_is_ptr = matches!(&self.ty, HirType::RawPtr(_));
                if dst_is_ptr && !src_is_ptr {
                    lines.push(format!("%t{} = inttoptr {} {} to {}", self.dest, from, src_val, to));
                    return lines;
                }
                if src_is_ptr && !dst_is_ptr {
                    lines.push(format!("%t{} = ptrtoint {} {} to {}", self.dest, from, src_val, to));
                    return lines;
                }
                if src_is_ptr && dst_is_ptr {
                    lines.push(format!("%t{} = bitcast {} {} to {}", self.dest, from, src_val, to));
                    return lines;
                }
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

