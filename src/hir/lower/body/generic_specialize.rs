use super::*;

impl crate::hir::lower::Ctx {
    /// Try to resolve a call by specializing a generic function.
    /// Returns the FnId of the newly-created specialized function on success.
    pub(crate) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId> {
        self.specialize_generic_call_with(name, arg_types, None, span, false)
    }

    /// bd 8yn：方法调用特化——仅考虑 impl 泛型方法候选（同名自由泛型函数不抢占）
    pub(crate) fn specialize_generic_method_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId> {
        self.specialize_generic_call_with(name, arg_types, None, span, true)
    }

    /// #72：是否存在同名、参数个数一致且接收者基类型匹配的泛型方法候选。
    /// 用于区分「真的没有该方法」与「特化内部失败（如方法体编译错误）」。
    pub(crate) fn has_generic_method_candidate(&self, method: &Symbol, receiver_ty: &HirType, argc: usize) -> bool {
        let recv_base = match strip_ownership_ref(receiver_ty) {
            HirType::Named(n) => crate::hir::lower::strip_generic_name(n),
            _ => return false,
        };
        self.generic_fns.iter().any(|(gf_name, _, gf_stmt, is_method)| {
            if gf_name != method || !*is_method { return false; }
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
    pub(crate) fn specialize_generic_call_with(&mut self, name: &Symbol, arg_types: &[HirType], explicit: Option<&Vec<Type>>, span: &crate::span::Span, prefer_method: bool) -> Result<FnId> {
        // Find matching generic function — prefer one where self's base type matches
        let arg_base = (!arg_types.is_empty()).then(|| {
            match strip_ownership_ref(&arg_types[0]) {
                HirType::Named(n) => Some(crate::hir::lower::strip_generic_name(n)),
                _ => None,
            }
        }).flatten();

        let mut best_gf_idx = None;
        // #189：首参是候选自身泛型参数（如自由函数 map 的 I）时可绑定任意接收者，
        // 作为次级候选——精确基名匹配（如 Option.map）优先，避免自由调用被同名方法抢占
        let mut gp_self_idx = None;
        for (i, (gf_name, gf_params, gf_stmt, gf_is_method)) in self.generic_fns.iter().enumerate() {
            if gf_name != name || (prefer_method && !*gf_is_method) { continue; }
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
                        if self_base != *ab {
                            let is_gp = gf_params.iter().any(|(p, _)| *p == self_base);
                            if is_gp && gp_self_idx.is_none() { gp_self_idx = Some(i); }
                            continue;
                        }
                    }
                }
                best_gf_idx = Some(i);
                break;
            }
        }
        let best_gf_idx = best_gf_idx.or(gp_self_idx);

        let gf_idx = match best_gf_idx {
            Some(i) => i,
            None => {
                // Fallback: 同名且可见参数个数一致
                if let Some(i) = self.generic_fns.iter().position(|(gf_name, _, gf_stmt, gf_is_method)| {
                    gf_name == name && (!prefer_method || *gf_is_method) && match gf_stmt {
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

        let (gf_name, gf_params, gf_stmt, _gf_is_method) = &self.generic_fns[gf_idx];
        let Stmt::FnDecl { params, return_type, body, is_inline, extern_c, is_unsafe, param_attrs, attrs, vis, .. } = gf_stmt else {
            return Err(Error::Hir(format!("internal error: generic function `{}` is not a FnDecl at {}:{}", gf_name, span.start_line, span.start_col)));
        };

        let hidden = count_hidden_names(params);
        let visible = params.len().saturating_sub(hidden);
        // E1a：环境需求（泛型 bound 基名命中调用方提供者键）
        let env_needs = self.env_need_caps(gf_params);
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
                for (gp_name, hir_concrete) in infer_generic_from_param(param_ty, arg_ty, &self.interfaces) {
                    if !generic_names.contains(&gp_name) { continue; }
                    match generic_mappings.get(&gp_name) {
                        None => { generic_mappings.insert(gp_name, hir_concrete); }
                        Some(prev) if types_infer_conflict(prev, &hir_concrete) => {
                            return Err(Error::Hir(format!(
                                "generic parameter `{}` of `{}` inferred as `{}` from earlier argument(s) but `{}` here (at {}:{})",
                                gp_name.as_str(), gf_name.as_str(),
                                hir_type_display(prev), hir_type_display(&hir_concrete),
                                span.start_line, span.start_col
                            )));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        // #160：从参数化约束推断剩余泛型参数（I: Iterator[T]，I 已推断 → T 由实现方法反推）
        if generic_mappings.len() < gf_params.len() {
            self.infer_from_iface_constraints(gf_params, &generic_names, &mut generic_mappings);
        }
        // E1a：环境需求绑定 E（来自调用点提供值的类型）
        for (cap, gp) in &env_needs {
            if generic_mappings.contains_key(gp) { continue; }
            if let Some((_, ty)) = self.pending_env_args.iter().find(|(c, _)| c == cap) {
                generic_mappings.insert(*gp, ty.clone());
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
        for (gp_name, constraints) in gf_params {
            for iface_name in constraints {
                self.check_generic_constraint(gp_name, iface_name, &generic_mappings, span)?;
            }
        }

        // Step 2: Build substitution map from generic Symbol -> AST Type
        let substitutions: HashMap<Symbol, Type> = generic_mappings.iter()
            .map(|(k, v)| (*k, hir_type_to_ast_type(v)))
            .collect();

        // Step 3: Clone and substitute types in the AST
        let mut new_params: Vec<(Symbol, Type)> = params.iter()
            .map(|(n, t)| (*n, substitute_type_in_type(t, &substitutions)))
            .collect();
        // E1a：注入环境值形参（体降级据此注册局部）
        self.inject_env_params(&mut new_params, &env_needs, &generic_mappings);
        let new_return_type = substitute_type_in_type(return_type, &substitutions);
        let new_body = substitute_type_in_block(body, &substitutions);

        // Step 4: Create specialized function signature and register it
        let hir_return = ast_type_to_hir(&new_return_type, &self.interfaces);
        let hir_params: Vec<(Symbol, HirType)> = new_params.iter()
            .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
            .collect();
        // E1a：环境值参数已在 new_params 注入（hir_params 随之包含）
        // 特化名：附加泛型实参（`sz[T]() -> int` 等 T 无关签名两次特化必须区分，
        // 否则会误复用首个特化并生成同名符号）
        let args_suffix: String = gf_params.iter()
            .filter_map(|(gp, _)| generic_mappings.get(gp))
            .map(crate::lir::lower::type_to_mangle)
            .collect::<Vec<_>>()
            .join("_");
        let specialized_name = Symbol::intern(&format!("{}${}", name.as_str(), args_suffix));

        // 去重：签名 + 特化名（含泛型实参）都一致才复用
        if let Some(existing) = self.fn_map.get(name).and_then(|ids| {
            ids.iter().find(|id| {
                let s = &self.fns[id.0];
                s.name == specialized_name
                    && s.params == hir_params && s.return_type == hir_return
            })
        }) {
            return Ok(*existing);
        }

        let fid = FnId(self.fns.len());
        let hidden = count_hidden_params(&hir_params);
        let is_noreturn = matches!(hir_return, HirType::Never);
        self.fns.push(FnSig {
            name: specialized_name,
            params: hir_params,
            return_type: hir_return,
            span: crate::span::Span::default(),
            effects: crate::hir::effects::EffectDecl::default(),
                        inferred: Default::default(),
            hidden,
            is_noreturn,
            extern_c: false,
            is_unsafe: false,
        });
        self.fn_map.entry(*name).or_default().push(fid);
        self.specialized_ids.insert(fid);

        // Step 5: Lower the specialized function body
        let saved_current_fn = self.current_fn;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::take(&mut self.scopes);

        // A3a：泛型实例继承定义上的标注（含效应；此前 A1 标注在此丢失）
        let lowered = self.lower_fn(fid, *name, &new_params, &new_return_type, &new_body, *is_inline, *extern_c, *is_unsafe, Span::default(), attrs.clone(), param_attrs.clone(), vis.is_public());
        let mut hir_fn = match lowered {
            Ok(f) => f,
            Err(e) => {
                // 回滚注册：否则后续重试/解析会命中去重返回“僵尸签名”，
                // 调用引用无函数体（链接失败）且真实错误被吞掉
                if let Some(ids) = self.fn_map.get_mut(name) {
                    ids.retain(|x| *x != fid);
                }
                self.specialized_ids.remove(&fid);
                if self.fns.len() == fid.0 + 1 {
                    self.fns.pop();
                }
                // E1a 修复：失败路径同样恢复当前函数（此前泄漏到调用方，
                // 导致后续调用的环境作用域错误）
                self.current_fn = saved_current_fn;
                self.locals = saved_locals;
                self.scopes = saved_scopes;
                return Err(e);
            }
        };
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

/// 泛型实参推断冲突：两侧都具体且类别不同。
/// 数值互转与所有权包装差异放行（交由后续 coerce），单字母大写未解参数放行。
fn types_infer_conflict(a: &HirType, b: &HirType) -> bool {
    let a = strip_ownership_ref(a);
    let b = strip_ownership_ref(b);
    if a == b {
        return false;
    }
    let numeric = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { .. } | HirType::Float | HirType::F32);
    if numeric(a) && numeric(b) {
        return false;
    }
    let bare_param = |t: &HirType| matches!(t, HirType::Named(n)
        if n.as_str().len() == 1 && n.as_str().chars().all(|c| c.is_uppercase()));
    if bare_param(a) || bare_param(b) {
        return false;
    }
    // 一侧为命名类型、另一侧不是 → 冲突（如 `int` vs `String`）；命名类型之间不同也冲突
    matches!((a, b), (HirType::Named(_), HirType::Named(_)))
        || matches!(a, HirType::Named(_)) != matches!(b, HirType::Named(_))
}
