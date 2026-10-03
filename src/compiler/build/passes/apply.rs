//! A5d-3：把插件编辑应用到 MIR body（preorder 下标对齐）。

use crate::error::{Error, Result};
use crate::hir::ty::HirType;
use crate::mir::ir::{HirLiteral, MirFn, MirNodeBox, MirStmtNode, SMirLiteral};
use crate::parser::ast::Attr;

use super::flat::{EditView, FlatView, KIND_EXPR};

/// preorder 子树大小（用于替换后跳过原子树）
fn compute_sizes(child_counts: &[i64]) -> Vec<usize> {
    let n = child_counts.len();
    let mut sizes = vec![1usize; n];
    for i in (0..n).rev() {
        let mut s = 1usize;
        let mut c = i + 1;
        for _ in 0..child_counts[i].max(0) {
            if c >= n { break; }
            s += sizes[c];
            c += sizes[c];
        }
        sizes[i] = s;
    }
    sizes
}

fn apply_expr(e: &mut MirNodeBox, view: &FlatView, edits: &EditView, sizes: &[usize], idx: &mut usize, a: &Attr) -> Result<()> {
    let i = *idx;
    *idx += 1;
    let edit_kind = edits.kind.get(i).copied().unwrap_or(0);
    if edit_kind != 0 {
        if view.arrays[0][i] != KIND_EXPR {
            return Err(Error::Compile(format!(
                "pass #[{}]: edit target {} is not an expression (at {}:{})",
                a.path_str(), i, a.span.start_line, a.span.start_col
            )));
        }
        let v = edits.value.get(i).copied().unwrap_or(0);
        let ty = e.expr_type();
        // 比较节点在 MIR 中保留操作数类型，bool 结果由 MIR→LIR 决定；
        // 因此允许把比较节点（ops 6..=11）替换为 bool 字面量。
        let is_cmp = matches!(view.arrays[7].get(i).copied().unwrap_or(0), 6..=11);
        let (val, lit_ty) = match edit_kind {
            1 if ty == HirType::Int => (HirLiteral::Int(v), HirType::Int),
            2 if ty == HirType::Float => (HirLiteral::Float(f64::from_bits(v as u64)), HirType::Float),
            3 if ty == HirType::Bool || is_cmp => (HirLiteral::Bool(v != 0), HirType::Bool),
            4 if ty == HirType::Char => (HirLiteral::Char(v as u8 as char), HirType::Char),
            _ => {
                return Err(Error::Compile(format!(
                    "pass #[{}]: literal edit kind {} does not match target type `{}` (at {}:{})",
                    a.path_str(), edit_kind,
                    crate::hir::display::display_type(&ty),
                    a.span.start_line, a.span.start_col
                )));
            }
        };
        *e = SMirLiteral { val, ty: lit_ty }.into();
        *idx = i + sizes[i];
        return Ok(());
    }
    let mut err: Option<Error> = None;
    e.for_each_child_mut(&mut |c| {
        if err.is_none() {
            if let Err(x) = apply_expr(c, view, edits, sizes, idx, a) { err = Some(x); }
        }
    });
    match err { Some(x) => Err(x), None => Ok(()) }
}

fn apply_stmt(s: &mut dyn MirStmtNode, view: &FlatView, edits: &EditView, sizes: &[usize], idx: &mut usize, a: &Attr) -> Result<()> {
    let i = *idx;
    *idx += 1;
    if edits.kind.get(i).copied().unwrap_or(0) != 0 {
        return Err(Error::Compile(format!(
            "pass #[{}]: edits are only supported on expressions (node {} at {}:{})",
            a.path_str(), i, a.span.start_line, a.span.start_col
        )));
    }
    let mut err: Option<Error> = None;
    s.for_each_child_stmt_mut(&mut |c| {
        if err.is_none() {
            if let Err(x) = apply_stmt(&mut **c, view, edits, sizes, idx, a) { err = Some(x); }
        }
    });
    if err.is_none() {
        s.for_each_child_expr_mut(&mut |e| {
            if err.is_none() {
                if let Err(x) = apply_expr(e, view, edits, sizes, idx, a) { err = Some(x); }
            }
        });
    }
    match err { Some(x) => Err(x), None => Ok(()) }
}

pub(super) fn apply_edits(f: &mut MirFn, view: &FlatView, edits: &EditView, a: &Attr) -> Result<()> {
    if edits.kind.iter().all(|k| *k == 0) {
        return Ok(());
    }
    let sizes = compute_sizes(&view.arrays[6]);
    let mut idx = 0usize;
    idx += 1; // 根节点
    for s in f.body.iter_mut() {
        apply_stmt(&mut **s, view, edits, &sizes, &mut idx, a)?;
    }
    Ok(())
}
