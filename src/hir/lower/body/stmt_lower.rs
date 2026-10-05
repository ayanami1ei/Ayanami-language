use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt> {
        match stmt {
            Stmt::Assign { name, value, span, .. } => {
                if let Some(stmt) = self.try_lower_ref_assign(name, value, span)? {
                    return Ok(stmt);
                }
                // 后续用法推断出的泛型实参（如 `res = ArrayList::new(); res.push(TokenType::...)`）
                let hint = self.usage_hints.get(name).cloned();
                let hir_value = match (value, hint.as_ref()) {
                    (Expr::FnCall { name: fname, args, generic_args, .. }, Some(h))
                        if generic_args.is_empty() => self.lower_fn_call(fname, args, Some(h), span)?,
                    _ => self.lower_expr(value)?,
                };
                // 已存在的变量：按既有类型做隐式数值转换
                let hir_value = match self.lookup_var(name) {
                    Some((_, var_ty, _)) => coerce_expr(hir_value, &var_ty, span)?,
                    None => hir_value,
                };
                let hir_value = implicit_move(hir_value);
                let value_ty = expr_type(&hir_value);
                let (var_id, ty, _) = self.register_or_lookup(*name, value_ty);
                Ok(HirStmt::Assign {
                    target: SVar { var: var_id, ty: ty.clone() }.into(),
                    value: hir_value,
                    span: *span,
                })
            }
            Stmt::FieldAssign { object, field, value, span: stmt_span } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                let field_index = self.find_field_index(&object_ty, field, stmt_span)?;
                let field_ty = self.find_field_type(&object_ty, field, stmt_span)?;
                let hir_value = self.lower_expr(value)?;
                // 字段类型隐式数值转换
                let hir_value = coerce_expr(hir_value, &field_ty, stmt_span)?;
                let hir_value = implicit_move(hir_value);
                // A4b-2：引用字段允许存储（非引用字段仍拒绝）
                if matches!(expr_type(&hir_value), HirType::Ref(..))
                    && !matches!(field_ty, HirType::Ref(..))
                {
                    return Err(Error::Hir(format!(
                        "references cannot be stored in non-reference fields (at {}:{})",
                        stmt_span.start_line, stmt_span.start_col
                    )));
                }
                Ok(HirStmt::FieldAssign {
                    object: hir_object,
                    field: *field,
                    field_index,
                    field_ty,
                    value: hir_value,
                    span: *stmt_span,
                })
            }
            Stmt::IndexAssign { object, index, value, span } => {
                let hir_object = self.lower_expr(object)?;
                let hir_index = coerce_index(self.lower_expr(index)?, &HirType::Int, span)?;
                let hir_value = self.lower_expr(value)?;
                let hir_value = implicit_move(hir_value);
                if matches!(expr_type(&hir_value), HirType::Ref(..)) {
                    return Err(Error::Hir(format!(
                        "references cannot be stored in arrays (at {}:{})",
                        span.start_line, span.start_col
                    )));
                }
                Ok(HirStmt::IndexAssign {
                    object: hir_object,
                    index: hir_index,
                    value: hir_value,
                    span: *span,
                })
            }
            Stmt::Return { value, span } => {
                if value.is_none() {
                    let ret_ty = self.fns[self.current_fn.0].return_type.clone();
                    if !matches!(ret_ty, HirType::Void) {
                        let fn_name = self.fns[self.current_fn.0].name;
                        return Err(Error::Hir(format!(
                            "function `{}` returns `{}` but `return;` has no value (at {}:{})",
                            fn_name, hir_type_display(&ret_ty), span.start_line, span.start_col
                        )));
                    }
                }
                let hir_value = match value {
                    Some(v) => {
                        let expr = self.lower_expr(v)?;
                        // 隐式数值转换：按函数返回类型
                        let fn_ret = self.fns[self.current_fn.0].return_type.clone();
                        let expr = coerce_expr(expr, &fn_ret, span)?;
                        let expr_ty = expr_type(&expr);
                        // 若函数返回 unique T，但表达式是裸 T，自动包装为 ToUnique
                        let wrapped = match (&fn_ret, &expr_ty) {
                            (HirType::Unique(pt), _) if *pt.as_ref() == expr_ty => {
                                SToUnique { expr, ty: fn_ret.clone() }.into()
                            }
                            _ => expr,
                        };
                        // 泛型单态化：枚举构造按返回类型实例化
                        let wrapped = self.instantiate_enum_value(wrapped, &fn_ret)?;
                        Some(implicit_move(wrapped))
                    }
                    None => None,
                };
                Ok(HirStmt::Return { value: hir_value, span: *span })
            }
            Stmt::If { cond, then_block, elifs, else_block, span } => {
                let hir_cond = auto_deref(self.lower_expr(cond)?);
                let hir_then = self.lower_block(then_block)?;
                let hir_elifs = elifs.iter()
                    .map(|(ec, eb)| {
                        let c = self.lower_expr(ec).map(auto_deref);
                        let b = self.lower_block(eb);
                        c.and_then(|c| b.map(|b| (c, b)))
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let hir_else = match else_block {
                    Some(b) => Some(self.lower_block(b)?),
                    None => None,
                };
                Ok(HirStmt::If {
                    cond: hir_cond,
                    then_block: hir_then,
                    elifs: hir_elifs,
                    else_block: hir_else,
                    span: *span,
                })
            }
            Stmt::For { iterator, start, end, step, body, .. } => {
                self.lower_for(*iterator, start, end, step.as_ref(), body, &[])
            }
            Stmt::While { cond, body, .. } => self.lower_while(cond, body, &[]),
            Stmt::Attributed { attrs, stmt: inner, .. } => {
                // cfg 已在 hir::cfg::filter_program 中过滤，这里只剩 invariant
                let invs = crate::hir::contracts::invariant_conditions(attrs);
                match inner.as_ref() {
                    Stmt::While { cond, body, .. } => self.lower_while(cond, body, &invs),
                    Stmt::For { iterator, start, end, step, body, .. } => {
                        self.lower_for(*iterator, start, end, step.as_ref(), body, &invs)
                    }
                    _ => self.lower_stmt(inner),
                }
            }
            Stmt::Match { value, arms, span } => self.lower_match_stmt(value, arms, span),
            Stmt::Break { span } => Ok(HirStmt::Break { span: *span }),
            Stmt::Continue { span } => Ok(HirStmt::Continue { span: *span }),
            Stmt::ExprStmt { expr, span } => {
                let hir_expr = self.lower_expr(expr)?;
                Ok(HirStmt::Expr { expr: hir_expr, span: *span })
            }
            Stmt::Namespace { .. } | Stmt::FnDecl { .. } | Stmt::StructDef { .. } | Stmt::EnumDef { .. } | Stmt::InterfaceDef { .. } | Stmt::ImplBlock { .. } | Stmt::Import { .. } => {
                let s = stmt.span();
                Err(Error::Hir(format!("unexpected declaration inside function body (at {}:{})", s.start_line, s.start_col)))
            }
        }
    }

    /// while 循环（可选 #[invariant] 检查注入到每轮体首）。
    pub(crate) fn lower_while(
        &mut self,
        cond: &Expr,
        body: &Block,
        invariants: &[(&Expr, usize, usize)],
    ) -> Result<HirStmt> {
        let hir_cond = auto_deref(self.lower_expr(cond)?);
        let mut hir_body = self.lower_block(body)?;
        if !invariants.is_empty() {
            let mut checks = self.loop_check_stmts(invariants)?;
            checks.append(&mut hir_body.stmts);
            hir_body.stmts = checks;
        }
        let span = cond.span();
        Ok(HirStmt::While { cond: hir_cond, body: hir_body, span })
    }

    pub(crate) fn lower_for(
        &mut self,
        iter_name: Symbol,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        body: &Block,
        invariants: &[(&Expr, usize, usize)],
    ) -> Result<HirStmt> {
        self.push_scope();

        let hir_start = auto_deref(self.lower_expr(start)?);
        let hir_end = auto_deref(self.lower_expr(end)?);
        let raw_step = match step {
            Some(s) => auto_deref(self.lower_expr(s)?),
            None => SConst { val: HirLiteral::Int(1), ty: HirType::Int }.into(),
        };
        // #99：区间公共整数类型（字面量适配到非字面量边界；非字面量混用报错）
        let span = body.span;
        let ty = for_bound_type(&hir_start, &hir_end, &raw_step, &span)?;
        let hir_start = coerce_for_bound(hir_start, &ty, &span)?;
        let hir_end = coerce_for_bound(hir_end, &ty, &span)?;
        let step_expr = coerce_for_bound(raw_step, &ty, &span)?;
        let (var_id, _, _) = self.register_or_lookup(iter_name, ty.clone());

        let init = HirStmt::Assign {
            target: SVar { var: var_id, ty: ty.clone() }.into(),
            value: hir_start,
            span,
        };

        let cond: HirNodeBox = SBin {
            op: BinaryOp::Lt,
            lhs: SVar { var: var_id, ty: ty.clone() }.into(),
            rhs: hir_end,
            ty: ty.clone(),
        }.into();

        let hir_body = self.lower_block(body)?;
        let mut body_stmts = hir_body.stmts;
        // A2f：循环不变式（迭代变量仍在作用域内）
        if !invariants.is_empty() {
            let mut checks = self.loop_check_stmts(invariants)?;
            checks.append(&mut body_stmts);
            body_stmts = checks;
        }

        body_stmts.push(HirStmt::Assign {
            target: SVar { var: var_id, ty: ty.clone() }.into(),
            value: SBin {
                op: BinaryOp::Add,
                lhs: SVar { var: var_id, ty: ty.clone() }.into(),
                rhs: step_expr,
                ty: ty.clone(),
            }.into(),
            span,
        });

        self.pop_scope();

        Ok(HirStmt::Block {
            stmts: vec![
                init,
                HirStmt::While { cond, body: HirBlock::new(body_stmts), span },
            ],
            span,
        })
    }

    // ----------------------------------------------------------------
    //  表达式降级：将 AST 表达式递归降级为 HIR 表达式
    //  处理字面量、标识符、二元/一元运算、函数/方法调用、
    //  字段访问、结构体/数组字面量、指针比较等
    // ----------------------------------------------------------------
}

