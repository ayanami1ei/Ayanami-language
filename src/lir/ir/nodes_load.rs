//! `SLirLoad` 发射（从 nodes_a.rs 拆出，保持文件 ≤300 行）。
use super::*;

impl LirNode for SLirLoad {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Load" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        // #207：void 局部读取 → 占位常量（值不会被真正使用）
        if matches!(self.ty, HirType::Void) {
            return vec![format!("%t{} = add i64 0, 0", self.dest)];
        }
        let llvm_ty = ctx.llvm_type(&self.ty);
        vec![format!("%t{} = load {}, ptr %v{}, align 8", self.dest, llvm_ty, self.src.0)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = load v{} : {:?}", self.dest, self.src.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(2);
        put_u64(buf, self.dest);
        put_u32(buf, self.src.0 as u32);
        put_type(buf, &self.ty);
    }
}
