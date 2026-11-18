use super::*;

impl Ctx {
    pub(crate) fn is_scope_shared_type(name: &Symbol) -> bool {
        matches!(name.as_str().as_str(), "String" | "Job")
    }

    /// 从 HirType 中递归收集泛型参数名（如 T，含编码名 LinkedListNode[T] 里的 T）。
    pub(crate) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>) {
        match ty {
            HirType::Named(name) => {
                let text = name.as_str();
                if text.len() == 1 && text.chars().all(|c| c.is_uppercase()) {
                    out.push(*name);
                }
                if text.contains(['<', '[']) {
                    if let Some(inner) = generic_inner(&text) {
                        for part in split_generic_args(inner) {
                            let ty = sig_str_to_hir(part.trim());
                            Self::collect_gp_from_type(&ty, out);
                        }
                    }
                }
            }
            HirType::Unique(inner) => Self::collect_gp_from_type(inner, out),
            HirType::Array(inner) => Self::collect_gp_from_type(inner, out),
            HirType::Ref(inner, _) => Self::collect_gp_from_type(inner, out),
            HirType::FatPtr { kind, .. } => Self::collect_gp_from_type(kind, out),
            _ => {}
        }
    }

    pub(crate) fn collect_scoped_type_names(&mut self, stmts: &[Stmt], prefix: &str) {
        for stmt in stmts {
            match stmt {
                Stmt::StructDef { vis, name, .. }
                | Stmt::EnumDef { vis, name, .. }
                    if !prefix.is_empty()
                        && (!prefix.starts_with(crate::package::SCOPE_PREFIX)
                            || !vis.is_public()) => {
                    self.scoped_type_names
                        .entry(Symbol::intern(prefix))
                        .or_default()
                        .insert(*name);
                }
                Stmt::InterfaceDef { name, .. }
                    if !prefix.is_empty()
                        && !prefix.starts_with(crate::package::SCOPE_PREFIX) => {
                    self.scoped_type_names
                        .entry(Symbol::intern(prefix))
                        .or_default()
                        .insert(*name);
                }
                Stmt::Namespace { name, items, .. } => {
                    let nested = if prefix.is_empty() {
                        name.as_str().to_string()
                    } else {
                        format!("{}.{}", prefix, name)
                    };
                    self.collect_scoped_type_names(items, &nested);
                }
                Stmt::Attributed { stmt, .. } => {
                    self.collect_scoped_type_names(std::slice::from_ref(stmt), prefix);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn scoped_type_symbol(&self, prefix: &str, name: Symbol) -> Symbol {
        if prefix.is_empty() || Self::is_scope_shared_type(&name) {
            return name;
        }
        let scope = Symbol::intern(prefix);
        if self.scoped_type_names.get(&scope).is_some_and(|n| n.contains(&name)) {
            Symbol::intern(&format!("{}.{}", prefix, name))
        } else {
            name
        }
    }

    pub(crate) fn ast_type_to_hir_scoped(&self, ty: &Type, module: Option<Symbol>) -> HirType {
        let hir = ast_type_to_hir(ty, &self.interfaces);
        self.qualify_hir_type(hir, module)
    }

    pub(crate) fn ast_type_to_hir_extern_scoped(&self, ty: &Type, module: Option<Symbol>) -> HirType {
        let hir = ast_type_to_hir_extern(ty, &self.interfaces);
        self.qualify_hir_type(hir, module)
    }

    pub(crate) fn ast_type_to_hir_param_scoped(
        &self,
        ty: &Type,
        extern_c: bool,
        attrs: &[crate::parser::ast::Attr],
        module: Option<Symbol>,
    ) -> HirType {
        let hir = ast_type_to_hir_param(ty, extern_c, attrs, &self.interfaces);
        self.qualify_hir_type(hir, module)
    }

    pub(crate) fn qualify_hir_type(&self, ty: HirType, module: Option<Symbol>) -> HirType {
        let Some(module) = module else { return ty };
        match ty {
            HirType::Named(name) => HirType::Named(self.qualify_named_type(module, name)),
            HirType::FatPtr { name, kind } => HirType::FatPtr {
                name: self.qualify_named_type(module, name),
                kind: Box::new(self.qualify_hir_type(*kind, Some(module))),
            },
            HirType::Unique(inner) => HirType::Unique(Box::new(
                self.qualify_hir_type(*inner, Some(module))
            )),
            HirType::Array(inner) => HirType::Array(Box::new(
                self.qualify_hir_type(*inner, Some(module))
            )),
            HirType::ArraySized(inner, n) => HirType::ArraySized(
                Box::new(self.qualify_hir_type(*inner, Some(module))), n
            ),
            HirType::Ref(inner, mutable) => HirType::Ref(
                Box::new(self.qualify_hir_type(*inner, Some(module))), mutable
            ),
            HirType::RawPtr(inner) => HirType::RawPtr(Box::new(
                self.qualify_hir_type(*inner, Some(module))
            )),
            HirType::Closure(ps, ret, owns, once) => HirType::Closure(
                ps.into_iter().map(|p| self.qualify_hir_type(p, Some(module))).collect(),
                Box::new(self.qualify_hir_type(*ret, Some(module))),
                owns,
                once,
            ),
            HirType::FnPtr(ps, ret) => HirType::FnPtr(
                ps.into_iter().map(|p| self.qualify_hir_type(p, Some(module))).collect(),
                Box::new(self.qualify_hir_type(*ret, Some(module))),
            ),
            other => other,
        }
    }

    fn qualify_named_type(&self, module: Symbol, name: Symbol) -> Symbol {
        let base = crate::hir::lower::strip_generic_name(&name);
        if Self::is_scope_shared_type(&base) {
            return name;
        }
        let mut scope = module.as_str().to_string();
        loop {
            if let Some(names) = self.scoped_type_names.get(&Symbol::intern(&scope)) {
                if names.contains(&base) {
                    let raw = name.as_str();
                    let suffix = &raw[base.as_str().len()..];
                    let qualified_base = Symbol::intern(&format!(
                        "{}.{}",
                        scope,
                        base.as_str()
                    ));
                    let qualified = if suffix.starts_with('<') && suffix.ends_with('>') {
                        let inner = &suffix[1..suffix.len() - 1];
                        let args: Vec<String> = crate::hir::lower::helpers::split_generic_args(inner)
                            .into_iter()
                            .map(|arg| {
                                let hir = crate::hir::lower::helpers::sig_str_to_hir(arg);
                                let qualified = self.qualify_hir_type(hir, Some(module));
                                type_to_string_generic(&hir_type_to_ast_type(&qualified), &self.interfaces)
                            })
                            .collect();
                        Symbol::intern(&format!("{}<{}>", qualified_base.as_str(), args.join(",")))
                    } else {
                        qualified_base
                    };
                    return qualified;
                }
                // 公共泛型容器（Mutex[PrivateStruct]）也要限定内部参数。
                let raw = name.as_str();
                let suffix = &raw[base.as_str().len()..];
                if suffix.starts_with('<') && suffix.ends_with('>') {
                    let inner = &suffix[1..suffix.len() - 1];
                    let args: Vec<String> = crate::hir::lower::helpers::split_generic_args(inner)
                        .into_iter()
                        .map(|arg| {
                            let hir = crate::hir::lower::helpers::sig_str_to_hir(arg);
                            let qualified = self.qualify_hir_type(hir, Some(module));
                            type_to_string_generic(&hir_type_to_ast_type(&qualified), &self.interfaces)
                        })
                        .collect();
                    return Symbol::intern(&format!("{}<{}>", base.as_str(), args.join(",")));
                }
            }
            let Some((parent, _)) = scope.rsplit_once('.') else { break };
            scope = parent.to_string();
        }
        name
    }

    pub(crate) fn scoped_fn_name(&self, name: Symbol) -> Symbol {
        self.scoped_fn_name_in(self.current_module, name)
    }

    pub(crate) fn scoped_fn_name_in(
        &self,
        module: Option<Symbol>,
        name: Symbol,
    ) -> Symbol {
        let Some(module) = module else { return name };
        let mut scope = module.as_str().to_string();
        loop {
            let qualified = Symbol::intern(&format!("{}.{}", scope, name.as_str()));
            if self.fn_map.contains_key(&qualified)
                || self.generic_fns.iter().any(|(n, ..)| *n == qualified)
                || self.consts.contains_key(&qualified)
                || self.statics.contains_key(&qualified)
                || self.struct_defs.contains_key(&qualified)
                || self.interfaces.contains_key(&qualified)
            {
                if std::env::var_os("AYANAMI_MODULE_TRACE").is_some() {
                    eprintln!(
                        "module trace: resolved {} -> {}",
                        name.as_str(),
                        qualified.as_str()
                    );
                }
                return qualified;
            }
            let Some((parent, _)) = scope.rsplit_once('.') else { break };
            scope = parent.to_string();
        }
        name
    }
}
