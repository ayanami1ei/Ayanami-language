//! M2/FnOnce：字段移出节点（load 值 + 源字段清零，避免源结构 drop 时双重释放）。
use super::*;

impl LirNode for SLirFieldTake {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FieldTake" }
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
        // 可深拷贝类型：字段读取为非破坏性 clone（源字段保留，各自 drop 一次）；
        // 闭包/拥有胖指针/裸动态数组无 clone glue → 保持移出 + 源字段清零。
        if super::helpers_clone::is_cloneable(&self.field_ty, &ctx.prog.struct_defs) {
            let alloca = ctx.tmp();
            let mut lines = vec![
                format!("%t{} = getelementptr {}, ptr {}, i32 0, i32 {}", self.gep_tmp, struct_llvm, obj_str, self.field_index),
                format!("%t{} = alloca {}, align 8", alloca, field_llvm),
            ];
            super::helpers_clone::emit_clone_value(
                ctx, &format!("%t{}", alloca), &format!("%t{}", self.gep_tmp), &self.field_ty, None, &mut lines);
            lines.push(format!("%t{} = load {}, ptr %t{}", self.dest, field_llvm, alloca));
            return lines;
        }
        vec![
            format!("%t{} = getelementptr {}, ptr {}, i32 0, i32 {}", self.gep_tmp, struct_llvm, obj_str, self.field_index),
            format!("%t{} = load {}, ptr %t{}", self.dest, field_llvm, self.gep_tmp),
            format!("store {} zeroinitializer, ptr %t{}", field_llvm, self.gep_tmp),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = field_take field={} : {:?}", self.dest, self.field_index, self.field_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(35);
        put_u64(buf, self.dest);
        put_u64(buf, self.gep_tmp);
        put_value(buf, &self.obj);
        put_u32(buf, self.field_index as u32);
        put_type(buf, &self.field_ty);
        put_type(buf, &self.struct_ty);
    }
}

/// 局部变量移出：load 值 + 源变量清零（循环回边重复移动时避免双重释放）。
impl LirNode for SLirLocalTake {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "LocalTake" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        vec![
            format!("%t{} = load {}, ptr %v{}", self.dest, llvm_ty, self.var.0),
            format!("store {} zeroinitializer, ptr %v{}", llvm_ty, self.var.0),
        ]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = local_take v{} : {:?}", self.dest, self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(36);
        put_u64(buf, self.dest);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}

/// 对指针指向的具体值做递归 drop（接口箱 vtable[0] drop glue 函数体）。
impl LirNode for SLirDropPtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "DropPtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let slot = ctx.value_ref(&self.ptr, &HirType::Ref(Box::new(self.ty.clone()), true));
        let mut lines = Vec::new();
        // Named：函数体展开字段（不调用自身）；其余类型按通用规则
        if matches!(&self.ty, HirType::Named(_)) {
            super::helpers_drop::emit_drop_fields(ctx, &slot, &self.ty, &mut lines);
        } else {
            emit_drop_value(ctx, &slot, &self.ty, &mut lines);
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    drop_ptr : {:?}", self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(37);
        put_value(buf, &self.ptr);
        put_type(buf, &self.ty);
    }
}
