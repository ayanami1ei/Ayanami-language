use super::*;

impl crate::hir::lower::Ctx {
    /// while 循环（可选 #[invariant] 检查注入到每轮体首）。
    pub(crate) fn lower_while(
        &mut self,
        cond: &Expr,
        body: &Block,
        invariants: &[(&Expr, usize, usize)],
    ) -> Result<HirStmt> {
        // #148：条件降级可能产生前置语句（`&&`/`||` 短路、if 表达式条件）；
        // 这些语句必须每轮在条件判断前执行，否则条件只求值一次（死循环/段错误）。
        let mark = self.pending_stmts.len();
        let hir_cond = auto_deref(self.lower_expr(cond)?);
        let cond_pre: Vec<HirStmt> = self.pending_stmts.split_off(mark);
        let mut hir_body = self.lower_block(body)?;
        if !invariants.is_empty() {
            let mut checks = self.loop_check_stmts(invariants)?;
            checks.append(&mut hir_body.stmts);
            hir_body.stmts = checks;
        }
        let span = cond.span();
        if cond_pre.is_empty() {
            return Ok(HirStmt::While { cond: hir_cond, body: hir_body, span });
        }
        // 条件需前置语句：`while true { <cond_pre>; if !cond { break }; <invariants+body> }`
        let mut stmts = cond_pre;
        stmts.push(HirStmt::If {
            cond: SUn {
                op: crate::parser::ast::UnaryOp::Not,
                arg: hir_cond,
                ty: HirType::Bool,
            }.into(),
            then_block: HirBlock::new(vec![HirStmt::Break { span }]),
            elifs: Vec::new(),
            else_block: None,
            span,
        });
        stmts.extend(hir_body.stmts);
        Ok(HirStmt::While {
            cond: SConst { val: HirLiteral::Bool(true), ty: HirType::Bool }.into(),
            body: HirBlock::new(stmts),
            span,
        })
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

    /// M3：`for x in iterable`（迭代器协议）：
    /// `__it = iterable; while true { if __it.next() 匹配 Some(x) { body } else { break } }`
    pub(crate) fn lower_for_in(
        &mut self,
        iter_name: Symbol,
        iterable: &Expr,
        body: &Block,
        invariants: &[(&Expr, usize, usize)],
        span: &Span,
    ) -> Result<HirStmt> {
        use crate::parser::ast::pattern::Pattern;
        self.push_scope();
        let hir_iter = self.lower_expr(iterable)?;
        let iter_ty = expr_type(&hir_iter);
        let n = self.for_iter_counter;
        self.for_iter_counter += 1;
        let tmp = Symbol::intern(&format!("__for_iter_{}", n));
        let (var_id, _, _) = self.register_or_lookup(tmp, iter_ty.clone());
        let init = HirStmt::Assign {
            target: SVar { var: var_id, ty: iter_ty }.into(),
            value: implicit_move(hir_iter),
            span: *span,
        };
        // 每轮：next() 先存入临时局部（避免条件/绑定重复求值），再匹配 Some(x)
        self.push_scope();
        let next_call = self.lower_method_call(
            &Box::new(Expr::Ident(tmp, *span)), &Symbol::intern("next"), &Vec::new(), span)?;
        let next_ty = expr_type(&next_call);
        let next_name = Symbol::intern(&format!("__for_next_{}", n));
        let (next_var, _, _) = self.register_or_lookup(next_name, next_ty.clone());
        let next_node: HirNodeBox = SVar { var: next_var, ty: next_ty.clone() }.into();
        let assign_next = HirStmt::Assign {
            target: next_node.clone(), value: next_call, span: *span,
        };
        let (cond, binds) = self.pattern_match(
            &Pattern::Enum { name: Symbol::intern("Some"), args: vec![Pattern::Binding(iter_name)] },
            &next_node, &next_ty, span)?;
        let hir_body = self.lower_block(body)?;
        let mut body_stmts = binds;
        if !invariants.is_empty() {
            body_stmts.extend(self.loop_check_stmts(invariants)?);
        }
        body_stmts.extend(hir_body.stmts);
        self.pop_scope();
        let cond = cond.unwrap_or_else(|| SConst { val: HirLiteral::Bool(true), ty: HirType::Bool }.into());
        let loop_body = HirStmt::If {
            cond,
            then_block: HirBlock::new(body_stmts),
            elifs: Vec::new(),
            else_block: Some(HirBlock::new(vec![HirStmt::Break { span: *span }])),
            span: *span,
        };
        self.pop_scope();
        let always = || -> HirNodeBox { SConst { val: HirLiteral::Bool(true), ty: HirType::Bool }.into() };
        Ok(HirStmt::Block {
            stmts: vec![
                init,
                HirStmt::While {
                    cond: always(),
                    body: HirBlock::new(vec![assign_next, loop_body]),
                    span: *span,
                },
            ],
            span: *span,
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
