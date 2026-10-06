//! A5d-3：MIR 扁平视图构建、序列化与插件输出校验。

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::ir::FnId;
use crate::mir::ir::{BinaryOp, HirLiteral, MirFn, MirNode, MirStmtNode, UnaryOp};
use crate::parser::ast::Attr;

/// schema 版本（首字段；C 桥接与编译器必须一致）
pub(super) const SCHEMA_VERSION: i64 = 3;
/// 头部字段数：version / pure / no_error / node_count
pub(super) const HEADER_WORDS: usize = 4;
/// 数组个数：v0 七项 + ops/lit_kinds/lit_i64/lit_f64 + stmt_kinds/var_ids + 编辑数组
pub(super) const ARRAY_COUNT: usize = 15;
/// 只读数组个数（前 13 个；后 2 个是编辑数组）
pub(super) const READONLY_ARRAYS: usize = 13;

pub(super) const KIND_FN: i64 = 0;
pub(super) const KIND_STMT: i64 = 1;
pub(super) const KIND_EXPR: i64 = 2;

pub(super) struct FlatView {
    pub arrays: Vec<Vec<i64>>,
    purity: HashMap<FnId, bool>,
}

impl FlatView {
    pub(super) fn new(purity: HashMap<FnId, bool>) -> Self {
        Self { arrays: vec![Vec::new(); ARRAY_COUNT], purity }
    }

    pub(super) fn len(&self) -> usize {
        self.arrays[0].len()
    }

    /// 顺序：kinds,is_call,callee_pure,is_alloc,is_asm,is_store,child_counts,
    /// ops,lit_kinds,lit_i64,lit_f64,stmt_kinds,var_ids,edit_kind,edit_i64
    #[allow(clippy::too_many_arguments)]
    fn push(&mut self, kind: i64, call: bool, callee_pure: bool, alloc: bool, asm: bool, store: bool, op: i64, lit_kind: i64, lit_i64: i64, lit_f64: i64, stmt_kind: i64, var_id: i64) -> usize {
        let idx = self.len();
        let vals = [kind, call as i64, callee_pure as i64, alloc as i64, asm as i64, store as i64, 0, op, lit_kind, lit_i64, lit_f64, stmt_kind, var_id, 0, 0];
        for (a, v) in self.arrays.iter_mut().zip(vals) {
            a.push(v);
        }
        idx
    }

    fn set_child_count(&mut self, idx: usize, n: i64) {
        self.arrays[6][idx] = n;
    }
}

fn op_code(op: BinaryOp) -> i64 {
    match op {
        BinaryOp::Add => 1, BinaryOp::Sub => 2, BinaryOp::Mul => 3,
        BinaryOp::Div => 4, BinaryOp::Mod => 5, BinaryOp::Eq => 6,
        BinaryOp::Neq => 7, BinaryOp::Lt => 8, BinaryOp::Gt => 9,
        BinaryOp::Le => 10, BinaryOp::Ge => 11, BinaryOp::And => 12,
        BinaryOp::Or => 13, BinaryOp::BitAnd => 14, BinaryOp::BitOr => 15,
        BinaryOp::BitXor => 16, BinaryOp::Shl => 17, BinaryOp::Shr => 18,
    }
}

fn unary_code(op: UnaryOp) -> i64 {
    match op {
        UnaryOp::Neg => 20, UnaryOp::Not => 21, UnaryOp::BitNot => 22,
    }
}

fn lit_kind(v: &HirLiteral) -> i64 {
    match v {
        HirLiteral::Int(_) => 1,
        HirLiteral::Float(_) => 2,
        HirLiteral::Bool(_) => 3,
        HirLiteral::Char(_) => 4,
        HirLiteral::String(_) => 0,
        HirLiteral::Array(_) => 0,
    }
}

fn lit_i64(v: &HirLiteral) -> i64 {
    match v {
        HirLiteral::Array(_) => 0,
        HirLiteral::Int(n) => *n,
        HirLiteral::Bool(b) => *b as i64,
        HirLiteral::Char(c) => *c as i64,
        _ => 0,
    }
}

fn lit_f64(v: &HirLiteral) -> i64 {
    match v {
        HirLiteral::Float(f) => f.to_bits() as i64,
        _ => 0,
    }
}

fn walk_expr(e: &dyn MirNode, v: &mut FlatView) -> usize {
    let callee_pure = if e.is_call() {
        e.call_fn_id().map(|fid| v.purity.get(&fid).copied().unwrap_or(false)).unwrap_or(false)
    } else {
        false
    };
    let op = match (e.binary_op(), e.unary_op()) {
        (Some(b), _) => op_code(b),
        (_, Some(u)) => unary_code(u),
        _ => 0,
    };
    let (lk, li, lf) = match e.literal_value() {
        Some((val, _ty)) => (lit_kind(val), lit_i64(val), lit_f64(val)),
        None => (0, 0, 0),
    };
    // var_ids 采用 +1 编码（0 = 非局部变量）
    let var_id = e.as_local().map(|v| v.0 as i64 + 1).unwrap_or(0);
    let idx = v.push(KIND_EXPR, e.is_call(), callee_pure, e.is_alloc(), e.is_asm(), false, op, lk, li, lf, 0, var_id);
    let mut n = 0;
    e.for_each_child(&mut |c| { walk_expr(c, v); n += 1; });
    v.set_child_count(idx, n);
    idx
}

