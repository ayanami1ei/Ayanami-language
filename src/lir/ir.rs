use std::collections::{HashMap, HashSet, BTreeMap};
use std::fmt::Write;

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirLocal;
use crate::parser::ast::{BinaryOp, UnaryOp};

/// Dynamic IR node — for user-defined extensions.
/// NOT an enum variant — attached to parent structs via `custom: Vec<IrNode>`.
#[derive(Debug, Clone)]
pub struct IrNode {
    pub kind: String,
    pub fields: BTreeMap<String, IrValue>,
}

#[derive(Debug, Clone)]
pub enum IrValue {
    Node(Box<IrNode>),
    Nodes(Vec<IrNode>),
    U64(u64),
    I64(i64),
    String(String),
    None,
}

impl IrNode {
    pub fn new(kind: &str) -> Self {
        Self { kind: kind.to_string(), fields: BTreeMap::new() }
    }
    pub fn set(&mut self, name: &str, val: IrValue) -> &mut Self {
        self.fields.insert(name.to_string(), val);
        self
    }
    pub fn get(&self, name: &str) -> Option<&IrValue> {
        self.fields.get(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvKind {
    ToUnique,
    ToShared,
    ToWeak,
}

#[derive(Debug, Clone)]
pub enum LirValue {
    Var(VarId),
    Tmp(u64),
    Param(u64),
    Literal(HirLiteral, HirType),
}

// ═══════════════════════════════════════════════════════════════════
//  LirEmitCtx — context provided to each instruction's emit()
//  Carries program info, temp counter, and helper methods.
// ═══════════════════════════════════════════════════════════════════

pub struct LirEmitCtx<'a> {
    pub prog: &'a LirProgram,
    pub load_tmp: u64,
    pub current_fn_ret_ty: HirType,
}

impl LirEmitCtx<'_> {
    pub fn llvm_type(&self, ty: &HirType) -> String {
        match ty {
            HirType::Int => "i64".into(),
            HirType::Float => "double".into(),
            HirType::Char => "i8".into(),
            HirType::Bool => "i1".into(),
            HirType::Void => "void".into(),
            HirType::Named(s) => {
                if self.prog.struct_defs.contains_key(s) {
                    format!("%struct.{}", sanitize_name(&s.as_str()))
                } else {
                    "i8*".into()
                }
            }
            HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
                if matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. }) {
                    "ptr".into()
                } else {
                    self.llvm_type(inner)
                }
            }
            HirType::FatPtr { .. } => "{ ptr, ptr }".into(),
            HirType::Array(_) | HirType::ArraySized(_, _) => "ptr".into(),
            HirType::FnPtr(..) => "ptr".into(),
            HirType::Ref(_, _) => "ptr".into(),
        }
    }

    pub fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String {
        match val {
            LirValue::Tmp(t) => format!("%t{}", t),
            LirValue::Param(i) => format!("%{}", i),
            LirValue::Var(v) => format!("%v{}", v.0),
            LirValue::Literal(lit, _) => lit_to_string(lit, expected_ty),
        }
    }

    pub fn tmp(&mut self) -> u64 {
        let t = self.load_tmp;
        self.load_tmp += 1;
        t
    }

    pub fn struct_llvm_name(&self, name: &Symbol) -> Option<String> {
        if self.prog.struct_defs.contains_key(name) {
            return Some(format!("%struct.{}", sanitize_name(&name.as_str())));
        }
        let s = name.as_str();
        let base = s.find('<').or_else(|| s.find('[')).map(|p| &s[..p]);
        if let Some(base) = base {
            let base_sym = Symbol::intern(base);
            if self.prog.struct_defs.contains_key(&base_sym) {
                return Some(format!("%struct.{}", sanitize_name(base)));
            }
        }
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
//  LirNode trait
// ═══════════════════════════════════════════════════════════════════

pub trait LirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn LirNode>;
    fn kind(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>;
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result;
    fn serialize(&self, buf: &mut Vec<u8>);
}

// ═══════════════════════════════════════════════════════════════════
//  LirNodeBox — cloneable wrapper
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct LirNodeBox(pub Box<dyn LirNode>);

impl Clone for LirNodeBox {
    fn clone(&self) -> Self {
        LirNodeBox(self.0.clone_node())
    }
}

impl std::ops::Deref for LirNodeBox {
    type Target = dyn LirNode;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl From<SLirCustom> for LirNodeBox {
    fn from(v: SLirCustom) -> Self { LirNodeBox(Box::new(v)) }
}

// ═══════════════════════════════════════════════════════════════════
//  s_lir! macro
// ═══════════════════════════════════════════════════════════════════

macro_rules! s_lir {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
}

// ═══════════════════════════════════════════════════════════════════
//  28 struct types for each LirInst variant
// ═══════════════════════════════════════════════════════════════════

s_lir!(SLirAlloca { var: VarId, ty: HirType });
s_lir!(SLirStore { dest: VarId, src: LirValue, ty: HirType });
s_lir!(SLirLoad { dest: u64, src: VarId, ty: HirType });
s_lir!(SLirBinOp { dest: u64, op: BinaryOp, lhs: LirValue, rhs: LirValue, ty: HirType, result_ty: HirType });
s_lir!(SLirUnaryOp { dest: u64, op: UnaryOp, src: LirValue, ty: HirType });
s_lir!(SLirCall { dest: Option<u64>, fn_id: FnId, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirCallPtr { dest: u64, fn_ptr: LirValue, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirFnAddr { dest: u64, fn_id: FnId });
s_lir!(SLirStrGlobal { dest: u64, str_idx: u64 });
s_lir!(SLirConv { dest: u64, alloca_tmp: u64, malloc_tmp: u64, src: LirValue, kind: ConvKind, src_ty: HirType, ty: HirType });
s_lir!(SLirDropValue { var: VarId, ty: HirType });
s_lir!(SLirRetainValue { var: VarId, ty: HirType });
s_lir!(SLirReleaseValue { var: VarId, ty: HirType });
s_lir!(SLirBr { label: String });
s_lir!(SLirBrCond { cond: LirValue, true_block: String, false_block: String });
s_lir!(SLirRet { val: Option<(LirValue, HirType)> });
s_lir!(SLirMakeFatPtr { dest: u64, malloc_tmp: u64, bc_tmp: u64, vtable_gep_tmp: u64, iv_tmp: u64, value_src: LirValue, value_ty: HirType, vtable_name: String, ty: HirType });
s_lir!(SLirFieldAccess { dest: u64, gep_tmp: u64, src: LirValue, field_index: usize, field_ty: HirType, struct_ty: HirType });
s_lir!(SLirAsm { dest: Option<u64>, template: String, output_constraints: Vec<String>, input_operands: Vec<(LirValue, HirType)>, input_constraints: Vec<String>, ret_ty: HirType });
s_lir!(SLirRefInst { dest: u64, var_id: VarId, mutable: bool, ty: HirType });
s_lir!(SLirArraySized { dest: u64, malloc_tmp: u64, count_tmp: u64, size_tmp: u64, elem_count: LirValue, elem_size: u64, elem_ty: HirType, ty: HirType });
s_lir!(SLirArrayLit { dest: u64, malloc_tmp: u64, elem_geps: Vec<u64>, elems: Vec<(LirValue, HirType)>, elem_ty: HirType, ty: HirType });
s_lir!(SLirIndexAccess { dest: u64, gep_tmp: u64, load_tmp: u64, arr: LirValue, index: LirValue, elem_ty: HirType, ty: HirType });
s_lir!(SLirStructLit { dest: u64, alloca_tmp: u64, field_geps: Vec<u64>, fields: Vec<(LirValue, HirType)>, struct_name: Symbol, struct_ty: HirType });
s_lir!(SLirVirtualCall { fn_dest: Option<u64>, receiver_tmp: u64, data_tmp: u64, vtable_tmp: u64, gep_tmp: u64, fn_ptr_tmp: u64, method_index: usize, args: Vec<(LirValue, HirType)>, ret_ty: HirType });
s_lir!(SLirFieldStore { dest: u64, var_id: Option<VarId>, gep_tmp: u64, iv_tmp: u64, src: LirValue, field_index: usize, field_ty: HirType, struct_ty: HirType });
s_lir!(SLirIndexStore { dest: u64, gep_tmp: u64, src: LirValue, index: LirValue, elem_ty: HirType, array_ty: HirType });
s_lir!(SLirCustom { node: IrNode });

// ═══════════════════════════════════════════════════════════════════
//  impl_into_lir_node_box! macro
// ═══════════════════════════════════════════════════════════════════

macro_rules! impl_into_lir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for LirNodeBox {
            fn from(v: $ty) -> Self { LirNodeBox(Box::new(v) as Box<dyn LirNode>) }
        })*
    };
}

impl_into_lir_node_box!(
    SLirAlloca, SLirStore, SLirLoad, SLirBinOp, SLirUnaryOp,
    SLirCall, SLirCallPtr, SLirFnAddr, SLirStrGlobal, SLirConv,
    SLirDropValue, SLirRetainValue, SLirReleaseValue,
    SLirBr, SLirBrCond, SLirRet,
    SLirMakeFatPtr, SLirFieldAccess, SLirAsm, SLirRefInst,
    SLirArraySized, SLirArrayLit, SLirIndexAccess, SLirStructLit,
    SLirVirtualCall, SLirFieldStore, SLirIndexStore,
);

// ═══════════════════════════════════════════════════════════════════
//  Helper functions (moved from emit.rs)
// ═══════════════════════════════════════════════════════════════════

pub fn sanitize_name(name: &str) -> String {
    name.replace('<', "_lt_").replace('>', "_gt_")
        .replace(',', "_c_").replace('[', "_lb_").replace(']', "_rb_").replace(' ', "_")
}

pub(crate) fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Int(0), ty) if is_pointer_type(ty) && !matches!(ty, HirType::Named(_)) => "null".into(),
        (HirLiteral::Int(0), HirType::Named(_)) => "zeroinitializer".into(),
        (HirLiteral::Int(n), _) => format!("{}", n),
        (HirLiteral::Float(n), _) => {
            let s = format!("{}", n);
            if !s.contains('.') { format!("{}.0", s) } else { s }
        }
        (HirLiteral::Char(c), _) => format!("{}", *c as u8),
        (HirLiteral::Bool(b), _) => if *b { "1".into() } else { "0".into() },
        (HirLiteral::String(_), _) => "null".into(),
    }
}

