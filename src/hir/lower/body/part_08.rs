use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt> {
        match stmt {
            Stmt::Assign { name, value, span: _, .. } => {
                let hir_value = self.lower_expr(value)?;
                let hir_value = implicit_move(hir_value);
                let value_ty = expr_type(&hir_value);
                let (var_id, ty, _) = self.register_or_lookup(*name, value_ty);
                Ok(HirStmt::Assign {
                    target: SVar { var: var_id, ty: ty.clone() }.into(),
                    value: hir_value,
                })
            }
            Stmt::FieldAssign { object, field, value, span: stmt_span } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                let field_index = self.find_field_index(&object_ty, field, stmt_span)?;
                let field_ty = self.find_field_type(&object_ty, field, stmt_span)?;
                let hir_value = self.lower_expr(value)?;
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
                })
            }
            Stmt::IndexAssign { object, index, value, span } => {
                let hir_object = self.lower_expr(object)?;
                let hir_index = self.lower_expr(index)?;
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
                })
            }
            Stmt::Return { value, .. } => {
                let hir_value = match value {
                    Some(v) => {
                        let expr = self.lower_expr(v)?;
                        let expr_ty = expr_type(&expr);
                        // 若函数返回 unique T，但表达式是裸 T，自动包装为 ToUnique
                        let fn_ret = self.fns[self.current_fn.0].return_type.clone();
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
                Ok(HirStmt::Return { value: hir_value })
            }
            Stmt::If { cond, then_block, elifs, else_block, .. } => {
                let hir_cond = self.lower_expr(cond)?;
                let hir_then = self.lower_block(then_block)?;
                let hir_elifs = elifs.iter()
                    .map(|(ec, eb)| {
                        let c = self.lower_expr(ec);
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
            Stmt::Match { value, arms, span } => {
                let hir_value = self.lower_expr(value)?;
                let value_ty = expr_type(&hir_value);
                let value_ty_name = match &value_ty {
                    HirType::Named(n) => *n,
                    _ => return Err(Error::Hir(format!("match on non-enum type at {}:{}", span.start_line, span.start_col))),
                };
                let (val_var, _, _) = self.register_or_lookup(Symbol::intern("__match_val"), value_ty.clone());
                let val_local: HirNodeBox = SVar { var: val_var, ty: value_ty.clone() }.into();
                let store_val = HirStmt::Assign { target: val_local, value: hir_value };
                let mut conds: Vec<HirNodeBox> = Vec::new();
                let mut blocks: Vec<HirBlock> = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let tag_cmp: HirNodeBox = SBin {
                        op: crate::parser::ast::BinaryOp::Eq,
                        lhs: SField {
                            object: SVar { var: val_var, ty: value_ty.clone() }.into(),
                            field: Symbol::intern("_tag"), field_index: 0, ty: HirType::Int,
                        }.into(),
                        rhs: SConst { val: HirLiteral::Int(i as i64), ty: HirType::Int }.into(),
                        ty: HirType::Int,
                    }.into();
                    conds.push(tag_cmp);
                    let data_field = Symbol::intern(&format!("_data_{}", arm.variant_name));
                    let var_struct = Symbol::intern(&format!("{}_{}", value_ty_name, arm.variant_name));
                    let mut arm_stmts = Vec::new();
                    for (j, (bind_name, _)) in arm.bindings.iter().enumerate() {
                        let inner_acc: HirNodeBox = SField {
                            object: SVar { var: val_var, ty: value_ty.clone() }.into(),
                            field: data_field, field_index: i + 1, ty: HirType::Named(var_struct),
                        }.into();
                        let fval: HirNodeBox = SField {
                            object: inner_acc,
                            field: Symbol::intern(&format!("_{}", j)), field_index: j, ty: HirType::Int,
                        }.into();
                        let (bid, _, _) = self.register_or_lookup(*bind_name, HirType::Int);
                        arm_stmts.push(HirStmt::Assign { target: SVar { var: bid, ty: HirType::Int }.into(), value: fval });
                    }
                    arm_stmts.push(HirStmt::Expr(self.lower_expr(&arm.body)?));
                    blocks.push(HirBlock { stmts: arm_stmts });
                }
                if conds.is_empty() {
                    return Ok(HirStmt::Expr(SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into()));
                }
                let first_cond = conds.remove(0);
                let first_block = blocks.remove(0);
                let else_block = if !blocks.is_empty() && blocks.len() == conds.len() {
                    blocks.pop()
                } else {
                    None
                };
                let elifs: Vec<(HirNodeBox, HirBlock)> = conds.into_iter().zip(blocks.into_iter()).collect();
                Ok(HirStmt::Block(vec![store_val, HirStmt::If {
                    cond: first_cond, then_block: first_block, elifs, else_block,
                }]))
            }
            Stmt::Break { .. } => Ok(HirStmt::Break),
            Stmt::Continue { .. } => Ok(HirStmt::Continue),
            Stmt::ExprStmt { expr, .. } => {
                let hir_expr = self.lower_expr(expr)?;
                Ok(HirStmt::Expr(hir_expr))
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
        let hir_cond = self.lower_expr(cond)?;
        let mut hir_body = self.lower_block(body)?;
        if !invariants.is_empty() {
            let mut checks = self.loop_check_stmts(invariants)?;
            checks.append(&mut hir_body.stmts);
            hir_body.stmts = checks;
        }
        Ok(HirStmt::While { cond: hir_cond, body: hir_body })
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

        let hir_start = self.lower_expr(start)?;
        let ty = expr_type(&hir_start);
        let (var_id, _, _) = self.register_or_lookup(iter_name, ty.clone());

        let init = HirStmt::Assign {
            target: SVar { var: var_id, ty: ty.clone() }.into(),
            value: hir_start,
        };

        let hir_end = self.lower_expr(end)?;
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

        let step_expr = match step {
            Some(s) => self.lower_expr(s)?,
            None => SConst { val: HirLiteral::Int(1), ty: HirType::Int }.into(),
        };
        let step_ty = expr_type(&step_expr);
        body_stmts.push(HirStmt::Assign {
            target: SVar { var: var_id, ty: ty.clone() }.into(),
            value: SBin {
                op: BinaryOp::Add,
                lhs: SVar { var: var_id, ty: ty.clone() }.into(),
                rhs: step_expr,
                ty: step_ty,
            }.into(),
        });

        self.pop_scope();

        Ok(HirStmt::Block(vec![
            init,
            HirStmt::While { cond, body: HirBlock::new(body_stmts) },
        ]))
    }

    // ----------------------------------------------------------------
    //  表达式降级：将 AST 表达式递归降级为 HIR 表达式
    //  处理字面量、标识符、二元/一元运算、函数/方法调用、
    //  字段访问、结构体/数组字面量、指针比较等
    // ----------------------------------------------------------------
}
