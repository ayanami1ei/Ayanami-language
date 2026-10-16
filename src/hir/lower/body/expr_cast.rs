use super::*;

impl crate::hir::lower::Ctx {
    /// M1.3：显式转换 `expr as T`（仅基元数值类型）
    pub(crate) fn lower_cast(&mut self, inner: &Box<Expr>, ty: &Type, span: &Span) -> Result<HirNodeBox> {
        let target = ast_type_to_hir(ty, &self.interfaces);
        // M5 P2：目标是裸指针时保留 `ref` 表达式本身（取地址语义），不做值上下文自动解引用
        let expr = if matches!(target, HirType::RawPtr(_)) {
            let e = self.lower_expr(inner)?;
            match e.expr_type() {
                // `ref [T]` 的 LIR 值指向槽（槽内才是缓冲区指针）；裸指针应指向缓冲区 → 先解引用一次
                HirType::Ref(it, _) if is_array_like(&it) => {
                    SDeref { expr: e, ty: (*it).clone() }.into()
                }
                _ => e,
            }
        } else {
            auto_deref(self.lower_expr(inner)?)
        };
        let raw_src = expr_type(&expr);
        let src = strip_ownership(raw_src.clone());
        let is_prim = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { .. } | HirType::Char | HirType::Bool | HirType::Float | HirType::F32);
        // M5 P2：裸指针转换（全部需 unsafe）
        //   int/IntN ↔ *T；ref T / Ptr[T] / fn(..) / *T → *T；*T → *T
        let raw_target = matches!(&target, HirType::RawPtr(_));
        let raw_source = matches!(&raw_src, HirType::RawPtr(_));
        if raw_target || raw_source {
            self.require_unsafe("raw pointer cast", span)?;
            let is_int = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { .. });
            let src_ok = raw_source || is_int(&src)
                || matches!(&raw_src, HirType::Ref(..) | HirType::FnPtr(..) | HirType::Unique(_));
            let tgt_ok = raw_target || is_int(&target);
            if !src_ok || !tgt_ok {
                return Err(Error::Hir(format!(
                    "invalid raw pointer cast: `{}` as `{}` (at {}:{})",
                    hir_type_display(&raw_src), hir_type_display(&target),
                    span.start_line, span.start_col
                )));
            }
            return Ok(SCast { expr, ty: target }.into());
        }
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

/// 数组类类型（`[T]` / `[T; n]` / 拥有堆数组）：引用这类值在 LIR 中是「槽指针」。
fn is_array_like(t: &HirType) -> bool {
    match t {
        HirType::Array(_) | HirType::ArraySized(_, _) => true,
        HirType::Unique(inner) => matches!(inner.as_ref(), HirType::Array(_) | HirType::ArraySized(_, _)),
        _ => false,
    }
}
