use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox> {
        match expr {
            Expr::Literal(lit) => self.lower_literal(lit),
            Expr::Ident(name, span) => {
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
            Expr::FnCall { name, args, span } => self.lower_fn_call(name, args, span),
            Expr::CallExpr { target, args, span } => self.lower_call_expr(target, args, span),
            Expr::TryOp(inner, span) => self.lower_try_op(inner, span),
            Expr::Match { .. } => todo!(),
            Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, span } => self.lower_enum_construct(enum_name, variant_name, tuple_args, named_args, span),
            Expr::MethodCall { object, method, args, span } => self.lower_method_call(object, method, args, span),
            Expr::Move(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                Ok(SMove { expr: hir_inner, ty }.into())
            }
            Expr::Clone(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                match &ty {
                    HirType::Shared(inner_ty) if needs_deep_copy(inner_ty) => {
                        let new_ty = HirType::Shared(inner_ty.clone());
                        Ok(SToShared { expr: SClone { expr: hir_inner, ty: ty.clone() }.into(), ty: new_ty }.into())
                    }
                    HirType::Unique(inner_ty) if needs_deep_copy(inner_ty) => {
                        let new_ty = HirType::Unique(inner_ty.clone());
                        Ok(SToUnique { expr: SClone { expr: hir_inner, ty: ty.clone() }.into(), ty: new_ty }.into())
                    }
                    _ if needs_deep_copy(&ty) => {
                        let new_ty = HirType::Unique(Box::new(ty.clone()));
                        Ok(SToUnique { expr: SClone { expr: hir_inner, ty: ty.clone() }.into(), ty: new_ty }.into())
                    }
                    _ => Ok(SClone { expr: hir_inner, ty }.into()),
                }
            }
            Expr::ToUnique(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = hir_inner.expr_type();
                let ty = HirType::Unique(Box::new(strip_ownership(inner_ty)));
                Ok(SToUnique { expr: hir_inner, ty }.into())
            }
            Expr::ToShared(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = hir_inner.expr_type();
                let ty = HirType::Shared(Box::new(strip_ownership(inner_ty)));
                Ok(SToShared { expr: hir_inner, ty }.into())
            }
            Expr::ToWeak(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = hir_inner.expr_type();
                let ty = HirType::Weak(Box::new(strip_ownership(inner_ty)));
                Ok(SToWeak { expr: hir_inner, ty }.into())
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
                    HirType::ArraySized(Box::new(elem_ty.clone()), *n as usize)
                } else {
                    HirType::Array(Box::new(elem_ty.clone()))
                };
                Ok(SArrSz { count: hir_count, elem_ty, ty }.into())
            }
            Expr::Asm { template, outputs, inputs, .. } => self.lower_asm(template, outputs, inputs),
            Expr::Lambda { params, return_type, body, .. } => self.lower_lambda(params, return_type, body),
        }
    }
}
