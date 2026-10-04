use super::*;

impl crate::hir::lower::Ctx {
    /// `explicit`：显式泛型实参（`ns.fn[T](...)`，普通调用为 None）
    pub(crate) fn lower_fn_call(&mut self, name: &Symbol, args: &Vec<Expr>, explicit: Option<&Vec<Type>>, span: &Span) -> Result<HirNodeBox> {
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

        // Step 2: extract arg types（引用实参另存解引用类型，用于回退解析）
        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
        let deref_arg_types: Vec<HirType> = arg_types.iter().map(deref_type).collect();

        // Step 3：显式泛型实参优先（零参构造函数特化后参数表相同，
        // 重载解析无法区分返回类型，必须先按显式实参特化）
        let explicit_id = match explicit {
            Some(types) => self.specialize_generic_call_with(name, &arg_types, Some(types), span).ok(),
            None => None,
        };
        // Step 3: resolve overloaded function（先按原类型，再按解引用类型）
        let fn_id = match explicit_id {
            Some(fid) => fid,
            None => match self.resolve_fn_call(name, &arg_types)
            .or_else(|| self.resolve_fn_call(name, &deref_arg_types))
        {
            Some(fid) => fid,
            None => {
                // Step 3b: try generic specialization
                let spec = self.specialize_generic_call_with(name, &arg_types, explicit, span);
                let spec = match spec {
                    Ok(fid) => Ok(fid),
                    Err(_) => self.specialize_generic_call_with(name, &deref_arg_types, explicit, span),
                };
                match spec {
                    Ok(fid) => fid,
                    Err(e) => {
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
                        // 传播真实原因；前缀类型未知时补 import 提示
                        if let Some((prefix, _)) = name.as_str().split_once('.') {
                            let pfx = Symbol::intern(prefix);
                            let base = crate::hir::lower::strip_generic_name(&pfx);
                            let known = self.is_enum_type(&pfx)
                                || self.struct_defs.contains_key(&pfx)
                                || self.struct_defs.contains_key(&base)
                                || self.type_ifaces.contains_key(&pfx);
                            if !known {
                                return Err(Error::Hir(format!(
                                    "{}（类型 `{}` 未知：若来自包，请确认已 import 对应模块）",
                                    e, prefix
                                )));
                            }
                        }
                        return Err(e);
                    }
                }
            }
        } };

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
            } else if matches!(arg_ty, HirType::Ref(..)) {
                // 值形参 + 引用实参：自动解引用
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

}
