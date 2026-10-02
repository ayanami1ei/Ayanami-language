use super::*;

impl LirNode for SLirCustom {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Custom" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> {
        eprintln!("unhandled custom LIR instruction");
        vec![]
    }
    fn display(&self, _f: &mut dyn Write) -> std::fmt::Result { Ok(()) }
    fn serialize(&self, _buf: &mut Vec<u8>) {}
}

// ═══════════════════════════════════════════════════════════════════
//  LirBlock, LirFn, VtableDesc, LirProgram
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct LirBlock {
    pub label: String,
    pub insts: Vec<LirNodeBox>,
}

/// 标注（名称 + 参数），LIR 层保留用于 LLVM 属性映射。
#[derive(Debug, Clone, PartialEq)]
pub struct LirAttr {
    pub name: String,
    pub args: Vec<String>,
}

/// 外部函数声明（含真实签名与标注，用于 `declare` 发射）。
#[derive(Debug, Clone)]
pub struct ExternDecl {
    pub name: String,
    pub params: Vec<HirType>,
    pub return_type: HirType,
    pub attrs: Vec<LirAttr>,
}

#[derive(Debug, Clone)]
pub struct LirFn {
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
    pub attrs: Vec<LirAttr>,
    pub blocks: Vec<LirBlock>,
    /// User-defined extension nodes (not in LirInst enum)
    pub custom: Vec<IrNode>,
}

/// Describes a vtable global constant for interface dispatch
#[derive(Debug, Clone)]
pub struct VtableDesc {
    pub name: String,
    pub fn_ids: Vec<FnId>,
}

#[derive(Debug, Clone)]
pub struct LirProgram {
    pub strings: Vec<String>,
    pub fn_names: HashMap<FnId, String>,
    pub functions: Vec<LirFn>,
    pub vtables: Vec<VtableDesc>,
    pub struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
    pub generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,
    pub extern_decls: Vec<ExternDecl>,
}

// ═══════════════════════════════════════════════════════════════════
//  Serialization helper functions (used by trait serialize() methods)
// ═══════════════════════════════════════════════════════════════════

pub(crate) fn put_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
pub(crate) fn put_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
pub(crate) fn put_str(buf: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    put_u32(buf, b.len() as u32);
    buf.extend_from_slice(b);
}

pub(crate) fn put_type(buf: &mut Vec<u8>, ty: &HirType) {
    match ty {
        HirType::Int => buf.push(0),
        HirType::Float => buf.push(1),
        HirType::Char => buf.push(2),
        HirType::Void => buf.push(3),
        HirType::Bool => buf.push(4),
        HirType::Named(s) => { buf.push(5); put_str(buf, &s.as_str()); }
        HirType::Unique(inner) => { buf.push(6); put_type(buf, inner); }
        HirType::FatPtr { name, kind } => {
            buf.push(9);
            put_str(buf, &name.as_str());
            put_type(buf, kind);
        }
        HirType::Array(inner) | HirType::ArraySized(inner, _) => { buf.push(10); put_type(buf, inner); }
        HirType::FnPtr(params, ret) => {
            buf.push(12);
            put_u32(buf, params.len() as u32);
            for p in params.iter() { put_type(buf, p); }
            put_type(buf, ret);
        }
        HirType::Ref(inner, mutable) => { buf.push(11); put_type(buf, inner); buf.push(if *mutable { 1 } else { 0 }); }
    }
}

pub(crate) fn put_value(buf: &mut Vec<u8>, v: &LirValue) {
    match v {
        LirValue::Var(vid) => { buf.push(0); put_u32(buf, vid.0 as u32); }
        LirValue::Tmp(t) => { buf.push(1); put_u64(buf, *t); }
        LirValue::Param(i) => { buf.push(2); put_u64(buf, *i); }
        LirValue::Literal(lit, ty) => {
            buf.push(3);
            put_literal(buf, lit);
            put_type(buf, ty);
        }
    }
}

pub(crate) fn put_literal(buf: &mut Vec<u8>, lit: &HirLiteral) {
    match lit {
        HirLiteral::Int(n) => { buf.push(0); put_u64(buf, *n as u64); }
        HirLiteral::Float(n) => { buf.push(1); buf.extend_from_slice(&n.to_le_bytes()); }
        HirLiteral::Char(c) => { buf.push(2); put_u32(buf, *c as u32); }
        HirLiteral::String(s) => { buf.push(3); put_str(buf, s); }
        HirLiteral::Bool(b) => { buf.push(4); buf.push(if *b { 1 } else { 0 }); }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  LirLowerCtx — trait for the lowering context
// ═══════════════════════════════════════════════════════════════════

/// Trait interface for LIR lowering context — allows SMir* nodes to lower themselves.
pub trait LirLowerCtx {
    fn next_tmp(&mut self) -> u64;
    fn emit(&mut self, inst: LirNodeBox);
    fn str_map(&self) -> &HashMap<String, u64>;
    fn loop_stack(&self) -> &Vec<(String, String)>;
    fn loop_stack_mut(&mut self) -> &mut Vec<(String, String)>;
    fn next_block_label(&mut self, prefix: &str) -> String;
    fn set_current_block(&mut self, label: String);
}
