use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt> {
        match stmt {
            Stmt::ConstDecl { span, .. } => {
                Err(Error::Hir(format!(
                    "const declarations are only allowed at top level (at {}:{})",
                    span.start_line, span.start_col
                )))
            }
            Stmt::StaticDecl { span, .. } => {
                Err(Error::Hir(format!(
                    "static declarations are only allowed at top level (at {}:{})",
                    span.start_line, span.start_col
                )))
            }
            Stmt::Assign { name, value, span, .. } => {
                // M2：闭包捕获变量赋值 → env 字段写（FnMut）
                if let Some((env_var, env_ty, idx, cap_ty)) = self.lambda_env.as_ref().and_then(|env| {
                    env.lookup(name).map(|(i, t)| (env.var, env.ty.clone(), i, t))
                }) {
                    let hir_value = self.lower_expr(value)?;
                    let hir_value = coerce_expr(hir_value, &cap_ty, span)?;
                    let hir_value = implicit_move(hir_value);
                    let object: HirNodeBox = SVar { var: env_var, ty: env_ty }.into();
                    return Ok(HirStmt::FieldAssign {
                        object, field: *name, field_index: idx, field_ty: cap_ty,
                        value: hir_value, span: *span,
                    });
                }
                // M6.2：全局变量赋值（static mut）→ 穿透引用写入
                if let Some(st) = self.statics.get(name).cloned() {
                    if !st.is_mut {
                        return Err(Error::Hir(format!(
                            "cannot assign to immutable static `{}` (declare `static mut`) (at {}:{})",
                            name.as_str(), span.start_line, span.start_col
                        )));
                    }
                    let target: HirNodeBox = SGlobal { name: st.name, ty: st.ty.clone(), mutable: true }.into();
                    let hir_value = self.lower_expr(value)?;
                    let hir_value = coerce_expr(hir_value, &st.ty, span)?;
                    let hir_value = implicit_move(hir_value);
                    return Ok(HirStmt::DerefAssign { target, value: hir_value, span: *span });
                }
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
                // 已存在的变量：按既有类型做隐式数值转换 + 枚举实例化（#119/#120）
                let hir_value = match self.lookup_var(name) {
                    Some((_, var_ty, _)) => {
                        let v = self.instantiate_enum_value(hir_value, &var_ty)?;
                        coerce_expr(v, &var_ty, span)?
                    }
                    None => hir_value,
                };
                let hir_value = implicit_move(hir_value);
                // #132：比较运算在 HIR 中保留操作数类型（Bool 由 MIR 决定）→ 变量类型按 Bool
                let value_ty = if hir_value.is_comparison() { HirType::Bool } else { expr_type(&hir_value) };
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
                        // #119/#120：无载荷变体（Opt::None()）按期望返回类型实例化
                        let expr = self.instantiate_enum_value(expr, &fn_ret)?;
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
}
