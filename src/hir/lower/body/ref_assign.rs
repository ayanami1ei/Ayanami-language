//! `ref mut` 目标的穿透赋值（`i = v`，i 为引用参数/局部）。
use super::*;

impl crate::hir::lower::Ctx {
    /// 已有变量为引用时返回 `HirStmt::DerefAssign`；否则 `None` 交给普通赋值路径。
    pub(crate) fn try_lower_ref_assign(
        &mut self,
        name: &Symbol,
        value: &Expr,
        span: &Span,
    ) -> Result<Option<HirStmt>> {
        let Some((var_id, var_ty, _)) = self.lookup_var(name) else {
            return Ok(None);
        };
        let HirType::Ref(inner, mutable) = &var_ty else {
            return Ok(None);
        };
        let inner = (**inner).clone();
        if !*mutable {
            return Err(Error::Hir(format!(
                "cannot assign through an immutable reference `{}` (at {}:{})",
                name, span.start_line, span.start_col
            )));
        }
        let hir_value = self.lower_expr(value)?;
        let hir_value = coerce_expr(hir_value, &inner, span)?;
        let hir_value = implicit_move(hir_value);
        Ok(Some(HirStmt::DerefAssign {
            target: SVar { var: var_id, ty: var_ty }.into(),
            value: hir_value,
            span: *span,
        }))
    }
}
