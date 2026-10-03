//! A5d-2：MIR 优化注解（#[pass]）执行器（schema v0）。
//!
//! 流程：
//! 1. 把 `MirFn` 展平为 preorder 视图（合成根节点 + kinds/flags/child_counts）；
//! 2. 通过插件 ABI 调用用户 pass（生成的 C 桥接解码为 `std/mir.aya` 的 `MirFunction`）；
//! 3. 只接受效应字段（pure/no_error）变化；body 变化在 v0 直接报错。

use std::collections::HashMap;
use std::path::Path;

use super::*;
use crate::hir::ir::{FnId, HirItem, HirProgram};
use crate::mir::ir::{MirFn, MirItem, MirNode, MirStmtNode};
use crate::parser::ast::{Attr, Program};

/// schema 版本（首字段；C 桥接与编译器必须一致）
const SCHEMA_VERSION: i64 = 1;
/// 头部字段数：version / pure / no_error / node_count
const HEADER_WORDS: usize = 4;

const KIND_FN: i64 = 0;
const KIND_STMT: i64 = 1;
const KIND_EXPR: i64 = 2;

struct FlatView {
    kinds: Vec<i64>,
    is_call: Vec<i64>,
    callee_pure: Vec<i64>,
    is_alloc: Vec<i64>,
    is_asm: Vec<i64>,
    is_store: Vec<i64>,
    child_counts: Vec<i64>,
    purity: HashMap<FnId, bool>,
}

impl FlatView {
    fn new(purity: HashMap<FnId, bool>) -> Self {
        Self {
            kinds: Vec::new(), is_call: Vec::new(), callee_pure: Vec::new(),
            is_alloc: Vec::new(), is_asm: Vec::new(), is_store: Vec::new(),
            child_counts: Vec::new(), purity,
        }
    }

    fn push(&mut self, kind: i64, call: bool, callee_pure: bool, alloc: bool, asm: bool, store: bool) -> usize {
        let idx = self.kinds.len();
        self.kinds.push(kind);
        self.is_call.push(call as i64);
        self.callee_pure.push(callee_pure as i64);
        self.is_alloc.push(alloc as i64);
        self.is_asm.push(asm as i64);
        self.is_store.push(store as i64);
        self.child_counts.push(0);
        idx
    }
}

fn walk_expr(e: &dyn MirNode, v: &mut FlatView) -> usize {
    let callee_pure = if e.is_call() {
        e.call_fn_id().map(|fid| v.purity.get(&fid).copied().unwrap_or(false)).unwrap_or(false)
    } else {
        false
    };
    let idx = v.push(KIND_EXPR, e.is_call(), callee_pure, e.is_alloc(), e.is_asm(), false);
    let mut n = 0;
    e.for_each_child(&mut |c| { walk_expr(c, v); n += 1; });
    v.child_counts[idx] = n;
    idx
}

fn walk_stmt(s: &dyn MirStmtNode, v: &mut FlatView) -> usize {
    let is_store = s.field_assign_parts().is_some() || s.index_assign_parts().is_some();
    let idx = v.push(KIND_STMT, false, false, false, false, is_store);
    let mut n = 0;
    s.for_each_child_stmt(&mut |c| { walk_stmt(c, v); n += 1; });
    s.for_each_child_expr(&mut |c| { walk_expr(c, v); n += 1; });
    v.child_counts[idx] = n;
    idx
}

fn push_i64(buf: &mut Vec<u8>, v: i64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// MirFn → 扁平视图 blob（schema v0）
fn serialize_fn(f: &MirFn, purity: &HashMap<FnId, bool>) -> Vec<u8> {
    let mut v = FlatView::new(purity.clone());
    let root = v.push(KIND_FN, false, false, false, false, false);
    let mut n = 0;
    for s in &f.body {
        walk_stmt(&**s, &mut v);
        n += 1;
    }
    v.child_counts[root] = n;

    let mut buf = Vec::new();
    push_i64(&mut buf, SCHEMA_VERSION);
    push_i64(&mut buf, f.effects.pure as i64);
    push_i64(&mut buf, f.effects.no_error as i64);
    push_i64(&mut buf, v.kinds.len() as i64);
    for arr in [&v.kinds, &v.is_call, &v.callee_pure, &v.is_alloc, &v.is_asm, &v.is_store, &v.child_counts] {
        for x in arr.iter() {
            push_i64(&mut buf, *x);
        }
    }
    buf
}

fn read_i64(buf: &[u8], off: usize) -> i64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&buf[off..off + 8]);
    i64::from_le_bytes(b)
}

