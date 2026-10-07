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
        emit_drop_value(ctx, &slot, &self.ty, &mut lines);
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
