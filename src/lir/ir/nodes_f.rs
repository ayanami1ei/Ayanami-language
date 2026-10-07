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