/// 校验插件输出：版本/长度/body 不变；返回 (pure, no_error)
fn apply_output(input: &[u8], out: &[u8], a: &Attr) -> Result<(bool, bool)> {
    let body_off = HEADER_WORDS * 8;
    let body_len = input.len() - body_off;
    if out.len() != input.len() || read_i64(out, 0) != SCHEMA_VERSION {
        return Err(Error::Compile(format!(
            "pass #[{}] returned a mismatched schema (expected v{}, {} bytes; got {} bytes) (at {}:{})",
            a.path_str(), SCHEMA_VERSION, input.len(), out.len(), a.span.start_line, a.span.start_col
        )));
    }
    if out[body_off..] != input[body_off..body_off + body_len] {
        return Err(Error::Compile(format!(
            "pass #[{}] modified the function body; body rewriting is not supported in schema v0 (at {}:{})",
            a.path_str(), a.span.start_line, a.span.start_col
        )));
    }
    Ok((read_i64(out, 8) != 0, read_i64(out, 16) != 0))
}

/// 函数纯度表：声明 pure，或本轮推断无效应且无未知 extern
fn purity_map(hir: &HirProgram) -> HashMap<FnId, bool> {
    let mut m = HashMap::new();
    for item in &hir.items {
        if let HirItem::Fn(f) = item {
            let pure = f.effects.pure
                || (f.inferred.effects.is_empty() && !f.inferred.may_unknown_effects);
            m.insert(f.fn_id, pure);
        }
    }
    for imp in &hir.imported_fns {
        let pure = imp.effects.pure
            || (imp.inferred.effects.is_empty() && !imp.inferred.may_unknown_effects);
        m.entry(imp.fn_id).or_insert(pure);
    }
    m
}

fn is_builtin_attr(a: &Attr) -> bool {
    let name = a.name.as_str();
    (a.is_builtin() || (a.qualifier.len() == 1 && a.qualifier[0].as_str() == "core"))
        && (crate::hir::attrs::ALLOWED.contains(&name.as_str()) || crate::hir::effects::is_effect(&name))
}

/// 对 MIR 程序应用所有 `#[pass]` 注解
pub(super) fn apply_passes(
    mir: &mut crate::mir::ir::MirProgram,
    hir: &HirProgram,
    program: &Program,
    src_path: &Path,
) -> Result<()> {
    let tables = crate::compiler::macro_expand::annotation_tables(&program.stmts)?;
    if tables.is_empty() {
        return Ok(());
    }
    let purity = purity_map(hir);
    for item in &mut mir.items {
        let MirItem::Fn(f) = item else { continue };
        let attrs: Vec<Attr> = f.attrs.iter().filter(|a| !is_builtin_attr(a)).cloned().collect();
        for a in attrs {
            let resolved = tables.resolve(&a)?;
            let Some((pkg, name, kind)) = resolved else {
                return Err(Error::Compile(format!(
                    "unknown annotation #[{}] at {}:{}", a.path_str(), a.span.start_line, a.span.start_col
                )));
            };
            if kind != crate::compiler::macro_expand::AnnKind::Pass {
                continue;
            }
            let lcl = tables.lcl_path(&pkg).ok_or_else(|| Error::Compile(format!(
                "package `{}` has no annotation table", pkg
            )))?;
            let blob = serialize_fn(f, &purity);
            let out = crate::compiler::macro_expand::invoke_pass(&lcl, &name, &blob)
                .map_err(|e| Error::Compile(format!(
                    "pass `#[{}]` at {}:{} failed: {}",
                    a.path_str(), a.span.start_line, a.span.start_col, e
                )))?;
            let (pure, no_error) = apply_output(&blob, &out, &a)?;
            f.effects.pure = pure;
            f.effects.no_error = no_error;
        }
    }
    let _ = src_path;
    Ok(())
}
