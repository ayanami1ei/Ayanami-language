use super::*;
use crate::error::{Error, Result};

/// #119：从 lambda body 的 return 值推断返回类型（递归，取第一个带值 return）。
pub(crate) fn find_return_type(stmts: &[crate::hir::HirStmt]) -> Option<HirType> {
    for s in stmts {
        match s {
            // 比较运算在 HIR 中保留操作数类型（Bool 由 MIR 决定）→ 按 Bool 推断
            crate::hir::HirStmt::Return { value: Some(v), .. } => {
                return Some(if v.is_comparison() { HirType::Bool } else { v.expr_type() });
            }
            crate::hir::HirStmt::If { then_block, elifs, else_block, .. } => {
                if let Some(t) = find_return_type(&then_block.stmts) { return Some(t); }
                for (_, b) in elifs {
                    if let Some(t) = find_return_type(&b.stmts) { return Some(t); }
                }
                if let Some(b) = else_block {
                    if let Some(t) = find_return_type(&b.stmts) { return Some(t); }
                }
            }
            crate::hir::HirStmt::While { body, .. } => {
                if let Some(t) = find_return_type(&body.stmts) { return Some(t); }
            }
            _ => {}
        }
    }
    None
}

/// #168：无标注 lambda 的返回类型推断完成后，校验此前登记的带值 `return`。
pub(crate) fn check_pending_returns(
    name: Symbol,
    inferred_ret: &HirType,
    checks: &[(Span, HirType)],
) -> Result<()> {
    let mut first_void_reported = false;
    for (span, src_ty) in checks {
        let src_is_void = matches!(super::strip_ownership_ref(src_ty), HirType::Void);
        if matches!(inferred_ret, HirType::Void) {
            if first_void_reported {
                return Err(Error::Hir(format!(
                    "function `{}` returns no value, but `return` has a value of type `{}` (at {}:{})",
                    name, hir_type_display(src_ty), span.start_line, span.start_col
                )));
            }
            first_void_reported = true;
        } else if src_is_void {
            return Err(Error::Hir(format!(
                "function `{}` returns `{}`, but `return` has no value (`{}` call) (at {}:{})",
                name, hir_type_display(inferred_ret), hir_type_display(src_ty),
                span.start_line, span.start_col
            )));
        }
    }
    Ok(())
}

/// 推断出非 void 返回类型后，禁止同一 lambda 中出现裸 `return;`。
pub(crate) fn check_bare_returns(
    name: Symbol,
    inferred_ret: &HirType,
    stmts: &[crate::hir::HirStmt],
) -> Result<()> {
    if matches!(inferred_ret, HirType::Void) {
        return Ok(());
    }
    fn find(stmts: &[crate::hir::HirStmt]) -> Option<Span> {
        for stmt in stmts {
            match stmt {
                crate::hir::HirStmt::Return { value: None, span } => return Some(*span),
                crate::hir::HirStmt::Block { stmts, .. } => {
                    if let Some(span) = find(stmts) { return Some(span); }
                }
                crate::hir::HirStmt::If { then_block, elifs, else_block, .. } => {
                    if let Some(span) = find(&then_block.stmts) { return Some(span); }
                    for (_, block) in elifs {
                        if let Some(span) = find(&block.stmts) { return Some(span); }
                    }
                    if let Some(block) = else_block {
                        if let Some(span) = find(&block.stmts) { return Some(span); }
                    }
                }
                crate::hir::HirStmt::While { body, .. } => {
                    if let Some(span) = find(&body.stmts) { return Some(span); }
                }
                _ => {}
            }
        }
        None
    }
    if let Some(span) = find(stmts) {
        return Err(Error::Hir(format!(
            "function `{}` returns `{}`, but `return;` has no value (at {}:{})",
            name,
            hir_type_display(inferred_ret),
            span.start_line,
            span.start_col
        )));
    }
    Ok(())
}
