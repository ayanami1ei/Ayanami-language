use super::*;

impl crate::hir::lower::Ctx {
    /// For each impl block type, check which interfaces it satisfies
    /// (structural typing: methods with same name + compatible signatures)
    pub(crate) fn build_vtables(&mut self) -> Result<()> {
        // Collect all impl types and their methods (owned copies to avoid borrow conflicts)
        let mut impl_methods: HashMap<Symbol, Vec<FnSig>> = HashMap::new();
        for sig in &self.fns {
            if !sig.params.is_empty() {
                let inner_ty = match &sig.params[0].1 {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                let type_name = match inner_ty {
                    HirType::Named(n) => Some(*n),
                    HirType::Int => Some(Symbol::intern("int")),
                    HirType::Float => Some(Symbol::intern("float")),
                    HirType::Char => Some(Symbol::intern("char")),
                    HirType::Bool => Some(Symbol::intern("bool")),
                    HirType::Void => Some(Symbol::intern("void")),
                    _ => None,
                };
                if let Some(tn) = type_name {
                    impl_methods.entry(tn).or_default().push(sig.clone());
                }
            }
        }

        // Also add generic_fns methods for interface matching
        for (gf_name, _, gf_stmt) in &self.generic_fns {
            if let Stmt::FnDecl { params, return_type, .. } = gf_stmt {
                if params.is_empty() { continue; }
                let param_ty = ast_type_to_hir(&params[0].1, &self.interfaces);
                let inner_ty = match &param_ty {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                if let HirType::Named(n) = inner_ty {
                    let base = crate::hir::lower::strip_generic_name(n);
                    let hir_params: Vec<(Symbol, HirType)> = params.iter()
                        .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                        .collect();
                    let hir_return = ast_type_to_hir(return_type, &self.interfaces);
                    impl_methods.entry(base).or_default().push(FnSig {
                        name: *gf_name,
                        params: hir_params,
                        return_type: hir_return,
                    });
                }
            }
        }

        let iface_list: Vec<(Symbol, Vec<(Symbol, Option<Symbol>)>, Vec<HirInterfaceMethod>)> = self.interfaces.iter()
            .map(|(name, reg)| (*name, reg.generic_params.clone(), reg.methods.clone()))
            .collect();
        for (iface_name, iface_gp, iface_methods) in &iface_list {
            for (type_name, methods) in &impl_methods {
                let reg = InterfaceReg { generic_params: iface_gp.clone(), methods: iface_methods.clone() };
                if !iface_gp.is_empty() {
                    self.try_match_generic_interface(iface_name, &reg, type_name, methods)?;
                } else {
                    self.try_match_interface(iface_name, &reg, type_name, methods)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn try_match_interface(
        &mut self, iface_name: &Symbol, iface_reg: &InterfaceReg,
        type_name: &Symbol, methods: &[FnSig],
    ) -> Result<()> {
        let mut vtable_fns: Vec<FnId> = Vec::new();
        vtable_fns.push(FnId(usize::MAX));
        let mut all_match = true;
        for iface_method in &iface_reg.methods {
            let found = methods.iter().find(|m| m.name == iface_method.name);
            match found {
                Some(fsig) => {
                    let params_match = iface_method.params.len() == fsig.params.len() - 1
                        && iface_method.params.iter().zip(&fsig.params[1..])
                            .all(|((_, ift), (_, ft))| Self::type_matches(&ift, &ft))
                        && Self::type_matches(&iface_method.return_type, &fsig.return_type);
                    if params_match {
                        let fn_id = self.fn_map.get(&fsig.name)
                            .and_then(|ids| ids.iter().find(|id| {
                                let s = &self.fns[id.0];
                                s.params == fsig.params && s.return_type == fsig.return_type
                            }));
                        if let Some(&fid) = fn_id {
                            vtable_fns.push(fid);
                        } else {
                            all_match = false;
                            break;
                        }
                    } else {
                        let iface_sig = format!("({} self{}) -> {}",
                            iface_method.self_keyword.as_str(),
                            iface_method.params.iter().map(|(n, t)| format!(", {} {}", hir_type_display(t), n.as_str())).collect::<String>(),
                            hir_type_display(&iface_method.return_type));
                        let impl_sig = format!("({} self{}) -> {}",
                            iface_method.self_keyword.as_str(),
                            fsig.params[1..].iter().map(|(n, t)| format!(", {} {}", hir_type_display(t), n.as_str())).collect::<String>(),
                            hir_type_display(&fsig.return_type));
                        return Err(Error::Hir(format!(
                            "method `{}` in impl `{}` has wrong signature for interface `{}`:\n  expected {}\n  found    {}",
                            iface_method.name.as_str(), type_name.as_str(), iface_name.as_str(),
                            iface_sig, impl_sig)));
                    }
                }
                None => { all_match = false; break; }
            }
        }
        if all_match {
            self.vtables.push(VtableEntry {
                concrete_type: *type_name,
                interface: *iface_name,
                method_fn_ids: vtable_fns,
            });
            self.type_ifaces.entry(*type_name).or_default().push(*iface_name);
        }
        Ok(())
    }

    /// Try to match a generic interface against a type's methods.
    /// For each method in the interface, infer generic param substitutions
    /// and create a specialized interface for each valid substitution set.
    pub(crate) fn try_match_generic_interface(
        &mut self, iface_name: &Symbol, iface_reg: &InterfaceReg,
        type_name: &Symbol, methods: &[FnSig],
    ) -> Result<()> {
        let gp_names: Vec<Symbol> = iface_reg.generic_params.iter().map(|(n, _)| *n).collect();
        let mut results: Vec<HashMap<Symbol, HirType>> = vec![HashMap::new()];

        for iface_method in &iface_reg.methods {
            let mut next_results = Vec::new();
            for impl_method in methods.iter().filter(|m| m.name == iface_method.name) {
                if iface_method.params.len() != impl_method.params.len() - 1 { continue; }
                for subst in &results {
                    let mut local = subst.clone();
                    let mut ok = true;
                    for ((_, ift), (_, impt)) in iface_method.params.iter().zip(&impl_method.params[1..]) {
                        if !Self::infer_iface_generic(ift, impt, &gp_names, &mut local) { ok = false; break; }
                    }
                    if !ok { continue; }
                    if !Self::infer_iface_generic(&iface_method.return_type, &impl_method.return_type, &gp_names, &mut local) {
                        continue;
                    }
                    if local.iter().any(|(k, _)| gp_names.contains(k)) {
                        let fn_id = self.fn_map.get(&impl_method.name)
                            .and_then(|ids| ids.iter().find(|id| {
                                let s = &self.fns[id.0];
                                s.params == impl_method.params && s.return_type == impl_method.return_type
                            }));
                        if let Some(&fid) = fn_id {
                            local.insert(Symbol::intern(&format!("__fn{}", iface_method.name.as_str())), HirType::Named(Symbol::intern(&format!("id{}", fid.0))));
                            next_results.push((local, fid));
                        }
                    }
                }
            }
            if next_results.is_empty() { return Ok(()); }
            results = next_results.iter().map(|(s, _)| s.clone()).collect();
        }

        // Build specialized interface name for each result
        for subst in &results {
            let args_str: Vec<String> = gp_names.iter()
                .map(|n| subst.get(n).map(|t| hir_type_display(t)).unwrap_or_else(|| n.to_string()))
                .collect();
            let specialized_name = format!("{}<{}>", iface_name.as_str(), args_str.join(","));
            let specialized_sym = Symbol::intern(&specialized_name);

            // Register the specialized interface
            if !self.interfaces.contains_key(&specialized_sym) {
                let subst_methods: Vec<HirInterfaceMethod> = iface_reg.methods.iter().map(|m| {
                    HirInterfaceMethod {
                        name: m.name,
                        self_keyword: m.self_keyword,
                        params: m.params.iter().map(|(n, t)| (*n, Self::substitute_iface_type(t, subst, &gp_names))).collect(),
                        return_type: Self::substitute_iface_type(&m.return_type, subst, &gp_names),
                    }
                }).collect();
                self.interfaces.insert(specialized_sym, InterfaceReg { generic_params: vec![], methods: subst_methods });
            }

            // Build vtable for this specialized interface
            let mut vtable_fns: Vec<FnId> = vec![FnId(usize::MAX)];
            let mut ok = true;
            for iface_method in &iface_reg.methods {
                let substituted_params: Vec<HirType> = iface_method.params.iter().map(|(_, t)| Self::substitute_iface_type(t, subst, &gp_names)).collect();
                let substituted_ret = Self::substitute_iface_type(&iface_method.return_type, subst, &gp_names);
                let found = methods.iter().filter(|m| m.name == iface_method.name).find(|m| {
                    m.params.len() - 1 == substituted_params.len()
                    && m.params[1..].iter().zip(&substituted_params).all(|((_, pt), st)| pt == st)
                    && Self::type_matches(&m.return_type, &substituted_ret)
                });
                match found {
                    Some(fsig) => {
                        let fn_id = self.fn_map.get(&fsig.name).and_then(|ids| ids.iter().find(|id| {
                            let s = &self.fns[id.0]; s.params == fsig.params && s.return_type == fsig.return_type
                        }));
                        if let Some(&fid) = fn_id { vtable_fns.push(fid); } else { ok = false; break; }
                    }
                    None => { ok = false; break; }
                }
            }
            if ok {
                self.vtables.push(VtableEntry { concrete_type: *type_name, interface: specialized_sym, method_fn_ids: vtable_fns });
                self.type_ifaces.entry(*type_name).or_default().push(specialized_sym);
            }
        }
        Ok(())
    }
}
