use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox> {
        match expr {
            Expr::Literal(lit) => self.lower_literal(lit),
            Expr::Ident(name, span) => {
                // `Enum::Variant`（无参数）被解析器合并为 `Enum.Variant`
                if let Some((enum_name, variant_name)) = name.as_str().split_once('.') {
                    let enum_sym = Symbol::intern(enum_name);
                    if self.is_enum_type(&enum_sym) {
                        return self.lower_enum_construct(
                            &enum_sym,
                            &Symbol::intern(variant_name),
                            &vec![],
                            &vec![],
                            span,
                        );
                    }
                }
                if let Some((var_id, ty, _)) = self.lookup_var(name) {
                    return Ok(SVar { var: var_id, ty }.into());
                }
                if let Some(candidates) = self.fn_map.get(name) {
                    if let Some(&first) = candidates.first() {
                        let sig = &self.fns[first.0];
                        if sig.params.is_empty() {
                            return Err(Error::Hir(format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col)));
                        }
                        let params: Vec<HirType> = sig.params.iter().map(|(_, t)| t.clone()).collect();
                        let ret = sig.return_type.clone();
                        let fnptr_ty = HirType::FnPtr(params, Box::new(ret));
                        return Ok(SFnPtr { fn_id: first, ty: fnptr_ty }.into());
                    }
                }
                Err(Error::Hir(format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col)))
            }
            Expr::Binary { op, lhs, rhs, span } => self.lower_binary(op, lhs, rhs, span),
            Expr::Unary { op, arg, .. } => self.lower_unary(op, arg),
            Expr::FnCall { name, args, generic_args, span } => self.lower_fn_call(name, args, if generic_args.is_empty() { None } else { Some(generic_args) }, span),
            Expr::CallExpr { target, args, span } => self.lower_call_expr(target, args, span),
            Expr::TryOp(inner, span) => self.lower_try_op(inner, span),
            Expr::Match { value, arms, span } => self.lower_match_expr(value, arms, span),
            Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, span } => self.lower_enum_construct(enum_name, variant_name, tuple_args, named_args, span),
            Expr::MethodCall { object, method, args, span } => self.lower_method_call(object, method, args, span),
            Expr::Move(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                Ok(SMove { expr: hir_inner, ty }.into())
            }
            Expr::Clone(inner, span) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                // Copy 类型直接复制；拥有堆数据的类型没有通用深拷贝（数组运行时不带长度），
                // 引导使用各类型的 `.copy()` 方法。
                if ty.is_copy() {
                    Ok(hir_inner)
                } else {
                    Err(Error::Hir(format!(
                        "`clone` is not implemented for owned value of type {}; use `.copy()` instead (at {}:{})",
                        crate::hir::display::display_type(&ty),
                        span.start_line,
                        span.start_col
                    )))
                }
            }
            Expr::ToUnique(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = hir_inner.expr_type();
                // Copy 类型的 unique 无意义（装箱会被复制语义抵消）：零开销传递
                if inner_ty.is_copy() {
                    return Ok(hir_inner);
                }
                let ty = HirType::Unique(Box::new(strip_ownership(inner_ty)));
                Ok(SToUnique { expr: hir_inner, ty }.into())
            }
            Expr::FieldAccess { object, field, span: expr_span } => self.lower_field_access(object, field, expr_span),
            Expr::StructLiteral { type_name, generic_args, fields, .. } => self.lower_struct_literal(type_name, generic_args, fields),
            Expr::ArrayLiteral(elems, span) => self.lower_array_literal(elems, span),
            Expr::Index { object, index, span } => self.lower_index(object, index, span),
            Expr::Null(_) => {
                Ok(SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into())
            }
            Expr::Ref(inner, mutable, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let inner_ty = hir_inner.expr_type();
                let ty = HirType::Ref(Box::new(inner_ty), *mutable);
                Ok(SRef { expr: hir_inner, mutable: *mutable, ty }.into())
            }
            Expr::ArraySized { elem_type, count, .. } => {
                let hir_count = self.lower_expr(count)?;
                let elem_ty = ast_type_to_hir(elem_type, &self.interfaces);
                // If count is a compile-time constant, use ArraySized type
                let ty = if let Some(HirLiteral::Int(n)) = hir_count.as_const() {
                    HirType::Unique(Box::new(HirType::ArraySized(Box::new(elem_ty.clone()), *n as usize)))
                } else {
                    HirType::Unique(Box::new(HirType::Array(Box::new(elem_ty.clone()))))
                };
                Ok(SArrSz { count: hir_count, elem_ty, ty }.into())
            }
            Expr::Asm { template, outputs, inputs, .. } => self.lower_asm(template, outputs, inputs),
            Expr::Lambda { params, return_type, body, .. } => self.lower_lambda(params, return_type, body),
        }
    }
}