/// #99：for 区间公共整数类型 —— 非字面量边界优先（字面量适配到它）；
/// 非字面量整数类型混用报错（需显式转换）。
fn for_bound_type(start: &HirNodeBox, end: &HirNodeBox, step: &HirNodeBox, span: &Span) -> Result<HirType> {
    let st = strip_ownership(expr_type(start));
    let et = strip_ownership(expr_type(end));
    let pt = strip_ownership(expr_type(step));
    for (t, which) in [(&st, "start"), (&et, "end"), (&pt, "step")] {
        if !is_int_type(t) {
            return Err(Error::Hir(format!(
                "for {} bound must be an integer, found `{}` (at {}:{})",
                which, hir_type_display(t), span.start_line, span.start_col
            )));
        }
    }
    let mut concrete: Option<HirType> = None;
    for (e, t) in [(start, &st), (end, &et), (step, &pt)] {
        // 仅「无后缀」整数字面量（类型为 int）可适配；`0usize` 等带后缀字面量是具体类型
        if is_numeric_literal(e) && *t == HirType::Int { continue; }
        match &concrete {
            None => concrete = Some(t.clone()),
            Some(c) if c == t => {}
            Some(c) => return Err(Error::Hir(format!(
                "for bounds have different integer types: `{}` and `{}` (add an explicit cast) (at {}:{})",
                hir_type_display(c), hir_type_display(t), span.start_line, span.start_col
            ))),
        }
    }
    Ok(concrete.unwrap_or(HirType::Int))
}

/// 边界字面量适配到公共类型（非字面量已由 for_bound_type 保证同型）
fn coerce_for_bound(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox> {
    let src = strip_ownership(expr.expr_type());
    if src == *target {
        return Ok(expr);
    }
    if is_numeric_literal(&expr) {
        return Ok(retype_int_literal(expr, target));
    }
    Err(Error::Hir(format!(
        "cannot implicitly convert `{}` to `{}` in for bounds (at {}:{})",
        hir_type_display(&src), hir_type_display(target), span.start_line, span.start_col
    )))
}
