use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_call_expr(&mut self, target: &Box<Expr>, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        // Function call on arbitrary expression: look for `call` method
        let hir_target = self.lower_expr(target)?;
        let target_ty = expr_type(&hir_target);
        let hir_args: Vec<HirNodeBox> = args.iter()
            .map(|a| self.lower_expr(a))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
        let all_types = std::iter::once(target_ty.clone()).chain(arg_types.clone()).collect::<Vec<_>>();
        // Check if target is a function pointer type
        if let HirType::FnPtr(param_tys, ret_ty) = &target_ty {
            let args = hir_args.into_iter().enumerate().map(|(i, a)| {
                if i < param_tys.len() { wrap_arg_for_param(a, &param_tys[i]) } else { a }
            }).collect();
            return Ok(SCallP { fn_ptr: hir_target, args, ty: *ret_ty.clone() }.into());
        }
        if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern("call"), &all_types) {
            let ret_ty = self.fns[fn_id.0].return_type.clone();
            let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
            let mut all_args = vec![hir_target];
            all_args.extend(hir_args);
            all_args = all_args.into_iter().enumerate().map(|(i, arg)| {
                if i >= param_tys.len() { return arg; }
                wrap_arg_for_param(arg, &param_tys[i])
            }).collect();
            return Ok(SCall { fn_id, args: all_args, ty: ret_ty }.into());
        }
        Err(Error::Hir(format!("type `{}` cannot be called as a function at {}:{}",
            hir_type_display(&target_ty), span.start_line, span.start_col)))
    }

    /// 构造接口胖指针实参：ref 形参对裸值自动借用（不装箱）；拥有型在 LIR 装箱。
    pub(crate) fn make_fatptr_arg(&self, arg: HirNodeBox, param_ty: &HirType, ct: Symbol, iface: Symbol) -> HirNodeBox {
        let arg_ty = expr_type(&arg);
        let kind_ref = matches!(param_ty, HirType::FatPtr { kind, .. } if matches!(kind.as_ref(), HirType::Ref(..)));
        let value = if kind_ref && !matches!(arg_ty, HirType::Ref(..)) {
            let mutable = matches!(param_ty, HirType::FatPtr { kind, .. } if matches!(kind.as_ref(), HirType::Ref(_, true)));
            SRef { expr: arg, mutable, ty: HirType::Ref(Box::new(arg_ty.clone()), mutable) }.into()
        } else {
            arg
        };
        SMFP { value, concrete_type: ct, interface_name: iface, ty: param_ty.clone() }.into()
    }
}
