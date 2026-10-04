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

    pub(crate) fn lower_lambda(&mut self, params: &Vec<(Symbol, Type)>, return_type: &Type, body: &Vec<Stmt>) -> Result<HirNodeBox> {
        // Synthesize a unique function name for the lambda
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;
        let name_sym = Symbol::intern(&lambda_name);

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
        });
        self.fn_map.entry(name_sym).or_default().push(fn_id);

        // Build a temporary Block from the body stmts
        let block_span = body.first().map(|s| s.span()).unwrap_or_default();
        let tmp_block = crate::parser::ast::block::Block::new(body.clone(), block_span);
        // Save current locals/scope before lowering lambda function
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::replace(&mut self.scopes, Vec::new());
        let hir_fn = self.lower_fn(fn_id, name_sym, params, return_type, &tmp_block, false, false, block_span, vec![], vec![], false)?;
        // Restore parent function's locals/scope
        self.locals = saved_locals;
        self.scopes = saved_scopes;
        self.lambda_fns.push(hir_fn);

        let fnptr_ty = HirType::FnPtr(
            hir_params.iter().map(|(_, t)| t.clone()).collect(),
            Box::new(hir_ret),
        );
        Ok(SFnPtr { fn_id, ty: fnptr_ty }.into())
    }
}
