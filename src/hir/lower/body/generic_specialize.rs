use super::*;

impl crate::hir::lower::Ctx {
    /// Try to resolve a call by specializing a generic function.
    /// Returns the FnId of the newly-created specialized function on success.
    pub(crate) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId> {
        self.specialize_generic_call_with(name, arg_types, None, span)
    }

    /// #72：是否存在同名、参数个数一致且接收者基类型匹配的泛型方法候选。
    /// 用于区分「真的没有该方法」与「特化内部失败（如方法体编译错误）」。
    pub(crate) fn has_generic_method_candidate(&self, method: &Symbol, receiver_ty: &HirType, argc: usize) -> bool {
        let recv_base = match strip_ownership_ref(receiver_ty) {
            HirType::Named(n) => crate::hir::lower::strip_generic_name(n),
            _ => return false,
        };
        self.generic_fns.iter().any(|(gf_name, _, gf_stmt)| {
            if gf_name != method { return false; }
            let Stmt::FnDecl { params, .. } = gf_stmt else { return false; };
            let visible = params.len().saturating_sub(count_hidden_names(params));
            if visible != argc { return false; }
            match params.first() {
                Some((_, t)) => {
                    let self_hir = ast_type_to_hir(t, &self.interfaces);
                    matches!(strip_ownership_ref(&self_hir), HirType::Named(n)
                        if crate::hir::lower::strip_generic_name(n) == recv_base)
                }
                None => false,
            }
        })
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
                let visible = params.len().saturating_sub(count_hidden_names(params));
                if visible != arg_types.len() { continue; }
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
                // Fallback: 同名且可见参数个数一致
                if let Some(i) = self.generic_fns.iter().position(|(gf_name, _, gf_stmt)| {
                    gf_name == name && match gf_stmt {
                        Stmt::FnDecl { params, .. } => {
                            params.len().saturating_sub(count_hidden_names(params)) == arg_types.len()
                        }
                        _ => false,
                    }
                }) {
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

        let hidden = count_hidden_names(params);
        let visible = params.len().saturating_sub(hidden);
        if visible != arg_types.len() {
            return Err(Error::Hir(format!(
                "generic function `{}` takes {} argument(s) but {} given at {}:{}",
                gf_name, visible, arg_types.len(), span.start_line, span.start_col
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
            for ((_, param_ty), arg_ty) in params.iter().take(visible).zip(arg_types.iter()) {
                for (gp_name, hir_concrete) in infer_generic_from_param(param_ty, arg_ty) {
                    if generic_names.contains(&gp_name) && !generic_mappings.contains_key(&gp_name) {
                        generic_mappings.insert(gp_name, hir_concrete);
                    }
                }
            }
        }
        // #160：从参数化约束推断剩余泛型参数（I: Iterator[T]，I 已推断 → T 由实现方法反推）
        if generic_mappings.len() < gf_params.len() {
            for (gp_name, constraint) in gf_params {
                let Some(iface_sym) = constraint else { continue };
                let Some(concrete) = generic_mappings.get(gp_name).cloned() else { continue };
                let iface_base = crate::hir::lower::strip_generic_name(iface_sym);
                let iface_gp: Vec<Symbol> = match self.interfaces.get(&iface_base) {
                    Some(reg) => reg.generic_params.iter().map(|(n, _)| *n).collect(),
                    None => continue,
                };
                let sym_text = iface_sym.as_str();
                let Some(inner) = generic_inner(&sym_text) else { continue };
                let arg_texts = split_generic_args(inner);
                if arg_texts.len() != iface_gp.len() { continue; }
                let Some(iface_args) = self.infer_iface_args_for_concrete(iface_sym, &concrete) else { continue };
                for (i, text) in arg_texts.iter().enumerate() {
                    let Some(ty) = iface_args.get(&iface_gp[i]) else { continue };
                    let gp = Symbol::intern(text.trim());
                    if generic_names.contains(&gp) && !generic_mappings.contains_key(&gp) {
                        generic_mappings.insert(gp, ty.clone());
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

        // Step 1.5: Check interface constraints（#160：参数化约束先替换泛型实参再比对）
        for (gp_name, constraint) in gf_params {
            if let Some(iface_name) = constraint {
                self.check_generic_constraint(gp_name, iface_name, &generic_mappings, span)?;
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
        let hidden = count_hidden_params(&hir_params);
        let is_noreturn = matches!(hir_return, HirType::Never);
        self.fns.push(FnSig {
            name: *name,
            params: hir_params,
            return_type: hir_return,
            span: crate::span::Span::default(),
            effects: crate::hir::effects::EffectDecl::default(),
                        inferred: Default::default(),
            hidden,
            is_noreturn,
        });
        self.fn_map.entry(*name).or_default().push(fid);
        self.specialized_ids.insert(fid);

        // Step 5: Lower the specialized function body
        let saved_current_fn = self.current_fn;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::take(&mut self.scopes);

        // A3a：泛型实例继承定义上的标注（含效应；此前 A1 标注在此丢失）
        let mut hir_fn = self.lower_fn(fid, *name, &new_params, &new_return_type, &new_body, *is_inline, *extern_c, Span::default(), attrs.clone(), param_attrs.clone(), vis.is_public())?;
        hir_fn.is_specialized = true;

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
