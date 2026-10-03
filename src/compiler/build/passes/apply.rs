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
    if edits.kind.get(i).copied().unwrap_or(0) != 0 {
        if view.arrays[0][i] != KIND_EXPR {
            return Err(Error::Compile(format!(
                "pass #[{}]: edit target {} is not an expression (at {}:{})",
                a.path_str(), i, a.span.start_line, a.span.start_col
            )));
        }
        let ty = e.expr_type();
        if ty != HirType::Int {
            return Err(Error::Compile(format!(
                "pass #[{}]: only int literal replacement is supported (target type `{}` at {}:{})",
                a.path_str(), crate::hir::display::display_type(&ty), a.span.start_line, a.span.start_col
            )));
        }
        *e = SMirLiteral {
            val: HirLiteral::Int(edits.value.get(i).copied().unwrap_or(0)),
            ty: HirType::Int,
        }.into();
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
