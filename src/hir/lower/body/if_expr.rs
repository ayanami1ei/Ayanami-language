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

        // M1.9：分支分类 —— Void / 值(ty) / 发散(`!`)
        let mut never: Vec<bool> = Vec::new();
        let mut value_tys: Vec<HirType> = Vec::new();
        let mut has_void = false;
        for (i, v) in values.iter().enumerate() {
            match v {
                Some(e) => {
                    let ty = strip_ownership(expr_type(e));
                    if ty == HirType::Never { never.push(true); }
                    else if ty == HirType::Void { never.push(false); has_void = true; }
                    else { never.push(false); value_tys.push(ty); }
                }
                None => {
                    let d = super::match_util::block_diverges(&blocks[i]);
                    never.push(d);
                    if !d { has_void = true; }
                }
            }
        }
        if has_void && !value_tys.is_empty() {
            return Err(Error::Hir(format!(
                "if branches have inconsistent types (void vs value) (at {}:{})",
                span.start_line, span.start_col
            )));
        }

        // 无值分支（全 void / 含发散）：按语句求值；全发散 → 结果类型 `!`
        if value_tys.is_empty() {
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
            let ty = if never.iter().all(|d| *d) { HirType::Never } else { HirType::Void };
            return Ok(SConst { val: HirLiteral::Int(0), ty }.into());
        }

        // 值：公共类型（数值提升；`!` 分支不参与、不赋值）
        let res_ty = value_tys.iter()
            .fold(None::<HirType>, |acc, t| Some(match acc {
                Some(a) => super::match_util::match_result_type(&a, t),
                None => t.clone(),
            }))
            .unwrap_or(HirType::Int);
        // #135：分支构造回退到枚举基名时，用当前函数返回类型细化
        let res_ty = self.refine_enum_result_type(res_ty);
        let res_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__if_res"), res_ty.clone(), true));
        let res_node: HirNodeBox = SVar { var: res_var, ty: res_ty.clone() }.into();

        let mut assigned: Vec<HirBlock> = Vec::new();
        for (i, (mut b, v)) in blocks.into_iter().zip(values).enumerate() {
            if let Some(v) = v {
                if never[i] {
                    // 发散分支：只求值（副作用），不写结果
                    b.stmts.push(HirStmt::Expr { expr: v, span: *span });
                } else {
                    // #119/#120：无载荷变体（Opt::None()）按公共结果类型实例化
                    let v = self.instantiate_enum_value(v, &res_ty)?;
                    let v = coerce_expr(v, &res_ty, span)?;
                    b.stmts.push(HirStmt::Assign { target: res_node.clone(), value: v, span: *span });
                }
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
        Ok(res_node)
    }
}