fn stmt_kind(s: &dyn MirStmtNode) -> i64 {
    if s.assign_parts().is_some() { 1 }
    else if s.field_assign_parts().is_some() { 2 }
    else if s.index_assign_parts().is_some() { 3 }
    else if s.deref_assign_parts().is_some() { 12 }
    else if s.is_return() { 4 }
    else if s.as_if().is_some() { 5 }
    else if s.as_while().is_some() { 6 }
    else if s.is_break() { 7 }
    else if s.is_continue() { 8 }
    else if s.expr_part().is_some() { 9 }
    else if s.as_block().is_some() { 10 }
    else if s.as_drop().is_some() { 11 }
    else { 0 }
}

fn walk_stmt(s: &dyn MirStmtNode, v: &mut FlatView) -> usize {
    let is_store = s.field_assign_parts().is_some() || s.index_assign_parts().is_some() || s.deref_assign_parts().is_some();
    let idx = v.push(KIND_STMT, false, false, false, false, is_store, 0, 0, 0, 0, stmt_kind(s), 0);
    let mut n = 0;
    s.for_each_child_stmt(&mut |c| { walk_stmt(c, v); n += 1; });
    s.for_each_child_expr(&mut |c| { walk_expr(c, v); n += 1; });
    v.set_child_count(idx, n);
    idx
}

fn push_i64(buf: &mut Vec<u8>, v: i64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// MirFn → 扁平视图 blob（schema v1）
pub(super) fn serialize_fn(f: &MirFn, purity: &HashMap<FnId, bool>) -> Vec<u8> {
    let mut v = FlatView::new(purity.clone());
    let root = v.push(KIND_FN, false, false, false, false, false, 0, 0, 0, 0, 0, 0);
    let mut n = 0;
    for s in &f.body {
        walk_stmt(&**s, &mut v);
        n += 1;
    }
    v.set_child_count(root, n);

    let mut buf = Vec::new();
    push_i64(&mut buf, SCHEMA_VERSION);
    push_i64(&mut buf, f.effects.pure as i64);
    push_i64(&mut buf, f.effects.no_error as i64);
    push_i64(&mut buf, v.len() as i64);
    for arr in &v.arrays {
        for x in arr.iter() {
            push_i64(&mut buf, *x);
        }
    }
    buf
}

pub(super) fn read_i64(buf: &[u8], off: usize) -> i64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&buf[off..off + 8]);
    i64::from_le_bytes(b)
}

pub(super) struct EditView {
    pub kind: Vec<i64>,
    pub value: Vec<i64>,
}

pub(super) struct PassOutput {
    pub pure: bool,
    pub no_error: bool,
    pub edits: EditView,
}

/// 校验插件输出：版本/长度；只读数组必须不变；返回效应与编辑
pub(super) fn parse_output(input: &[u8], out: &[u8], a: &Attr) -> Result<PassOutput> {
    let header = HEADER_WORDS * 8;
    if out.len() != input.len() || read_i64(out, 0) != SCHEMA_VERSION {
        return Err(Error::Compile(format!(
            "pass #[{}] returned a mismatched schema (expected v{}, {} bytes; got {} bytes) (at {}:{})",
            a.path_str(), SCHEMA_VERSION, input.len(), out.len(), a.span.start_line, a.span.start_col
        )));
    }
    let n = read_i64(out, 24) as usize;
    let arr_len = n * 8;
    let ro_end = header + READONLY_ARRAYS * arr_len;
    if out[header..ro_end] != input[header..ro_end] {
        return Err(Error::Compile(format!(
            "pass #[{}] modified read-only MIR fields; use the edit arrays (schema v1) (at {}:{})",
            a.path_str(), a.span.start_line, a.span.start_col
        )));
    }
    let edits_off = ro_end;
    let kind: Vec<i64> = (0..n).map(|i| read_i64(out, edits_off + i * 8)).collect();
    let value: Vec<i64> = (0..n).map(|i| read_i64(out, edits_off + arr_len + i * 8)).collect();
    Ok(PassOutput {
        pure: read_i64(out, 8) != 0,
        no_error: read_i64(out, 16) != 0,
        edits: EditView { kind, value },
    })
}

/// 从 blob 恢复视图数组（编辑应用需要 kinds/child_counts）
pub(super) fn deserialize_view(blob: &[u8]) -> FlatView {
    let n = read_i64(blob, 24) as usize;
    let header = HEADER_WORDS * 8;
    let arr_len = n * 8;
    let mut arrays = Vec::with_capacity(ARRAY_COUNT);
    for k in 0..ARRAY_COUNT {
        let off = header + k * arr_len;
        arrays.push((0..n).map(|i| read_i64(blob, off + i * 8)).collect());
    }
    FlatView { arrays, purity: HashMap::new() }
}