pub(crate) fn llvm_type_size(ty: &HirType) -> &'static str {
    match ty {
        HirType::Int => "8",
        HirType::Float => "8",
        HirType::Char => "1",
        HirType::Bool => "1",
        HirType::Void => "0",
        HirType::Named(_) | HirType::FatPtr { .. } => "16",
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => llvm_type_size(inner),
        HirType::Array(_) | HirType::ArraySized(_, _) => "16",
        HirType::FnPtr(..) => "8",
        HirType::Ref(_, _) => "16",
    }
}

fn needs_heap_ops(ty: &HirType) -> bool {
    match ty {
        HirType::FatPtr { .. } => true,
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
            matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. })
        }
        HirType::Named(_) => false,
        HirType::Array(_) | HirType::ArraySized(_, _) => false,
        HirType::Ref(_, _) => false,
        _ => false,
    }
}

fn is_pointer_type(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_)
        | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_)
        | HirType::Ref(_, _) | HirType::FnPtr(..)
    )
}

// ═══════════════════════════════════════════════════════════════════
//  impl LirNode for all 28 struct types
// ═══════════════════════════════════════════════════════════════════

impl LirNode for SLirAlloca {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Alloca" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let llvm_ty = ctx.llvm_type(&self.ty);
        vec![format!("%v{} = alloca {}, align 8", self.var.0, llvm_ty)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    alloca v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(0);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirStore {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Store" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let llvm_ty = ctx.llvm_type(&self.ty);
        let src_str = match &self.src {
            LirValue::Param(i) => format!("%{}", i),
            LirValue::Tmp(t) => format!("%t{}", t),
            LirValue::Var(v) => {
                let tmp = ctx.tmp();
                lines.push(format!("%e{} = load {}, ptr %v{}, align 8", tmp, llvm_ty, v.0));
                format!("%e{}", tmp)
            }
            LirValue::Literal(lit, _) => lit_to_string(lit, &self.ty),
        };
        lines.push(format!("store {} {}, ptr %v{}, align 8", llvm_ty, src_str, self.dest.0));
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    store {:?} -> v{} : {:?}", self.src, self.dest.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(1);
        put_u32(buf, self.dest.0 as u32);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirLoad {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Load" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
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

impl LirNode for SLirBinOp {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "BinOp" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let l = ctx.value_ref(&self.lhs, &self.ty);
        let r = ctx.value_ref(&self.rhs, &self.ty);
        let is_ptr = l == "null" || r == "null" || l.contains("ptr") || r.contains("ptr");
        let icmp_llvm = |ty: &HirType| -> &str {
            match ty {
                HirType::Char => "i8",
                HirType::Bool => "i1",
                HirType::Int => "i64",
                _ => { if is_ptr { "ptr" } else { "i64" } }
            }
        };
        match (&self.op, &self.ty) {
            (BinaryOp::Add, HirType::Int) => vec![format!("%t{} = add i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Int) => vec![format!("%t{} = sub i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Mul, HirType::Int) => vec![format!("%t{} = mul i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Div, HirType::Int) => vec![format!("%t{} = sdiv i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Mod, HirType::Int) => vec![format!("%t{} = srem i64 {}, {}", self.dest, l, r)],
            (BinaryOp::Add, HirType::Float) => vec![format!("%t{} = fadd double {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Float) => vec![format!("%t{} = fsub double {}, {}", self.dest, l, r)],
            (BinaryOp::Mul, HirType::Float) => vec![format!("%t{} = fmul double {}, {}", self.dest, l, r)],
            (BinaryOp::Div, HirType::Float) => vec![format!("%t{} = fdiv double {}, {}", self.dest, l, r)],
            (BinaryOp::Mod, HirType::Float) => vec![format!("%t{} = frem double {}, {}", self.dest, l, r)],
            (BinaryOp::Add, HirType::Char) => vec![format!("%t{} = add i8 {}, {}", self.dest, l, r)],
            (BinaryOp::Sub, HirType::Char) => vec![format!("%t{} = sub i8 {}, {}", self.dest, l, r)],
            (BinaryOp::Eq, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp eq ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp eq {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Neq, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ne ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp ne {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Lt, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ult ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp slt {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Gt, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ugt ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp sgt {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Le, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp ule ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp sle {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Ge, _) if self.ty != HirType::Float => {
                let llvm_int = icmp_llvm(&self.ty);
                if is_ptr { vec![format!("%t{} = icmp uge ptr {}, {}", self.dest, l, r)] }
                else { vec![format!("%t{} = icmp sge {} {}, {}", self.dest, llvm_int, l, r)] }
            }
            (BinaryOp::Eq, HirType::Float) => vec![format!("%t{} = fcmp oeq double {}, {}", self.dest, l, r)],
            (BinaryOp::Neq, HirType::Float) => vec![format!("%t{} = fcmp one double {}, {}", self.dest, l, r)],
            (BinaryOp::Lt, HirType::Float) => vec![format!("%t{} = fcmp olt double {}, {}", self.dest, l, r)],
            (BinaryOp::Gt, HirType::Float) => vec![format!("%t{} = fcmp ogt double {}, {}", self.dest, l, r)],
            (BinaryOp::Le, HirType::Float) => vec![format!("%t{} = fcmp ole double {}, {}", self.dest, l, r)],
            (BinaryOp::Ge, HirType::Float) => vec![format!("%t{} = fcmp oge double {}, {}", self.dest, l, r)],
            (BinaryOp::And, _) => vec![format!("%t{} = and i1 {}, {}", self.dest, l, r)],
            (BinaryOp::Or, _) => vec![format!("%t{} = or i1 {}, {}", self.dest, l, r)],
            _ => vec![],
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = {:?} {:?} {:?} : {:?}", self.dest, self.op, self.lhs, self.rhs, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(3);
        put_u64(buf, self.dest);
        put_u32(buf, self.op as u32);
        put_value(buf, &self.lhs);
        put_value(buf, &self.rhs);
        put_type(buf, &self.ty);
        put_type(buf, &self.result_ty);
    }
}

impl LirNode for SLirUnaryOp {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "UnaryOp" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let s = ctx.value_ref(&self.src, &self.ty);
        match (&self.op, &self.ty) {
            (UnaryOp::Neg, HirType::Int) => vec![format!("%t{} = sub i64 0, {}", self.dest, s)],
            (UnaryOp::Neg, HirType::Float) => vec![format!("%t{} = fsub double -0.0, {}", self.dest, s)],
            (UnaryOp::Not, _) => vec![format!("%t{} = xor i1 1, {}", self.dest, s)],
            _ => vec![],
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = {:?} {:?} : {:?}", self.dest, self.op, self.src, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(4);
        put_u64(buf, self.dest);
        put_u32(buf, self.op as u32);
        put_value(buf, &self.src);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirCall {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Call" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_name = &ctx.prog.fn_names[&self.fn_id];
        let mut arg_strs = Vec::new();
        for (val, aty) in &self.args {
            let llvm_ty = ctx.llvm_type(aty);
            let val_str = ctx.value_ref(val, aty);
            arg_strs.push(format!("{} {}", llvm_ty, val_str));
        }
        let is_void = matches!(&self.ret_ty, HirType::Void);
        let dest_str = match (&self.dest, is_void) {
            (Some(d), false) => format!("%t{} = ", d),
            _ => String::new(),
        };
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        vec![format!("{}call {} @{}({})", dest_str, ret_llvm, fn_name, arg_strs.join(", "))]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let dest_str = self.dest.map(|d| format!("t{}", d)).unwrap_or("_".into());
        let args_str: Vec<String> = self.args.iter().map(|(v, t)| format!("{:?}:{:?}", v, t)).collect();
        writeln!(f, "    {} = call fn{} ({}) : {:?}", dest_str, self.fn_id.0, args_str.join(", "), self.ret_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(5);
        put_u32(buf, self.dest.map_or(0xFFFFFFFF, |d| d as u32));
        put_u32(buf, self.fn_id.0 as u32);
        put_u32(buf, self.args.len() as u32);
        for (v, t) in &self.args { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.ret_ty);
    }
}

impl LirNode for SLirCallPtr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "CallPtr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_src = ctx.value_ref(&self.fn_ptr, &HirType::Int);
        let ret_llvm = ctx.llvm_type(&self.ret_ty);
        let call_args: Vec<String> = self.args.iter().map(|(v, t)| format!("{} {}", ctx.llvm_type(t), ctx.value_ref(v, t))).collect();
        let is_void = matches!(&self.ret_ty, HirType::Void);
        if is_void {
            vec![format!("call void {} ({})", fn_src, call_args.join(", "))]
        } else {
            vec![format!("%t{} = call {} {} ({})", self.dest, ret_llvm, fn_src, call_args.join(", "))]
        }
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|(v, t)| format!("{:?}:{:?}", v, t)).collect();
        writeln!(f, "    t{} = callptr {:?} ({}) : {:?}", self.dest, self.fn_ptr, args_str.join(", "), self.ret_ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(25);
        put_u64(buf, self.dest);
        put_value(buf, &self.fn_ptr);
        put_u32(buf, self.args.len() as u32);
        for (v, t) in &self.args { put_value(buf, v); put_type(buf, t); }
        put_type(buf, &self.ret_ty);
    }
}

impl LirNode for SLirFnAddr {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "FnAddr" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let fn_name = &ctx.prog.fn_names[&self.fn_id];
        vec![format!("%t{} = getelementptr i8, ptr @{}, i32 0", self.dest, fn_name)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = fnaddr fn{}", self.dest, self.fn_id.0)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(26);
        put_u64(buf, self.dest);
        put_u32(buf, self.fn_id.0 as u32);
    }
}

impl LirNode for SLirStrGlobal {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "StrGlobal" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, _ctx: &mut LirEmitCtx) -> Vec<String> {
        vec![format!("%t{} = getelementptr inbounds i8, ptr @__str_{}, i64 0", self.dest, self.str_idx)]
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    t{} = str_global @__str_{}", self.dest, self.str_idx)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(6);
        put_u64(buf, self.dest);
        put_u64(buf, self.str_idx);
    }
}

impl LirNode for SLirConv {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Conv" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        let src_val = ctx.value_ref(&self.src, &self.src_ty);
        let src_is_heap_ptr = matches!(&self.src_ty,
            HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)
        );
        match &self.kind {
            ConvKind::ToWeak => {
                if src_is_heap_ptr {
                    lines.push(format!("%t{} = bitcast ptr {} to ptr", self.dest, src_val));
                } else {
                    lines.push(format!("%t{} = bitcast ptr {} to ptr", self.dest, src_val));
                }
            }
            ConvKind::ToShared => {
                if src_is_heap_ptr {
                    lines.push(format!("%t{} = bitcast ptr {} to ptr", self.dest, src_val));
                    if matches!(&self.src_ty, HirType::Shared(_)) {
                        lines.push(format!("call void @__ayanami_shared_retain(i8* %t{})", self.dest));
                    }
                } else {
                    let inner_ty = match &self.ty {
                        HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                        _ => &self.ty,
                    };
                    let size = llvm_type_size(inner_ty);
                    let src_ptr = if matches!(&self.src_ty, HirType::Named(s) if ctx.prog.struct_defs.contains_key(s)) {
                        let src_llvm = ctx.llvm_type(&self.src_ty);
                        lines.push(format!("%t{} = alloca {}, align 8", self.alloca_tmp, src_llvm));
                        lines.push(format!("store {} {}, ptr %t{}", src_llvm, src_val, self.alloca_tmp));
                        format!("%t{}", self.alloca_tmp)
                    } else {
                        src_val.clone()
                    };
                    let alloc_fn = if needs_heap_ops(&self.ty) { "__ayanami_shared_alloc" } else { "malloc" };
                    lines.push(format!("%l{} = call i8* @{}(i64 {})", self.malloc_tmp, alloc_fn, size));
                    lines.push(format!("call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr {}, i64 {}, i1 false)", self.malloc_tmp, src_ptr, size));
                    lines.push(format!("%t{} = bitcast i8* %l{} to {}", self.dest, self.malloc_tmp, ctx.llvm_type(&self.ty)));
                }
            }
            ConvKind::ToUnique => {
                let inner_ty = match &self.ty {
                    HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                    _ => &self.ty,
                };
                let size = llvm_type_size(inner_ty);
                let src_ptr = if matches!(&self.src_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool) {
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
                let alloc_fn = if needs_heap_ops(&self.ty) { "__ayanami_shared_alloc" } else { "malloc" };
                lines.push(format!("%l{} = call i8* @{}(i64 {})", self.malloc_tmp, alloc_fn, size));
                lines.push(format!("call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr {}, i64 {}, i1 false)", self.malloc_tmp, src_ptr, size));
                lines.push(format!("%t{} = bitcast i8* %l{} to {}", self.dest, self.malloc_tmp, ctx.llvm_type(&self.ty)));
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

impl LirNode for SLirDropValue {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "DropValue" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        if needs_heap_ops(&self.ty) {
            let tmp = ctx.tmp();
            let llvm_ty = ctx.llvm_type(&self.ty);
            lines.push(format!("%c{} = load {}, ptr %v{}, align 8", tmp, llvm_ty, self.var.0));
            lines.push(format!("call void @__ayanami_shared_release(i8* %c{})", tmp));
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    drop v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(8);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirRetainValue {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "RetainValue" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        if needs_heap_ops(&self.ty) {
            let tmp = ctx.tmp();
            let llvm_ty = ctx.llvm_type(&self.ty);
            lines.push(format!("%c{} = load {}, ptr %v{}, align 8", tmp, llvm_ty, self.var.0));
            lines.push(format!("call void @__ayanami_shared_retain(i8* %c{})", tmp));
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    retain v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(9);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
    }
}

impl LirNode for SLirReleaseValue {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "ReleaseValue" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        let mut lines = Vec::new();
        if needs_heap_ops(&self.ty) {
            let tmp = ctx.tmp();
            let llvm_ty = ctx.llvm_type(&self.ty);
            lines.push(format!("%c{} = load {}, ptr %v{}, align 8", tmp, llvm_ty, self.var.0));
            lines.push(format!("call void @__ayanami_shared_release(i8* %c{})", tmp));
        }
        lines
    }
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result {
        writeln!(f, "    release v{} : {:?}", self.var.0, self.ty)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(10);
        put_u32(buf, self.var.0 as u32);
        put_type(buf, &self.ty);
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

impl LirNode for SLirRet {
    fn clone_node(&self) -> Box<dyn LirNode> { Box::new(self.clone()) }
    fn kind(&self) -> &'static str { "Ret" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String> {
        match &self.val {
            Some((v, ty)) => {
                let s = ctx.value_ref(v, ty);
                let llvm_ty = ctx.llvm_type(&ctx.current_fn_ret_ty);
                vec![format!("ret {} {}", llvm_ty, s)]
            }
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
        let is_ptr_type = matches!(&self.value_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_));
        let data_ptr = if is_ptr_type {
            ctx.value_ref(&self.value_src, &self.value_ty)
        } else {
            let size = llvm_type_size(&self.value_ty);
            lines.push(format!("%t{} = call i8* @__ayanami_shared_alloc(i64 {})", self.malloc_tmp, size));
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
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
            other => other,
        };
        let struct_name = match inner {
            HirType::Named(n) => n,
            _ => unreachable!(),
        };
        let struct_llvm = ctx.struct_llvm_name(struct_name)
            .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
        let src_str = ctx.value_ref(&self.src, &self.struct_ty);
        if matches!(&self.struct_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)) {
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
        let alloc_fn = if needs_heap_ops(&self.ty) { "__ayanami_shared_alloc" } else { "malloc" };
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
        let alloc_fn = if needs_heap_ops(&self.ty) { "__ayanami_shared_alloc" } else { "malloc" };
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
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
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
        if matches!(&self.struct_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)) {
            lines.push(format!("%t{} = getelementptr {}, ptr %t{}, i32 0, i32 {}", self.gep_tmp, struct_llvm, self.dest, self.field_index));
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

#[derive(Debug, Clone)]
pub struct LirFn {
    pub fn_id: FnId,
    pub name: Symbol,
    pub is_inline: bool,
    pub extern_c: bool,
    pub params: Vec<(Symbol, HirType)>,
    pub return_type: HirType,
    pub locals: Vec<MirLocal>,
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
    pub imported_fn_ids: HashSet<FnId>,
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
        HirType::Shared(inner) => { buf.push(7); put_type(buf, inner); }
        HirType::Weak(inner) => { buf.push(8); put_type(buf, inner); }
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
