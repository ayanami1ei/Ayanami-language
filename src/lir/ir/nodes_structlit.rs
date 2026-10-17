//! `SLirStructLit` 发射；从 nodes_c.rs 拆出，保持文件 ≤300 行。
use super::*;

impl LirNode for SLirStructLit {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "StructLit" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn alloca_lines(&self, _ctx: &LirEmitCtx) -> Vec<String> {
        let struct_llvm = format!("%struct.{}", sanitize_name(&self.struct_name.as_str()));
        vec![format!("%t{} = alloca {}, align 8", self.alloca_tmp, struct_llvm)]
    }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let struct_llvm = format!("%struct.{}", sanitize_name(&self.struct_name.as_str()));
        // P3b：union —— 先清零存储（其余字段别名读到确定值）
        let is_union = ctx.prog.struct_defs.get(&self.struct_name).and_then(|f| f.first())
            .map(|(n, _)| n.as_str() == "__union").unwrap_or(false);
        if is_union {
            lines.push(format!("store {} zeroinitializer, ptr %t{}", struct_llvm, self.alloca_tmp));
        }
        for (i, ((val, fty), gep_tmp)) in self.fields.iter().zip(self.field_geps.iter()).enumerate() {
            let field_llvm = ctx.llvm_type(fty);
            // Var 源是栈槽地址，需要 load 出值再存
            let val_str = match val {
                LirValue::Var(v) => {
                    let t = ctx.tmp();
                    lines.push(format!("%e{} = load {}, ptr %v{}, align 8", t, field_llvm, v.0));
                    format!("%e{}", t)
                }
                _ => ctx.value_ref(val, fty),
            };
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
