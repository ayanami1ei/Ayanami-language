use super::*;

impl crate::hir::lower::Ctx {
    /// M1.3：显式转换 `expr as T`（仅基元数值类型）
    pub(crate) fn lower_cast(&mut self, inner: &Box<Expr>, ty: &Type, span: &Span) -> Result<HirNodeBox> {
        let expr = auto_deref(self.lower_expr(inner)?);
        let src = strip_ownership(expr_type(&expr));
        let target = ast_type_to_hir(ty, &self.interfaces);
        let is_prim = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { .. } | HirType::Char | HirType::Bool | HirType::Float);
        if !is_prim(&src) || !is_prim(&target) {
            return Err(Error::Hir(format!(
                "non-primitive cast: `{}` as `{}` (at {}:{})",
                hir_type_display(&src), hir_type_display(&target),
                span.start_line, span.start_col
            )));
        }
        if matches!(target, HirType::Bool) {
            return Err(Error::Hir(format!(
                "cannot cast `{}` to `bool` (at {}:{})",
                hir_type_display(&src), span.start_line, span.start_col
            )));
        }
        Ok(SCast { expr, ty: target }.into())
    }
}
