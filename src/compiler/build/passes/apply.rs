//! A5d-3c-2：把插件编辑应用到 MIR body（preorder 下标对齐）。
//!
//! 编辑种类：
//! 1/2/3/4 = int/float/bool/char 字面量替换；
//! 5 = 克隆替换（`edit_i64` 为源表达式节点下标，类型必须一致）；
//! 6 = 删除语句（替换为空块）。

use crate::error::{Error, Result};
use crate::hir::ty::HirType;
use crate::mir::ir::{
    HirLiteral, MirFn, MirNode, MirNodeBox, MirStmtBox, MirStmtNode, SMirBlockStmt, SMirLiteral,
};
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

/// 不可变 preorder 收集表达式节点克隆（按扁平视图下标对齐）
fn collect_expr(e: &dyn MirNode, out: &mut Vec<Option<MirNodeBox>>, idx: &mut usize) {
    let i = *idx;
    *idx += 1;
    out[i] = Some(MirNodeBox(e.clone_node()));
    e.for_each_child(&mut |c| collect_expr(c, out, idx));
}

fn collect_stmt(s: &dyn MirStmtNode, out: &mut Vec<Option<MirNodeBox>>, idx: &mut usize) {
    *idx += 1;
    s.for_each_child_stmt(&mut |c| collect_stmt(c, out, idx));
    s.for_each_child_expr(&mut |e| collect_expr(e, out, idx));
}

fn collect_clones(f: &MirFn, n: usize) -> Vec<Option<MirNodeBox>> {
    let mut out = vec![None; n];
    let mut idx = 1usize; // 根节点
    for s in &f.body {
        collect_stmt(&**s, &mut out, &mut idx);
    }
    out
}

fn apply_expr(
    e: &mut MirNodeBox,
    view: &FlatView,
    edits: &EditView,
    clones: &[Option<MirNodeBox>],
    sizes: &[usize],
    idx: &mut usize,
    a: &Attr,
) -> Result<()> {
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
        if edit_kind == 5 {
            let j = v.max(0) as usize;
            if j >= clones.len() || view.arrays[0].get(j).copied().unwrap_or(0) != KIND_EXPR {
                return Err(Error::Compile(format!(
                    "pass #[{}]: replace_with source {} is not an expression (at {}:{})",
                    a.path_str(), j, a.span.start_line, a.span.start_col
                )));
            }
            let src = clones[j].as_ref().ok_or_else(|| Error::Compile(format!(
                "pass #[{}]: replace_with source {} is unavailable (at {}:{})",
                a.path_str(), j, a.span.start_line, a.span.start_col
            )))?;
            let ty = e.expr_type();
            if src.expr_type() != ty {
                return Err(Error::Compile(format!(
                    "pass #[{}]: replace_with type mismatch (`{}` vs `{}` at {}:{})",
                    a.path_str(),
                    crate::hir::display::display_type(&src.expr_type()),
                    crate::hir::display::display_type(&ty),
                    a.span.start_line, a.span.start_col
                )));
            }
            *e = src.clone();
            *idx = i + sizes[i];
            return Ok(());
        }
        if edit_kind == 6 {
            return Err(Error::Compile(format!(
                "pass #[{}]: delete_stmt is only allowed on statements (node {} at {}:{})",
                a.path_str(), i, a.span.start_line, a.span.start_col
            )));
        }
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
            if let Err(x) = apply_expr(c, view, edits, clones, sizes, idx, a) { err = Some(x); }
        }
    });
    match err { Some(x) => Err(x), None => Ok(()) }
}

fn apply_stmt(
    s: &mut MirStmtBox,
    view: &FlatView,
    edits: &EditView,
    clones: &[Option<MirNodeBox>],
    sizes: &[usize],
    idx: &mut usize,
    a: &Attr,
) -> Result<()> {
    let i = *idx;
    *idx += 1;
    let edit_kind = edits.kind.get(i).copied().unwrap_or(0);
    if edit_kind != 0 {
        if edit_kind != 6 {
            return Err(Error::Compile(format!(
                "pass #[{}]: statements only support delete_stmt (node {} at {}:{})",
                a.path_str(), i, a.span.start_line, a.span.start_col
            )));
        }
        *s = SMirBlockStmt { stmts: Vec::new(), span: crate::span::Span::default() }.into();
        *idx = i + sizes[i];
        return Ok(());
    }
    let mut err: Option<Error> = None;
    s.for_each_child_stmt_mut(&mut |c| {
        if err.is_none() {
            if let Err(x) = apply_stmt(c, view, edits, clones, sizes, idx, a) { err = Some(x); }
        }
    });
    if err.is_none() {
        s.for_each_child_expr_mut(&mut |e| {
            if err.is_none() {
                if let Err(x) = apply_expr(e, view, edits, clones, sizes, idx, a) { err = Some(x); }
            }
        });
    }
    match err { Some(x) => Err(x), None => Ok(()) }
}

pub(super) fn apply_edits(f: &mut MirFn, view: &FlatView, edits: &EditView, a: &Attr) -> Result<()> {
    if edits.kind.iter().all(|k| *k == 0) {
        return Ok(());
    }
    let clones = collect_clones(f, view.arrays[0].len());
    let sizes = compute_sizes(&view.arrays[6]);
    let mut idx = 1usize; // 根节点
    for s in f.body.iter_mut() {
        apply_stmt(s, view, edits, &clones, &sizes, &mut idx, a)?;
    }
    Ok(())
}
