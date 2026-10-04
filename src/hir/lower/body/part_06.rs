use super::*;

impl crate::hir::lower::Ctx {
    /// Try to resolve a call by specializing a generic function.
    /// Returns the FnId of the newly-created specialized function on success.
    pub(crate) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId> {
        self.specialize_generic_call_with(name, arg_types, None, span)
    }

    /// `explicit`：显式泛型实参（优先于形参推导）
    pub(crate) fn specialize_generic_call_with(&mut self, name: &Symbol, arg_types: &[HirType], explicit: Option<&Vec<Type>>, span: &crate::span::Span) -> Result<FnId> {
        // Find matching generic function — prefer one where self's base type matches
        let arg_base = (!arg_types.is_empty()).then(|| {
            match strip_ownership_ref(&arg_types[0]) {
                HirType::Named(n) => Some(crate::hir::lower::strip_generic_name(n)),
                _ => None,
            }
        }).flatten();

        let mut best_gf_idx = None;
        for (i, (gf_name, _, gf_stmt)) in self.generic_fns.iter().enumerate() {
            if gf_name != name { continue; }
            if let Stmt::FnDecl { params, .. } = gf_stmt {
                if params.len() != arg_types.len() { continue; }
                if let Some(ab) = &arg_base {
                    if let Some(first) = params.first() {
                        let self_hir = ast_type_to_hir(&first.1, &self.interfaces);
                        let self_base = match strip_ownership_ref(&self_hir) {
                            HirType::Named(n) => crate::hir::lower::strip_generic_name(n),
                            _ => continue,
                        };
                        if self_base != *ab { continue; }
                    }
                }
                best_gf_idx = Some(i);
                break;
            }
        }

        let gf_idx = match best_gf_idx {
            Some(i) => i,
            None => {
                // Fallback: just find by name
                if let Some(i) = self.generic_fns.iter().position(|(gf_name, _, _)| gf_name == name) {
                    i
                } else {
                    return Err(if self.fn_map.contains_key(name) {
                        let ats: Vec<String> = arg_types.iter().map(hir_type_display).collect();
                        Error::Hir(format!("no matching overload of `{}` for argument types ({}); candidate(s) exist at {}:{}",
                            name, ats.join(", "), span.start_line, span.start_col))
                    } else {
                        Error::Hir(format!("undefined function `{}` at {}:{}", name, span.start_line, span.start_col))
                    });
                }
            }
        };

        let (gf_name, gf_params, gf_stmt) = &self.generic_fns[gf_idx];
        let Stmt::FnDecl { params, return_type, body, is_inline, extern_c, param_attrs, attrs, vis, .. } = gf_stmt else {
            return Err(Error::Hir(format!("internal error: generic function `{}` is not a FnDecl at {}:{}", gf_name, span.start_line, span.start_col)));
        };

        if params.len() != arg_types.len() {
            return Err(Error::Hir(format!(
                "generic function `{}` takes {} argument(s) but {} given at {}:{}",
                gf_name, params.len(), arg_types.len(), span.start_line, span.start_col
            )));
        }

        // Step 1: 显式泛型实参优先，否则按形参推导
        let generic_names: Vec<Symbol> = gf_params.iter().map(|(n, _)| *n).collect();
        let mut generic_mappings: HashMap<Symbol, HirType> = HashMap::new();
        if let Some(types) = explicit {
            if types.len() != gf_params.len() {
                return Err(Error::Hir(format!(
                    "generic function `{}` expects {} type argument(s), found {} at {}:{}",
                    gf_name, gf_params.len(), types.len(), span.start_line, span.start_col
                )));
            }
            for ((gp_name, _), ty) in gf_params.iter().zip(types.iter()) {
                generic_mappings.insert(*gp_name, ast_type_to_hir(ty, &self.interfaces));
            }
        } else {
            for ((_, param_ty), arg_ty) in params.iter().zip(arg_types.iter()) {
                let result = infer_generic_from_param(param_ty, arg_ty);
                if let Some((gp_name, hir_concrete)) = result {
                    if generic_names.contains(&gp_name) && !generic_mappings.contains_key(&gp_name) {
                        generic_mappings.insert(gp_name, hir_concrete.clone());
                    }
                }
            }
        }
        // Ensure all generic params were resolved
        for (gp_name, _) in gf_params {
            if !generic_mappings.contains_key(gp_name) {
                return Err(Error::Hir(format!(
                    "cannot infer generic parameter `{}` for function `{}` at {}:{}",
                    gp_name, gf_name, span.start_line, span.start_col
                )));
            }
        }

        // Step 1.5: Check interface constraints
        for (gp_name, constraint) in gf_params {
            if let Some(iface_name) = constraint {
                let concrete_ty = generic_mappings.get(gp_name)
                    .ok_or_else(|| Error::Hir(format!("internal error: generic param `{}` not resolved at {}:{}", gp_name, span.start_line, span.start_col)))?;
                // 递归剥离所有权包装（Unique/Shared/Weak），
                // 处理多层包装如 Unique(Shared(LinkedList)) → LinkedList
                let mut concrete_inner = concrete_ty;
                while matches!(concrete_inner, HirType::Unique(_)) {
                    concrete_inner = strip_ownership_ref(concrete_inner);
                }
                let concrete_type_name = match concrete_inner {
                    HirType::Named(n) => {
                        let base = crate::hir::lower::strip_generic_name(n);
                        if self.type_ifaces.contains_key(n) { *n }
                        else { base }
                    }
                    HirType::FatPtr { name, .. } => {
                        // FatPtr 是接口类型（如 shared List）。
                        // 检查该接口是否包含了约束接口的所有方法。
                        let iface_methods = self.interfaces.get(iface_name)
                            .map(|reg| reg.methods.iter().map(|m| m.name).collect::<Vec<_>>())
                            .unwrap_or_default();
                        let all_ok = iface_methods.iter().all(|method_name| {
                            self.interfaces.get(name)
                                .map(|reg| reg.methods.iter().any(|m| m.name == *method_name))
                                .unwrap_or(false)
                        });
                        if !all_ok {
                            return Err(Error::Hir(format!(
                                "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                                hir_type_display(concrete_ty), iface_name, gp_name,
                                span.start_line, span.start_col
                            )));
                        }
                        // 直接标记为已实现，跳过后续 type_ifaces 检查
                        continue;
                    }
                    HirType::Int => Symbol::intern("int"),
                    HirType::Float => Symbol::intern("float"),
                    HirType::Char => Symbol::intern("char"),
                    HirType::Bool => Symbol::intern("bool"),
                    HirType::Array(_) => Symbol::intern("[int]"), // simplified
                    _ => return Err(Error::Hir(format!(
                        "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                        hir_type_display(concrete_ty), iface_name, gp_name,
                        span.start_line, span.start_col
                    ))),
                };
                let implements = self.type_ifaces.get(&concrete_type_name)
                    .map(|ifaces| {
                        ifaces.contains(iface_name)
                            || ifaces.iter().any(|name| {
                                let s = name.as_str();
                                s.starts_with(&*iface_name.as_str()) && s.contains('<')
                            })
                    })
                    .unwrap_or(false);
                if !implements {
                    // 检查泛型方法是否实现了接口要求的方法
                    let iface_methods = self.interfaces.get(iface_name)
                        .map(|reg| reg.methods.iter().map(|m| m.name).collect::<Vec<_>>())
                        .unwrap_or_default();
                    let has_matching_method = iface_methods.iter().any(|method_name| {
                        self.generic_fns.iter().any(|(gf_name, _, _)| gf_name == method_name)
                        || self.fn_map.contains_key(method_name)
                    });
                    if !has_matching_method {
                        return Err(Error::Hir(format!(
                            "type `{}` does not implement interface `{}` required by generic parameter `{}` at {}:{}",
                            hir_type_display(concrete_ty), iface_name, gp_name,
                            span.start_line, span.start_col
                        )));
                    }
                }
            }
        }

        // Step 2: Build substitution map from generic Symbol -> AST Type
        let substitutions: HashMap<Symbol, Type> = generic_mappings.iter()
            .map(|(k, v)| (*k, hir_type_to_ast_type(v)))
            .collect();

        // Step 3: Clone and substitute types in the AST
        let new_params: Vec<(Symbol, Type)> = params.iter()
            .map(|(n, t)| (*n, substitute_type_in_type(t, &substitutions)))
            .collect();
        let new_return_type = substitute_type_in_type(return_type, &substitutions);
        let new_body = substitute_type_in_block(body, &substitutions);

        // Step 4: Create specialized function signature and register it
        let hir_return = ast_type_to_hir(&new_return_type, &self.interfaces);
        let hir_params: Vec<(Symbol, HirType)> = new_params.iter()
            .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
            .collect();



        // 去重：检查是否已存在相同签名的特化函数
        if let Some(existing) = self.fn_map.get(name).and_then(|ids| {
            ids.iter().find(|id| {
                let s = &self.fns[id.0];
                s.params == hir_params && s.return_type == hir_return
            })
        }) {
            return Ok(*existing);
        }

        let fid = FnId(self.fns.len());
        self.fns.push(FnSig {
            name: *name,
            params: hir_params,
            return_type: hir_return,
            span: crate::span::Span::default(),
            effects: crate::hir::effects::EffectDecl::default(),
                        inferred: Default::default(),
        });
        self.fn_map.entry(*name).or_default().push(fid);
        self.specialized_ids.insert(fid);

        // Step 5: Lower the specialized function body
        let saved_current_fn = self.current_fn;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::take(&mut self.scopes);

        // A3a：泛型实例继承定义上的标注（含效应；此前 A1 标注在此丢失）
        let hir_fn = self.lower_fn(fid, *name, &new_params, &new_return_type, &new_body, *is_inline, *extern_c, Span::default(), attrs.clone(), param_attrs.clone(), vis.is_public())?;

        self.current_fn = saved_current_fn;
        self.locals = saved_locals;
        self.scopes = saved_scopes;

        self.specialized_fns.push(hir_fn);

        Ok(fid)
    }

    // ----------------------------------------------------------------
    //  阶段 2：降级顶层项（函数、结构体、命名空间等）
    //  将 AST 中声明级别的节点递归降级为 HIR 节点
    // ----------------------------------------------------------------
}
