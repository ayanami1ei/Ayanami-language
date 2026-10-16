//! P3：volatile 读/写发射（LLVM `load volatile` / `store volatile`，MMIO 语义）。

use super::*;

impl LirNode for SLirVolatileLoad {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "VolatileLoad" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        let ptr_ty = HirType::RawPtr(Box::new(self.ty.clone()));
        let src = ctx.value_ref(&self.src, &ptr_ty);
        vec![format!("%t{} = load volatile {}, ptr {}", self.dest, llvm_ty, src)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = volatile_load : {:?}", self.dest, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(41);
        put_u64(buf, self.dest);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirVolatileStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "VolatileStore" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        let ptr_ty = HirType::RawPtr(Box::new(self.ty.clone()));
        let ptr = ctx.value_ref(&self.ptr, &ptr_ty);
        let src = ctx.value_ref(&self.src, &self.ty);
        vec![format!("store volatile {} {}, ptr {}", llvm_ty, src, ptr)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    volatile_store : {:?}", self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(42);
        put_value(buf, &self.ptr);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}
