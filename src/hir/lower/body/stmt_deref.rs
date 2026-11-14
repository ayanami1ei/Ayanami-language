//! `*p = v` 解引用写入（裸指针需 unsafe；#205：`ref mut T` 为安全操作）。
use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_deref_assign(
        &mut self,
        target: &Box<Expr>,
        value: &Box<Expr>,
        span: &Span,
    ) -> Result<HirStmt> {
        let hir_target = self.lower_expr(target)?;
        let target_ty = expr_type(&hir_target);
        // #205：`ref mut T` 解引用写入安全（借用检查覆盖）；`ref T` 不可写
        let ref_pointee = match &target_ty {
            HirType::Ref(inner, mutable) if *mutable => Some(inner.as_ref().clone()),
            HirType::Ref(..) => return Err(Error::Hir(format!(
                "cannot write through immutable reference `{}` (at {}:{})",
                hir_type_display(&target_ty), span.start_line, span.start_col
            ))),
            _ => None,
        };
        if ref_pointee.is_none() {
            self.require_unsafe("write through a raw pointer", span)?;
        }
        let pointee = match ref_pointee {
            Some(p) => p,
            None => match strip_ownership_ref(&target_ty) {
                HirType::RawPtr(inner) => inner.as_ref().clone(),
                other => return Err(Error::Hir(format!(
                    "cannot write through `{}`: expected a reference or raw pointer `*T` (at {}:{})",
                    hir_type_display(other), span.start_line, span.start_col
                ))),
            },
        };
        let hir_value = self.lower_expr(value)?;
        let hir_value = coerce_expr(hir_value, &pointee, span)?;
        // #201：解引用写入类型兼容检查
        ensure_assignable(&hir_value, &pointee, span,
            &format!("`{}`", hir_type_display(&target_ty)))?;
        let hir_value = implicit_move(hir_value);
        Ok(HirStmt::DerefAssign { target: hir_target, value: hir_value, span: *span })
    }
}
