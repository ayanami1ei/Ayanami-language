use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_binary(&mut self, op: &BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let hir_lhs = self.lower_expr(lhs)?;
        let hir_rhs = self.lower_expr(rhs)?;
        let lhs_ty = expr_type(&hir_lhs);
        let rhs_ty = expr_type(&hir_rhs);
        let inner_ty = strip_ownership(lhs_ty.clone());
        // Detect null-vs-pointer comparison (null is lowered to Int(0))
        // Only treat as pointer comparison when the non-null side's inner type is NOT primitive
        // (e.g. Shared(Node) vs null, but NOT Unique(Int) == 0 — int is passed by value)
        let lhs_is_null = is_null_literal(&hir_lhs);
        let rhs_is_null = is_null_literal(&hir_rhs);
        let non_null_ty = if lhs_is_null { &rhs_ty } else { &lhs_ty };
        let is_null_ptr_cmp = (lhs_is_null || rhs_is_null)
            && is_pointer_type_for_cmp(non_null_ty)
            && !matches!(strip_ownership(non_null_ty.clone()), HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
        // Try operator overloading first: look for a matching function
        // Primitive types use built-in operators, not overloading
        // Null-vs-pointer comparisons use built-in ptr comparison, not overloading
        let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
        if !is_primitive && !is_null_ptr_cmp {
            if let Some(op_fn_name) = binary_op_to_fn_name(op) {
                let param_types = [lhs_ty.clone(), rhs_ty];
                let fn_id = match self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                    Some(fid) => fid,
                    None => {
                        match self.specialize_generic_call(&Symbol::intern(op_fn_name), &param_types, span) {
                            Ok(fid) => fid,
                            Err(msg) => { return Err(msg); }
                        }
                    }
                };
                let ret_ty = self.fns[fn_id.0].return_type.clone();
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                let args = vec![hir_lhs, hir_rhs].into_iter().enumerate().map(|(i, arg)| {
                    if i >= param_tys.len() { return arg; }
                    wrap_arg_for_param(arg, &param_tys[i])
                }).collect();
                return Ok(SCall { fn_id, args, ty: ret_ty }.into());
            }
        }
        // 注意：比较运算的 ty 保持操作数类型；结果类型（Bool）由 MIR→LIR 降级决定
        let binop_ty = if is_null_ptr_cmp {
            if lhs_is_null { rhs_ty.clone() } else { lhs_ty.clone() }
        } else {
            inner_ty
        };
        Ok(SBin {
            op: *op,
            lhs: hir_lhs,
            rhs: hir_rhs,
            ty: binop_ty,
        }.into())
    }

    pub(crate) fn lower_unary(&mut self, op: &UnaryOp, arg: &Box<Expr>) -> Result<HirNodeBox> {
        let hir_arg = self.lower_expr(arg)?;
        let arg_ty = expr_type(&hir_arg);
        let inner_ty = strip_ownership(arg_ty.clone());
        // Try operator overloading (skip for primitive types)
        let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
        if !is_primitive {
            if let Some(op_fn_name) = unary_op_to_fn_name(op) {
                let param_types = [arg_ty.clone()];
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    return Ok(SCall { fn_id, args: vec![implicit_move(hir_arg)], ty: ret_ty }.into());
                }
            }
        }
        let ty = if matches!(op, UnaryOp::Not) {
            HirType::Bool
        } else {
            strip_ownership(arg_ty)
        };
        Ok(SUn {
            op: *op,
            arg: hir_arg,
            ty,
        }.into())
    }

    pub(crate) fn lower_try_op(&mut self, inner: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let hir_inner = self.lower_expr(inner)?;
        let inner_ty = expr_type(&hir_inner);
        if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern("try_unwrap"), &[inner_ty.clone()]) {
            let ret_ty = self.fns[fn_id.0].return_type.clone();
            return Ok(SCall { fn_id, args: vec![implicit_move(hir_inner)], ty: ret_ty }.into());
        }
        if let Ok(fn_id) = self.specialize_generic_call(&Symbol::intern("try_unwrap"), &[inner_ty.clone()], span) {
            let ret_ty = self.fns[fn_id.0].return_type.clone();
            return Ok(SCall { fn_id, args: vec![implicit_move(hir_inner)], ty: ret_ty }.into());
        }
        Err(Error::Hir(format!("type `{:?}` cannot use `?` operator at {}:{}", inner_ty, span.start_line, span.start_col)))
    }
}
