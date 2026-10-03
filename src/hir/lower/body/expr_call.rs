use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_fn_call(&mut self, name: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        // `Enum::Variant(args)` 被解析器合并为 `Enum.Variant`，按前缀类型分流
        if let Some((enum_name, variant_name)) = name.as_str().split_once('.') {
            let enum_sym = Symbol::intern(enum_name);
            if self.is_enum_type(&enum_sym) {
                return self.lower_enum_construct(
                    &enum_sym,
                    &Symbol::intern(variant_name),
                    args,
                    &vec![],
                    span,
                );
            }
        }

        // Step 1: lower all arguments
        let mut hir_args: Vec<HirNodeBox> = args.iter()
            .map(|a| self.lower_expr(a))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Step 2: extract arg types
        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();

        // Step 3: resolve overloaded function
        let fn_id = match self.resolve_fn_call(name, &arg_types) {
            Some(fid) => fid,
            None => {
                // Step 3b: try generic specialization
                match self.specialize_generic_call(name, &arg_types, span) {
                    Ok(fid) => fid,
                    Err(_) => {
                        // Step 3c: check if name is a variable with FnPtr type (function pointer call)
                        if let Some((var_id, ty, _)) = self.lookup_var(name) {
                            if let HirType::FnPtr(param_tys, ret_ty) = &ty {
                                let fn_ptr: HirNodeBox = SVar { var: var_id, ty: ty.clone() }.into();
                                let param_tys = param_tys.clone();
                                let args: Vec<HirNodeBox> = hir_args.into_iter().enumerate().map(|(i, a)| {
                                    if i < param_tys.len() { wrap_arg_for_param(a, &param_tys[i]) } else { a }
                                }).collect();
                                let args = self.adapt_enum_args(args, &param_tys)?;
                                return Ok(SCallP { fn_ptr, args, ty: *ret_ty.clone() }.into());
                            }
                        }
                        return Err(Error::Hir(format!("undefined function `{}` at {}:{}", name, span.start_line, span.start_col)));
                    }
                }
            }
        };

        // Step 4: wrap args into fat pointers where needed, apply implicit moves
        // Pre-register vtables for generic impl → interface (before the closure that can't use ?)
        let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
            .map(|(_, t)| t.clone())
            .collect();
        for (i, arg) in hir_args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                let arg_ty = expr_type(arg);
                if let Some(ct) = Self::extract_concrete_type_name(&arg_ty) {
                    let base_ct = crate::hir::lower::strip_generic_name(&ct);
                    if !self.type_ifaces.contains_key(&ct) || !self.type_ifaces[&ct].contains(iface_name) {
                        if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                            && self.type_ifaces[&base_ct].contains(iface_name) {
                            // already registered
                        } else if self.check_generic_fns_for_iface(&base_ct, iface_name) {
                            self.register_generic_vtable(&ct, &base_ct, iface_name)?;
                        }
                    }
                }
            }
        }
        for (i, arg) in hir_args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if matches!(&param_tys[i], HirType::Ref(_, true)) && matches!(arg.expr_type(), HirType::Ref(_, false)) {
                return Err(Error::Hir(format!(
                    "cannot pass an immutable reference as a mutable reference (at {}:{})",
                    span.start_line, span.start_col
                )));
            }
        }
        hir_args = hir_args.into_iter().enumerate().map(|(i, arg)| {
            if i >= param_tys.len() { return arg; }
            let arg_ty = expr_type(&arg);
            // Check if param expects FatPtr and arg is a concrete type that implements the interface
            if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                let concrete_type = match &arg_ty {
                    HirType::Unique(inner) => {
                        if let HirType::Named(n) = inner.as_ref() { Some(*n) } else { None }
                    }
                    HirType::Named(n) => Some(*n),
                    HirType::Int => Some(Symbol::intern("int")),
                    HirType::Float => Some(Symbol::intern("float")),
                    HirType::Char => Some(Symbol::intern("char")),
                    HirType::Bool => Some(Symbol::intern("bool")),
                    _ => None,
                };
                if let Some(ct) = concrete_type {
                    if self.type_ifaces.contains_key(&ct)
                        && self.type_ifaces[&ct].contains(iface_name) {
                        // Build fat pointer
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                    let base_ct = crate::hir::lower::strip_generic_name(&ct);
                    if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                        && self.type_ifaces[&base_ct].contains(iface_name) {
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                    if self.type_ifaces.contains_key(&ct) && self.type_ifaces[&ct].contains(iface_name) {
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                }
            }
            if matches!(param_tys[i], HirType::Unique(_) | HirType::Ref(..)) {
                wrap_arg_for_param(arg, &param_tys[i])
            } else if implicit_cast_ok(&arg_ty, &param_tys[i]) {
                // 按值基元参数的隐式数值转换（char→int / int→float / char→float）
                SCast { expr: arg, ty: strip_ownership_ref(&param_tys[i]).clone() }.into()
            } else {
                arg
            }
        }).collect();

        let hir_args = self.adapt_enum_args(hir_args, &param_tys)?;
        let ty = self.fns[fn_id.0].return_type.clone();
        Ok(SCall { fn_id, args: hir_args, ty }.into())
    }

    pub(crate) fn lower_method_call(&mut self, object: &Box<Expr>, method: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        // Lower the receiver first
        let receiver = self.lower_expr(object)?;
        let receiver_ty = expr_type(&receiver);

        // Lower call arguments
        let hir_args: Vec<HirNodeBox> = args.iter()
            .map(|a| self.lower_expr(a))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();

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
        // Static dispatch: find method by receiver type
        let fn_id = match self.resolve_method(&receiver_ty, method, &arg_types) {
            Some(id) => id,
            None => {
                let mut all_param_types = vec![receiver_ty.clone()];
                all_param_types.extend(arg_types.iter().cloned());
                let fid = self.specialize_generic_call(method, &all_param_types, span)?;
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
        let all_args = self.adapt_enum_args(all_args, &param_tys)?;

        let ty = self.fns[fn_id.0].return_type.clone();
        Ok(SCall { fn_id, args: all_args, ty }.into())
    }
}
