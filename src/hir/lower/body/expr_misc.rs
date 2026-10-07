use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_asm(&mut self, template: &String, outputs: &Vec<(String, Box<Expr>)>, inputs: &Vec<(String, Box<Expr>)>) -> Result<HirNodeBox> {
        let lowered_outputs: Vec<(String, HirNodeBox)> = outputs.iter().map(|(c, e)| {
            (c.clone(), self.lower_expr(e).unwrap())
        }).collect();
        let lowered_inputs: Vec<(String, HirNodeBox)> = inputs.iter().map(|(c, e)| {
            (c.clone(), self.lower_expr(e).unwrap())
        }).collect();
        let ty = if !lowered_outputs.is_empty() {
            lowered_outputs[0].1.expr_type()
        } else {
            HirType::Void
        };
        Ok(SAsm {
            template: template.clone(),
            outputs: lowered_outputs,
            inputs: lowered_inputs,
            ty,
        }.into())
    }

    pub(crate) fn lower_lambda(&mut self, params: &Vec<(Symbol, Type)>, return_type: &Type, body: &Block) -> Result<HirNodeBox> {
        // M2：先做捕获分析，决定走裸函数指针（无捕获）还是闭包（有捕获）
        let n = self.lambda_counter;
        self.lambda_counter += 1;
        let name_sym = Symbol::intern(&format!("__lambda_{}", n));
        let caps = collect_captures(self, params, body)?;
        if !caps.is_empty() {
            return self.lower_closure_lambda(params, return_type, body, &caps, name_sym, n);
        }

        // Synthesize a unique function name for the lambda
        let hir_params: Vec<(Symbol, HirType)> = params.iter()
            .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
            .collect();
        let hir_ret = ast_type_to_hir(return_type, &self.interfaces);

        let fn_id = FnId(self.fns.len());
        self.fns.push(FnSig {
            name: name_sym,
            params: hir_params.clone(),
            return_type: hir_ret.clone(),
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: Default::default(),
            span: crate::span::Span::default(),
            hidden: 0,
            is_noreturn: matches!(hir_ret, HirType::Never),
        });
        self.fn_map.entry(name_sym).or_default().push(fn_id);

        // Save current locals/scope before lowering lambda function
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::replace(&mut self.scopes, Vec::new());
        let saved_fn = self.current_fn;
        let saved_env = self.lambda_env.take();
        let mut hir_fn = self.lower_fn(fn_id, name_sym, params, return_type, body, false, false, body.span, vec![], vec![], false)?;
        // Restore parent function's locals/scope/current_fn（否则后续 return 按 lambda 返回类型转换）
        self.locals = saved_locals;
        self.scopes = saved_scopes;
        self.current_fn = saved_fn;
        self.lambda_env = saved_env;

        // #119：未标注返回类型（Void）时从 lambda body 的 return 值推断返回类型
        let inferred_ret = if matches!(hir_ret, HirType::Void) {
            find_return_type(&hir_fn.body.stmts).unwrap_or(HirType::Void)
        } else {
            hir_ret.clone()
        };
        if inferred_ret != hir_ret {
            hir_fn.return_type = inferred_ret.clone();
            self.fns[fn_id.0].return_type = inferred_ret.clone();
        }
        self.lambda_fns.push(hir_fn);

        let param_tys: Vec<HirType> = hir_params.iter().map(|(_, t)| t.clone()).collect();
        let fnptr_ty = HirType::FnPtr(param_tys.clone(), Box::new(inferred_ret.clone()));
        // M2 统一：非捕获 lambda 也是 Fn 值（静态闭包：Copy、零分配、env 存裸代码指针）
        let raw: HirNodeBox = SFnPtr { fn_id, ty: fnptr_ty }.into();
        Ok(self.make_static_closure(raw, &param_tys, &inferred_ret))
    }
}
