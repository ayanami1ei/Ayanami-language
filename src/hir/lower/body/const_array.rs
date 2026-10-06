use super::*;

impl crate::hir::lower::Ctx {
    /// M6.2c：数组常量求值（字面量 `[a, b, ...]` / 重复 `[v; n]`；元素为标量）
    pub(crate) fn eval_const_array(
        &self,
        expr: &Expr,
        env: &HashMap<Symbol, (HirType, HirLiteral)>,
        depth: usize,
    ) -> Result<(HirType, HirLiteral)> {
        match expr {
            Expr::ArrayLiteral(elems, span) => {
                if elems.is_empty() {
                    return Err(Error::Hir(format!(
                        "empty array constant has no element type (at {}:{})",
                        span.start_line, span.start_col
                    )));
                }
                let mut vals = Vec::with_capacity(elems.len());
                let mut elem_ty: Option<HirType> = None;
                for e in elems {
                    let (t, l) = self.eval_const_expr(e, env, depth)?;
                    if let Some(prev) = &elem_ty {
                        if *prev != t {
                            return Err(Error::Hir(format!(
                                "array constant elements must have one type ({} vs {}) (at {}:{})",
                                hir_type_display(prev), hir_type_display(&t),
                                span.start_line, span.start_col
                            )));
                        }
                    } else {
                        elem_ty = Some(t);
                    }
                    vals.push(l);
                }
                let ty = HirType::Unique(Box::new(HirType::ArraySized(
                    Box::new(elem_ty.unwrap()), vals.len())));
                Ok((ty, HirLiteral::Array(vals)))
            }
            Expr::ArrayRepeat { value, count, span } => {
                let (elem_ty, elem_lit) = self.eval_const_expr(value, env, depth)?;
                let (_, n_lit) = self.eval_const_expr(count, env, depth)?;
                let n = match n_lit {
                    HirLiteral::Int(n) if n >= 0 => n as usize,
                    _ => return Err(Error::Hir(format!(
                        "repeat count must be a non-negative integer (at {}:{})",
                        span.start_line, span.start_col
                    ))),
                };
                if n > 1_000_000 {
                    return Err(Error::Hir(format!(
                        "repeat count too large ({}) (at {}:{})",
                        n, span.start_line, span.start_col
                    )));
                }
                let ty = HirType::Unique(Box::new(HirType::ArraySized(Box::new(elem_ty), n)));
                Ok((ty, HirLiteral::Array(vec![elem_lit; n])))
            }
            _ => unreachable!("eval_const_array only handles array literals"),
        }
    }
}
