use super::*;

impl crate::hir::lower::Ctx {
    /// bbp：if 表达式 —— 各分支块尾作为值，按数值提升取公共类型；
    /// 全 void 分支按语句求值（返回 void 占位值）。
    pub(crate) fn lower_if_expr(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        elifs: &[(Expr, Block)],
        else_block: &Option<Block>,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let else_block = else_block.as_ref().ok_or_else(|| Error::Hir(format!(
            "if expression requires `else` (at {}:{})", span.start_line, span.start_col
        )))?;
        let hir_cond = auto_deref(self.lower_expr(cond)?);
        crate::hir::contracts::ensure_bool_condition(&hir_cond, "if", span.start_line, span.start_col)?;

        let mut hir_conds: Vec<HirNodeBox> = Vec::new();
        let mut blocks: Vec<HirBlock> = Vec::new();
        let mut values: Vec<Option<HirNodeBox>> = Vec::new();

        let (b, t) = self.lower_block_impl(then_block, true)?;
        blocks.push(b);
        values.push(t);
        for (ec, eb) in elifs {
            let c = auto_deref(self.lower_expr(ec)?);
            crate::hir::contracts::ensure_bool_condition(&c, "elif", span.start_line, span.start_col)?;
            hir_conds.push(c);
            let (b, t) = self.lower_block_impl(eb, true)?;
            blocks.push(b);
            values.push(t);
        }
        let (b, t) = self.lower_block_impl(else_block, true)?;
        blocks.push(b);
        values.push(t);

        let is_void = |v: &Option<HirNodeBox>| matches!(v.as_ref().map(|v| strip_ownership(expr_type(v))), None | Some(HirType::Void));
        let all_void = values.iter().all(is_void);
        let any_void = values.iter().any(is_void);
        if any_void && !all_void {
            return Err(Error::Hir(format!(
                "if branches have inconsistent types (void vs value) (at {}:{})",
                span.start_line, span.start_col
            )));
        }

        // void：分支尾按语句求值丢弃，返回 void 占位
        if all_void {
            let mut assigned: Vec<HirBlock> = Vec::new();
            for (mut b, v) in blocks.into_iter().zip(values) {
                if let Some(v) = v {
                    b.stmts.push(HirStmt::Expr { expr: v, span: *span });
                }
                assigned.push(b);
            }
            let else_hir = assigned.pop().unwrap();
            let then_hir = assigned.remove(0);
            let hir_elifs: Vec<(HirNodeBox, HirBlock)> = hir_conds.into_iter().zip(assigned).collect();
            self.pending_stmts.push(HirStmt::If {
                cond: hir_cond,
                then_block: then_hir,
                elifs: hir_elifs,
                else_block: Some(else_hir),
                span: *span,
            });
            return Ok(SConst { val: HirLiteral::Int(0), ty: HirType::Void }.into());
        }

        // 值：公共类型（数值提升）
        let vals: Vec<HirNodeBox> = values.into_iter().map(|v| v.unwrap()).collect();
        let res_ty = vals.iter().map(expr_type)
            .fold(None::<HirType>, |acc, t| Some(match acc {
                Some(a) => super::match_lower::match_result_type(&a, &t),
                None => t,
            }))
            .unwrap_or(HirType::Int);
        let res_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__if_res"), res_ty.clone(), true));
        let res_node: HirNodeBox = SVar { var: res_var, ty: res_ty.clone() }.into();

        let mut assigned: Vec<HirBlock> = Vec::new();
        for (mut b, v) in blocks.into_iter().zip(vals) {
            // #119/#120：无载荷变体（Opt::None()）按公共结果类型实例化
            let v = self.instantiate_enum_value(v, &res_ty)?;
            let v = coerce_expr(v, &res_ty, span)?;
            b.stmts.push(HirStmt::Assign { target: res_node.clone(), value: v, span: *span });
            assigned.push(b);
        }
        let else_hir = assigned.pop().unwrap();
        let then_hir = assigned.remove(0);
        let hir_elifs: Vec<(HirNodeBox, HirBlock)> = hir_conds.into_iter().zip(assigned).collect();
        self.pending_stmts.push(HirStmt::If {
            cond: hir_cond,
            then_block: then_hir,
            elifs: hir_elifs,
            else_block: Some(else_hir),
            span: *span,
        });
        Ok(res_node)
    }
}
