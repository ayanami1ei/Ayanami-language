use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_method_call(&mut self, object: &Box<Expr>, method: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        // `Type.method(...)`：类型名不是变量；若有同名命名空间函数/类型，提示用 `::`
        if let Expr::Ident(rname, _) = object.as_ref() {
            if self.lookup_var(rname).is_none() {
                let qualified = Symbol::intern(&format!("{}.{}", rname, method));
                let has_ns_fn = self.fn_map.contains_key(&qualified)
                    || self.generic_fns.iter().any(|(n, _, _)| *n == qualified);
                let is_type = self.struct_defs.contains_key(rname)
                    || self.is_enum_type(rname)
                    || self.type_ifaces.contains_key(rname);
                if has_ns_fn || is_type {
                    return Err(Error::Hir(format!(
                        "type `{}` has no method `{}`; associated functions use `{}::{}()` (namespace call) at {}:{}",
                        rname, method, rname, method, span.start_line, span.start_col
                    )));
                }
            }
        }
        // Lower the receiver first
        let receiver = self.lower_expr(object)?;
        let receiver_ty = expr_type(&receiver);

        // Lower call arguments
        let hir_args: Vec<HirNodeBox> = args.iter()
            .map(|a| self.lower_expr(a))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
        let deref_arg_types: Vec<HirType> = arg_types.iter().map(deref_type).collect();

        // Check if receiver type is a fat pointer (interface dispatch)
        // receiver_ty may be wrapped in ownership (e.g. Shared(FatPtr))
        let receiver_inner = strip_ownership_ref(&receiver_ty);
        if let HirType::FatPtr { name: iface, .. } = receiver_inner {
            // Virtual dispatch through interface
            let iface_name = *iface;
            if !self.interfaces.contains_key(&iface_name) {
                self.ensure_specialized_interface(&iface_name)?;
            }
            let iface_reg = self.interfaces.get(&iface_name)
                .ok_or_else(|| Error::Hir(format!("unknown interface `{}` used as type (at {}:{})", iface_name, span.start_line, span.start_col)))?;

            let method_idx = iface_reg.methods.iter()
                .position(|m| m.name == *method)
                .ok_or_else(|| Error::Hir(format!("interface `{}` has no method `{}` (at {}:{})", iface_name, method, span.start_line, span.start_col)))?;

            let ret_ty = iface_reg.methods[method_idx].return_type.clone();
            if Self::contains_self_type(&ret_ty)
                || iface_reg.methods[method_idx].params.iter().any(|(_, t)| Self::contains_self_type(t))
            {
                return Err(Error::Hir(format!(
                    "interface `{}` method `{}` uses Self; dynamic dispatch (`ref {}`) is not supported — use a generic bound like `[T: {}]` (at {}:{})",
                    iface_name.as_str(), method.as_str(), iface_name.as_str(), iface_name.as_str(),
                    span.start_line, span.start_col
                )));
            }
            return Ok(SVCall {
                receiver,
                interface: iface_name,
                method_index: method_idx,
                args: hir_args,
                concrete_type: iface_name,
                ty: ret_ty,
            }.into());
        }

        // Enum method dispatch: e.method() → EnumMatch over all variants
        if let HirType::Named(type_name) = receiver_inner {
            if self.is_enum_type(type_name) {
                if let Some(enum_fields) = self.struct_defs.get(type_name).cloned() {
                    let var_fields: Vec<_> = enum_fields.iter().skip(1)
                        .filter(|f| f.name.as_str().starts_with("_data_")).collect();
                    if !var_fields.is_empty() {
                        let mut arms: Vec<(i64, HirNodeBox)> = Vec::new();
                        for (i, vf) in var_fields.iter().enumerate() {
                            let ref_ty = HirType::Ref(Box::new(vf.ty.clone()), false);
                            if let Some(fn_id) = self.resolve_method(&ref_ty, method, &arg_types) {
                                let ret_ty = self.fns[fn_id.0].return_type.clone();
                                let data_expr: HirNodeBox = SRef {
                                    expr: SField {
                                        object: receiver.clone(),
                                        field: vf.name, field_index: i + 1, ty: vf.ty.clone(),
                                    }.into(),
                                    mutable: false,
                                    ty: ref_ty.clone(),
                                }.into();
                                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
                                    .map(|(_, t)| t.clone()).collect();
                                let mut all_args: Vec<HirNodeBox> = vec![data_expr];
                                all_args.extend(hir_args.clone());
                                all_args = all_args.into_iter().enumerate().map(|(j, arg)| {
                                    if j >= param_tys.len() { return arg; }
                                    wrap_arg_for_param(arg, &param_tys[j])
                                }).collect();
                                let all_args = self.adapt_enum_args(all_args, &param_tys)?;
                                arms.push((i as i64, SCall { fn_id, args: all_args, ty: ret_ty }.into()));
                            }
                        }
                        if arms.len() == var_fields.len() && !arms.is_empty() {
                            let ret_ty = expr_type(&arms[0].1);
                            return Ok(SEnumM {
                                value: receiver,
                                arms,
                                ty: ret_ty,
                            }.into());
                        }
                    }
                }
            }
        }
        // Static dispatch: find method by receiver type（精确 → 字面量适配 → 解引用）
        let lit_mask: Vec<bool> = hir_args.iter().map(is_numeric_literal).collect();
        let fn_id = match self.resolve_method(&receiver_ty, method, &arg_types)
            .or_else(|| self.resolve_method_literals(&receiver_ty, method, &arg_types, &lit_mask))
            .or_else(|| self.resolve_method(&receiver_ty, method, &deref_arg_types))
        {
            Some(id) => id,
            None => {
                let mut all_param_types = vec![receiver_ty.clone()];
                all_param_types.extend(deref_arg_types.iter().cloned());
                let fid = match self.specialize_generic_call(method, &all_param_types, span) {
                    Ok(fid) => fid,
                    Err(_) => {
                        let mut msg = format!(
                            "type `{}` has no method `{}` for argument types ({}) at {}:{}",
                            hir_type_display(&receiver_ty),
                            method,
                            arg_types.iter().map(hir_type_display).collect::<Vec<_>>().join(", "),
                            span.start_line, span.start_col
                        );
                        if !self.receiver_type_known(&receiver_ty) {
                            msg.push_str(&format!(
                                "（类型 `{}` 未知：若来自包，请确认已 import 对应模块）",
                                hir_type_display(&receiver_ty)
                            ));
                        }
                        return Err(Error::Hir(msg));
                    }
                };
                // 泛型推导成功后，尝试更新接收者变量的类型
                if let Some(var_id) = receiver.as_local() {
                    let spec_param_ty = &self.fns[fid.0].params[0].1;
                    let recv_stripped = strip_ownership_ref(&receiver_ty);
                    let spec_stripped = strip_ownership_ref(spec_param_ty);
                    if let (HirType::Named(rn), HirType::Named(sn)) = (recv_stripped, spec_stripped) {
                        let rs = rn.as_str();
                        let ss = sn.as_str();
                        if !rs.contains('<') && ss.contains('<') {
                            let base = crate::hir::lower::strip_generic_name(&sn);
                            if base.as_str() == rs {
                                self.update_var_type(var_id, spec_param_ty.clone());
                            }
                        }
                    }
                }
                fid
            }
        };

        // Apply implicit moves and ownership conversions on all args (including receiver)
        let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
            .map(|(_, t)| t.clone())
            .collect();
        let mut all_args: Vec<HirNodeBox> = vec![receiver];
        all_args.extend(hir_args);
        for (i, arg) in all_args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if matches!(&param_tys[i], HirType::Ref(_, true)) && matches!(arg.expr_type(), HirType::Ref(_, false)) {
                return Err(Error::Hir(format!(
                    "cannot pass an immutable reference as a mutable reference (at {}:{})",
                    span.start_line, span.start_col
                )));
            }
        }
        all_args = all_args.into_iter().enumerate().map(|(i, arg)| {
            if i >= param_tys.len() { return arg; }
            wrap_arg_for_param(arg, &param_tys[i])
        }).collect();
        let mut all_args = self.adapt_enum_args(all_args, &param_tys)?;
        // 函数级 track_caller：调用点信息作为隐藏实参追加
        let hidden = self.fns[fn_id.0].hidden;
        if hidden > 0 {
            all_args.extend(self.caller_hidden_args(span, hidden));
        }

        let ty = self.fns[fn_id.0].return_type.clone();
        Ok(SCall { fn_id, args: all_args, ty }.into())
    }
}
