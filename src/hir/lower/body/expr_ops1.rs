use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_binary(&mut self, op: &BinaryOp, lhs: &Box<Expr>, rhs: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let mut hir_lhs = auto_deref(self.lower_expr(lhs)?);
        let mut hir_rhs = auto_deref(self.lower_expr(rhs)?);
        let mut lhs_ty = expr_type(&hir_lhs);
        let mut rhs_ty = expr_type(&hir_rhs);
        // A6：基元混合类型统一提升（char→int、int/char→float），
        // 否则内建运算按左操作数类型发射会生成非法 IR。
        {
            // 整数字面量适配到另一侧的定宽整数
            {
                let l0 = strip_ownership(lhs_ty.clone()).clone();
                let r0 = strip_ownership(rhs_ty.clone()).clone();
                if matches!(l0, HirType::IntN { .. }) && r0 == HirType::Int && as_int_literal(&hir_rhs).is_some() {
                    hir_rhs = retype_int_literal(hir_rhs, &lhs_ty);
                    rhs_ty = lhs_ty.clone();
                } else if matches!(r0, HirType::IntN { .. }) && l0 == HirType::Int && as_int_literal(&hir_lhs).is_some() {
                    hir_lhs = retype_int_literal(hir_lhs, &rhs_ty);
                    lhs_ty = rhs_ty.clone();
                }
            }
            let l = strip_ownership(lhs_ty.clone()).clone();
            let r = strip_ownership(rhs_ty.clone()).clone();
            // 定宽整数不做隐式提升/混合（需显式 as）
            if l != r && (matches!(l, HirType::IntN { .. }) || matches!(r, HirType::IntN { .. })) {
                return Err(Error::Hir(format!(
                    "cannot implicitly convert `{}` to `{}` (at {}:{})",
                    hir_type_display(&rhs_ty), hir_type_display(&lhs_ty),
                    span.start_line, span.start_col
                )));
            }
            if l != r {
                // 比较表达式在 HIR 中保留操作数类型，但语义上是 bool
                let l_bool = l == HirType::Bool || hir_lhs.is_comparison();
                let r_bool = r == HirType::Bool || hir_rhs.is_comparison();
                let common = if l_bool && r_bool {
                    Some(HirType::Bool)
                } else {
                    match (&l, &r) {
                        (HirType::Float, HirType::Int) | (HirType::Int, HirType::Float)
                        | (HirType::Float, HirType::Char) | (HirType::Char, HirType::Float) => Some(HirType::Float),
                        (HirType::Char, HirType::Int) | (HirType::Int, HirType::Char) => Some(HirType::Int),
                        _ => None,
                    }
                };
                if let Some(c) = common {
                    hir_lhs = coerce_expr(hir_lhs, &c, span)?;
                    hir_rhs = coerce_expr(hir_rhs, &c, span)?;
                    lhs_ty = c.clone();
                    rhs_ty = c.clone();
                }
            }
        }
        let inner_ty = strip_ownership(lhs_ty.clone());
        // M1.2：位运算仅适用于整数（`&`/`|`/`^` 也适用于 bool），移位不适用于 bool
        let is_bitwise = matches!(op, BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr);
        if is_bitwise && matches!(inner_ty, HirType::Float) {
            return Err(Error::Hir(format!(
                "cannot apply bitwise operator to `float` (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        if matches!(op, BinaryOp::Shl | BinaryOp::Shr) && matches!(inner_ty, HirType::Bool) {
            return Err(Error::Hir(format!(
                "cannot shift `bool` (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        // Detect null-vs-pointer comparison (null is lowered to Int(0))
        // Only treat as pointer comparison when the non-null side's inner type is NOT primitive
        // (e.g. Shared(Node) vs null, but NOT Unique(Int) == 0 — int is passed by value)
        let lhs_is_null = is_null_literal(&hir_lhs);
        let rhs_is_null = is_null_literal(&hir_rhs);
        let non_null_ty = if lhs_is_null { &rhs_ty } else { &lhs_ty };
        let is_null_ptr_cmp = (lhs_is_null || rhs_is_null)
            && is_pointer_type_for_cmp(non_null_ty)
            && !matches!(strip_ownership(non_null_ty.clone()), HirType::Int | HirType::Float | HirType::Char | HirType::Bool | HirType::IntN { .. });
        // Try operator overloading first: look for a matching function
        // Primitive types use built-in operators, not overloading
        // Null-vs-pointer comparisons use built-in ptr comparison, not overloading
        let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool | HirType::IntN { .. });
        if !is_primitive && !is_null_ptr_cmp {
            if let Some(op_fn_name) = binary_op_to_fn_name(op) {
                let param_types = [lhs_ty.clone(), rhs_ty.clone()];
                let fn_id = match self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                    Some(fid) => fid,
                    None => {
                        match self.specialize_generic_call(&Symbol::intern(op_fn_name), &param_types, span) {
                            Ok(fid) => fid,
                            Err(_) => {
                                return Err(Error::Hir(format!(
                                    "no matching overload of `{}` for argument types ({}, {}) at {}:{}",
                                    op_fn_name,
                                    hir_type_display(&lhs_ty), hir_type_display(&rhs_ty),
                                    span.start_line, span.start_col
                                )));
                            }
                        }
                    }
                };
                let ret_ty = self.fns[fn_id.0].return_type.clone();
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                let mut args: Vec<HirNodeBox> = vec![hir_lhs, hir_rhs].into_iter().enumerate().map(|(i, arg)| {
                    if i >= param_tys.len() { return arg; }
                    wrap_arg_for_param(arg, &param_tys[i])
                }).collect();
                self.append_caller_args(fn_id, &mut args, span);
                return Ok(SCall { fn_id, args, ty: ret_ty }.into());
            }
        }
        // M1.3：移位量按左操作数位宽掩码（Rust release 语义，所有构建模式一致）
        let hir_rhs = if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
            let bits: u32 = match &inner_ty {
                HirType::Int => 64,
                HirType::Char => 8,
                HirType::IntN { bits, .. } => *bits as u32,
                _ => 0,
            };
            if bits > 0 {
                SBin {
                    op: BinaryOp::BitAnd,
                    lhs: hir_rhs,
                    rhs: SConst { val: HirLiteral::Int((bits - 1) as i64), ty: rhs_ty.clone() }.into(),
                    ty: rhs_ty.clone(),
                }.into()
            } else { hir_rhs }
        } else { hir_rhs };
        // 注意：比较运算的 ty 保持操作数类型；结果类型（Bool）由 MIR→LIR 降级决定。
        // 逻辑与/或的结果是 bool，直接给 Bool 类型。
        let binop_ty = if matches!(op, BinaryOp::And | BinaryOp::Or) {
            HirType::Bool
        } else if is_null_ptr_cmp {
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
        let hir_arg = auto_deref(self.lower_expr(arg)?);
        let arg_ty = expr_type(&hir_arg);
        let inner_ty = strip_ownership(arg_ty.clone());
        // M1.2：`~` 仅适用于整数/char
        if matches!(op, UnaryOp::BitNot) && matches!(inner_ty, HirType::Float | HirType::Bool) {
            return Err(Error::Hir(format!(
                "cannot apply `~` to `{}` (at {}:{})",
                hir_type_display(&inner_ty),
                arg.span().start_line, arg.span().start_col
            )));
        }
        // Try operator overloading (skip for primitive types)
        let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool | HirType::IntN { .. });
        if !is_primitive {
            if let Some(op_fn_name) = unary_op_to_fn_name(op) {
                let param_types = [arg_ty.clone()];
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    let mut args = vec![implicit_move(hir_arg)];
                    self.append_caller_args(fn_id, &mut args, &arg.span());
                    return Ok(SCall { fn_id, args, ty: ret_ty }.into());
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

    /// A3d：`expr?` 真传播——`Ok(v)` 取值；`Err` 直接把整个 Result 返回（要求与函数返回类型一致）。
    ///
    /// 生成（内联到当前语句之前）：
    /// ```text
    /// __try_val = <expr>
    /// if __try_val._tag == 0 { __try_ok = __try_val._data_Ok } else { return __try_val }
    /// ```
    /// 表达式结果 = `__try_ok`。
    pub(crate) fn lower_try_op(&mut self, inner: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let hir_inner = self.lower_expr(inner)?;
        let inner_ty = expr_type(&hir_inner);
        let fn_ret = self.fns[self.current_fn.0].return_type.clone();
        if fn_ret != inner_ty {
            return Err(Error::Hir(format!(
                "cannot use `?`: expression type {} does not match function return type {} (at {}:{})",
                hir_type_display(&inner_ty),
                hir_type_display(&fn_ret),
                span.start_line,
                span.start_col
            )));
        }

        let tag_field = Symbol::intern("_tag");
        let ok_field = Symbol::intern("_data_Ok");
        let payload_field = Symbol::intern("_0");
        let tag_index = self.find_field_index(&inner_ty, &tag_field, span)?;
        let ok_index = self.find_field_index(&inner_ty, &ok_field, span)?;
        let data_ty = self.find_field_type(&inner_ty, &ok_field, span)?;
        let ok_ty = self.variant_payload_type(&inner_ty, &ok_field, span)?;
        // 泛型枚举载荷结构体尚未单态化：变体字段仍是泛型参数时明确报错
        if let HirType::Named(vn) = &data_ty {
            if let Some(fields) = self.struct_defs.get(vn) {
                if let Some(f0) = fields.first() {
                    if let HirType::Named(pn) = &f0.ty {
                        let gps = self.collected_generic_params(vn);
                        if gps.iter().any(|(g, _)| g == pn) {
                            return Err(Error::Hir(format!(
                                "cannot use `?` on generic enum `{}`: payload monomorphization is not implemented yet (at {}:{})",
                                hir_type_display(&inner_ty), span.start_line, span.start_col
                            )));
                        }
                    }
                }
            }
        }

        // __try_val = <expr>
        let val_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__try_val"), inner_ty.clone(), false));
        let val_node: HirNodeBox = SVar { var: val_var, ty: inner_ty.clone() }.into();
        self.pending_stmts.push(HirStmt::Assign { target: val_node.clone(), value: hir_inner, span: Span::default() });

        // __try_ok（结果）
        let ok_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__try_ok"), ok_ty.clone(), true));

        // 条件：__try_val._tag == 0（Ok 为声明序 0）
        let tag_node: HirNodeBox = SField {
            object: val_node.clone(),
            field: tag_field,
            field_index: tag_index,
            ty: HirType::Int,
        }.into();
        let zero: HirNodeBox = SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into();
        let cond: HirNodeBox = SBin {
            op: BinaryOp::Eq,
            lhs: tag_node,
            rhs: zero,
            ty: HirType::Int,
        }.into();

        // then: __try_ok = __try_val._data_Ok._0
        let variant_node: HirNodeBox = SField {
            object: val_node.clone(),
            field: ok_field,
            field_index: ok_index,
            ty: data_ty,
        }.into();
        let ok_load: HirNodeBox = SField {
            object: variant_node,
            field: payload_field,
            field_index: 0,
            ty: ok_ty.clone(),
        }.into();
        let then_block = HirBlock::new(vec![HirStmt::Assign {
            target: SVar { var: ok_var, ty: ok_ty.clone() }.into(),
            value: ok_load,
            span: Span::default(),
        }]);

        // else: return __try_val（同类型的 Result）
        let else_block = HirBlock::new(vec![HirStmt::Return { value: Some(val_node), span: Span::default() }]);

        self.pending_stmts.push(HirStmt::If {
            cond,
            then_block,
            elifs: Vec::new(),
            span: Span::default(),
            else_block: Some(else_block),
        });

        Ok(SVar { var: ok_var, ty: ok_ty }.into())
    }
}
