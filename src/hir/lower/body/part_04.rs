use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn infer_iface_generic(
        expected: &HirType, actual: &HirType,
        gp_names: &[Symbol], subst: &mut HashMap<Symbol, HirType>,
    ) -> bool {
        match (expected, actual) {
            (HirType::Named(n), _) if gp_names.contains(n) => {
                if let Some(existing) = subst.get(n) { existing == actual }
                else { subst.insert(*n, actual.clone()); true }
            }
            _ => Self::type_matches(expected, actual),
        }
    }

    pub(crate) fn substitute_iface_type(ty: &HirType, subst: &HashMap<Symbol, HirType>, gp_names: &[Symbol]) -> HirType {
        match ty {
            HirType::Named(n) if gp_names.contains(n) => subst.get(n).cloned().unwrap_or_else(|| ty.clone()),
            HirType::Shared(inner) => HirType::Shared(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::Unique(inner) => HirType::Unique(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::Weak(inner) => HirType::Weak(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::FatPtr { name, kind } => HirType::FatPtr { name: *name, kind: Box::new(Self::substitute_iface_type(kind, subst, gp_names)) },
            HirType::FnPtr(params, ret) => HirType::FnPtr(
                params.iter().map(|p| Self::substitute_iface_type(p, subst, gp_names)).collect(),
                Box::new(Self::substitute_iface_type(ret, subst, gp_names)),
            ),
            _ => ty.clone(),
        }
    }

    pub(crate) fn type_matches(a: &HirType, b: &HirType) -> bool {
        if a == b { return true; }
        match (a, b) {
            (HirType::Named(an), HirType::Int) if an.as_str() == "int" => true,
            (HirType::Named(an), HirType::Float) if an.as_str() == "float" => true,
            (HirType::Named(an), HirType::Char) if an.as_str() == "char" => true,
            (HirType::Named(an), HirType::Void) if an.as_str() == "void" => true,
            (HirType::Named(an), HirType::Bool) if an.as_str() == "bool" => true,
            _ => false,
        }
    }

    /// Find a function by exact name + param type match (for Phase 2 lookup)
    pub(crate) fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId> {
        for (i, sig) in self.fns.iter().enumerate() {
            if sig.name == name
                && sig.params.len() == param_types.len()
                && sig.params.iter().zip(param_types).all(|((_, pt), at)| pt == at)
            {
                return Some(FnId(i));
            }
        }
        None
    }

    /// Extract a concrete type name from an HirType (stripping ownership).
    pub(crate) fn extract_concrete_type_name(ty: &HirType) -> Option<Symbol> {
        match ty {
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                Self::extract_concrete_type_name(inner)
            }
            HirType::Named(n) => Some(*n),
            HirType::Int => Some(Symbol::intern("int")),
            HirType::Float => Some(Symbol::intern("float")),
            HirType::Char => Some(Symbol::intern("char")),
            HirType::Bool => Some(Symbol::intern("bool")),
            _ => None,
        }
    }

    /// Check if an arg type can be passed to a param type (accounting for FatPtr wrapping)
    pub(crate) fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        // Check if param is FatPtr and arg is a concrete type that implements the interface
        if let HirType::FatPtr { name: iface_name, .. } = param_ty {
            let concrete = match arg_ty {
                HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                    if let HirType::Named(n) = inner.as_ref() { Some(*n) } else { None }
                }
                HirType::Named(n) => Some(*n),
                HirType::Int => Some(Symbol::intern("int")),
                HirType::Float => Some(Symbol::intern("float")),
                HirType::Char => Some(Symbol::intern("char")),
                HirType::Bool => Some(Symbol::intern("bool")),
                _ => None,
            };
            if let Some(ct) = concrete {
                if let Some(ifaces) = self.type_ifaces.get(&ct) {
                    return ifaces.contains(iface_name);
                }
                let base_ct = crate::hir::lower::strip_generic_name(&ct);
                if base_ct != ct {
                    if let Some(ifaces) = self.type_ifaces.get(&base_ct) {
                        return ifaces.contains(iface_name);
                    }
                }
                // Fallback: check generic_fns for matching impl methods
                // Only allow if the concrete type has generic params (<...>) or the struct isn't generic
                let has_gp = self.generic_struct_params.contains_key(&base_ct);
                if has_gp && !ct.as_str().contains('<') {
                    return false;
                }
                return self.check_generic_fns_for_iface(&base_ct, iface_name);
            }
        }
        false
    }

    /// Check if a type's generic_fns methods structurally match an interface.
    /// This enables on-the-fly interface matching for generic impls.
    pub(crate) fn check_generic_fns_for_iface(&self, type_name: &Symbol, iface_name: &Symbol) -> bool {
        let base_iface = crate::hir::lower::strip_generic_name(iface_name);
        let iface_reg = match self.interfaces.get(&base_iface) {
            Some(r) => r,
            None => return false,
        };
        for iface_method in &iface_reg.methods {
            let has_match = self.generic_fns.iter().any(|(gf_name, _, gf_stmt)| {
                if *gf_name != iface_method.name { return false; }
                if let Stmt::FnDecl { params, .. } = gf_stmt {
                    if params.is_empty() { return false; }
                    let self_ty = ast_type_to_hir(&params[0].1, &self.interfaces);
                    let self_inner = match &self_ty {
                        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                        other => other,
                    };
                    let self_base = match self_inner {
                        HirType::Named(n) => crate::hir::lower::strip_generic_name(n),
                        _ => return false,
                    };
                    if self_base != *type_name { return false; }
                    // iface.params excludes self, impl.params includes self
                    if params.len() - 1 != iface_method.params.len() { return false; }
                    params[1..].iter().zip(&iface_method.params).all(|((_, pt), (_, ift))| {
                        let pt_hir = ast_type_to_hir(pt, &self.interfaces);
                        Self::type_matches(&pt_hir, ift)
                    })
                } else { false }
            });
            if !has_match { return false; }
        }
        true
    }

    /// Ensure a specialized interface (e.g. "List<int>") is registered.
    pub(crate) fn ensure_specialized_interface(&mut self, specialized_name: &Symbol) -> Result<()> {
        if self.interfaces.contains_key(specialized_name) { return Ok(()); }
        let s = specialized_name.as_str();
        let base = crate::hir::lower::strip_generic_name(specialized_name);
        if base == *specialized_name { return Ok(()); } // not a specialized name
        let generics_reg = match self.interfaces.get(&base) {
            Some(r) => r.clone(),
            None => return Ok(()),
        };
        // Extract generic param values from the name
        let gp_start = s.find('<').unwrap();
        let gp_end = s.rfind('>').unwrap_or(s.len() - 1);
        let inner_str = &s[gp_start + 1..gp_end];
        let gp_values: Vec<HirType> = inner_str.split(',')
            .map(|p| sig_str_to_hir(p.trim()))
            .collect();
        let gp_names: Vec<Symbol> = generics_reg.generic_params.iter().map(|(n, _)| *n).collect();
        let mut subst: HashMap<Symbol, HirType> = HashMap::new();
        for ((name, _), val) in generics_reg.generic_params.iter().zip(gp_values.iter()) {
            subst.insert(*name, val.clone());
        }
        // Substitute types in all methods
        let subst_methods: Vec<HirInterfaceMethod> = generics_reg.methods.iter().map(|m| {
            HirInterfaceMethod {
                name: m.name,
                self_keyword: m.self_keyword.clone(),
                params: m.params.iter().map(|(n, t)| (*n, Self::substitute_iface_type(t, &subst, &gp_names))).collect(),
                return_type: Self::substitute_iface_type(&m.return_type, &subst, &gp_names),
            }
        }).collect();
        self.interfaces.insert(*specialized_name, super::InterfaceReg {
            generic_params: vec![],
            methods: subst_methods,
        });
        Ok(())
    }
    /// Specializes methods on the fly.
    pub(crate) fn register_generic_vtable(
        &mut self,
        concrete_type: &Symbol,
        base_type: &Symbol,
        iface_name: &Symbol,
    ) -> Result<()> {
        let iface_reg = match self.interfaces.get(iface_name) {
            Some(r) => r.clone(),
            None => return Ok(()),
        };
        let mut vtable_fns: Vec<FnId> = vec![FnId(usize::MAX)];
        let span = crate::span::Span::default();
        for iface_method in &iface_reg.methods {
            let self_ty = HirType::Shared(Box::new(HirType::Named(*concrete_type)));
            let mut arg_types = vec![self_ty];
            for (_, ift) in &iface_method.params {
                arg_types.push(ift.clone());
            }
            let fid = self.specialize_generic_call(&iface_method.name, &arg_types, &span)?;
            vtable_fns.push(fid);
        }
        self.vtables.push(VtableEntry {
            concrete_type: *concrete_type,
            interface: *iface_name,
            method_fn_ids: vtable_fns,
        });
        self.type_ifaces.entry(*concrete_type).or_default().push(*iface_name);
        // Also register under base type for future lookups
        if *base_type != *concrete_type {
            self.type_ifaces.entry(*base_type).or_default().push(*iface_name);
        }
        Ok(())
    }
}
