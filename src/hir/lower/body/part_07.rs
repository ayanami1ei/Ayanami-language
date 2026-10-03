use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>> {
        self.lower_items_with_ns(stmts, "")
    }

    pub(crate) fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>> {
        let mut items = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, params, return_type, body, is_inline, extern_c, generic_params, span, attrs, param_attrs, vis, .. } => {
                    // Skip generic functions — they are specialized on demand
                    if !generic_params.is_empty() {
                        continue;
                    }
                    let full_name = if ns_prefix.is_empty() {
                        *name
                    } else {
                        Symbol::intern(&format!("{}.{}", ns_prefix, name))
                    };
                    let ptypes: Vec<HirType> = params.iter()
                        .map(|(_, t)| ast_type_to_hir(t, &self.interfaces))
                        .collect();
                    let fn_id = self.find_fn_by_sig(full_name, &ptypes)
                        .ok_or_else(|| Error::Hir(format!("internal error: function `{}` not found at {}:{}", full_name, span.start_line, span.start_col)))?;
                    let hir_fn = self.lower_fn(fn_id, full_name, params, return_type, body, *is_inline, *extern_c, *span, attrs.clone(), param_attrs.clone(), vis.is_public())?;
                    items.push(HirItem::Fn(hir_fn));
                }
                Stmt::Namespace { name, items: ns_items, .. } => {
                    let nested = if ns_prefix.is_empty() {
                        name.as_str().to_string()
                    } else {
                        format!("{}.{}", ns_prefix, name)
                    };
                    let inner = self.lower_items_with_ns(ns_items, &nested)?;
                    items.push(HirItem::Namespace { name: *name, items: inner });
                }
                Stmt::InterfaceDef { name, methods, generic_params, .. } => {
                    let hir_methods: Vec<HirInterfaceMethod> = methods.iter().map(|m| {
                        HirInterfaceMethod {
                            name: m.name,
                            self_keyword: m.self_keyword,
                            params: m.params.iter()
                                .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                                .collect(),
                            return_type: ast_type_to_hir(&m.return_type, &self.interfaces),
                        }
                    }).collect();
                    items.push(HirItem::InterfaceDef { name: *name, generic_params: generic_params.clone(), methods: hir_methods });
                }
                Stmt::StructDef { name, fields, .. } => {
                    let hir_fields: Vec<HirStructField> = fields.iter()
                        .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                        .collect();
                    items.push(HirItem::StructDef(HirStructDef { name: *name, fields: hir_fields }));
                }
                Stmt::EnumDef { name, variants, .. } => {
                    for variant in variants {
                        let var_struct_name = Symbol::intern(&format!("{}_{}", name, variant.name));
                        let hir_fields: Vec<HirStructField> = match &variant.fields {
                            crate::parser::ast::stmt::EnumFields::Named(fields) => {
                                fields.iter()
                                    .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                                    .collect()
                            }
                            crate::parser::ast::stmt::EnumFields::Tuple(tys) => {
                                tys.iter().enumerate()
                                    .map(|(i, t)| HirStructField { name: Symbol::intern(&format!("_{}", i)), ty: ast_type_to_hir(t, &self.interfaces) })
                                    .collect()
                            }
                            crate::parser::ast::stmt::EnumFields::None => vec![],
                        };
                        items.push(HirItem::StructDef(HirStructDef { name: var_struct_name, fields: hir_fields }));
                    }
                    // Emit enum struct
                    let mut enum_fields = vec![HirStructField { name: Symbol::intern("_tag"), ty: HirType::Int }];
                    for variant in variants {
                        let vsn = Symbol::intern(&format!("{}_{}", name, variant.name));
                        enum_fields.push(HirStructField { name: Symbol::intern(&format!("_data_{}", variant.name)), ty: HirType::Named(vsn) });
                    }
                    items.push(HirItem::StructDef(HirStructDef { name: *name, fields: enum_fields }));
                }
                Stmt::Import { .. } => {} // already handled in collect_fns
                Stmt::ImplBlock { methods, generic_params: impl_gp, .. } => {
                    // Flatten impl block: lower each method as a regular Fn
                    for method_stmt in methods {
                        if let Stmt::FnDecl { name, params, return_type, body, generic_params, param_attrs, vis, .. } = method_stmt {
                            if !generic_params.is_empty() || !impl_gp.is_empty() {
                                continue; // generic methods are lowered during specialization
                            }
                            let ptypes: Vec<HirType> = params.iter()
                                .map(|(_, t)| ast_type_to_hir(t, &self.interfaces))
                                .collect();
                            let fn_id = self.find_fn_by_sig(*name, &ptypes)
                                .ok_or_else(|| {
                                    let s = method_stmt.span();
                                    Error::Hir(format!("internal error: method `{}` not found at {}:{}", name, s.start_line, s.start_col))
                                })?;
                            let hir_fn = self.lower_fn(fn_id, *name, params, return_type, body, false, false, Span::default(), vec![], param_attrs.clone(), vis.is_public())?;
                            items.push(HirItem::Fn(hir_fn));
                        }
                    }
                }
                _ => {
                    let s = stmt.span();
                    return Err(Error::Hir(format!("unexpected top-level statement (at {}:{})", s.start_line, s.start_col)));
                }
            }
        }
        Ok(items)
    }

    /// 从 HirType 中递归收集泛型参数名（如 T，含编码名 LinkedListNode[T] 里的 T）
    pub(crate) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>) {
        match ty {
            HirType::Named(n) => {
                let s = n.as_str();
                // 裸泛型参数名：T
                if s.len() == 1 && s.chars().all(|c| c.is_uppercase()) {
                    out.push(*n);
                }
                // 编码名中的泛型参数：LinkedListNode[T] → T
                let open = s.find('<').or_else(|| s.find('['));
                if let Some(start) = open {
                    let inner = s[start..].trim_start_matches('<').trim_start_matches('[')
                        .trim_end_matches('>').trim_end_matches(']');
                    for part in inner.split(',') {
                        let trimmed = part.trim();
                        if trimmed.len() == 1 && trimmed.chars().all(|c| c.is_uppercase()) {
                            out.push(Symbol::intern(trimmed));
                        }
                    }
                }
            }
            HirType::Unique(inner) => {
                Self::collect_gp_from_type(inner, out);
            }
            HirType::Array(inner) => Self::collect_gp_from_type(inner, out),
            HirType::Ref(inner, _) => Self::collect_gp_from_type(inner, out),
            HirType::FatPtr { kind, .. } => Self::collect_gp_from_type(kind, out),
            _ => {}
        }
    }

    pub(crate) fn lower_fn(
        &mut self,
        fn_id: FnId,
        name: Symbol,
        ast_params: &[(Symbol, Type)],
        _return_type: &Type,
        body: &Block,
        is_inline: bool,
        extern_c: bool,
        span: Span,
        attrs: Vec<crate::parser::ast::Attr>,
        param_attrs: Vec<Vec<crate::parser::ast::Attr>>,
        is_pub: bool,
    ) -> Result<HirFn> {
        // A1：`#[inline]`/`#[inline(always)]` 不再并入关键字标记，
        // 由 LIR 发射层按标注区分 inlinehint / alwaysinline。
        // A3a：解析效应注解（throws/eff；注解权威，推断在后续阶段）
        let effects = crate::hir::effects::parse(&attrs)?;
        let saved_pending = std::mem::take(&mut self.pending_stmts);
        self.current_fn = fn_id;
        self.locals = Vec::new();
        self.scopes = Vec::new();

        let sig = &self.fns[fn_id.0];
        let return_type = sig.return_type.clone();
        // 泛型单态化：签名类型先实例化
        self.instantiate_type(&return_type)?;

        // Push global scope for params
        self.push_scope();

        let mut hir_params = Vec::new();
        for (param_name, param_type) in ast_params {
            let hir_ty = ast_type_to_hir(param_type, &self.interfaces);
            self.instantiate_type(&hir_ty)?;
            let var_id = VarId(self.locals.len());
            self.locals.push(HirLocal::new(*param_name, hir_ty.clone(), false));
            self.bind_var(*param_name, var_id, hir_ty.clone(), false);
            hir_params.push((*param_name, hir_ty));
        }

        // A2c：函数级 #[assume(cond)] 前置于函数体（发射为 llvm.assume）
        let mut prelude: Vec<HirStmt> = Vec::new();
        for cond in crate::hir::contracts::assume_conditions(&attrs) {
            let hir_cond = self.lower_expr(cond)?;
            crate::hir::contracts::ensure_bool_condition(
                &hir_cond, "assume", span.start_line, span.start_col)?;
            prelude.push(HirStmt::Assume(hir_cond));
        }
        // A2d：函数级 #[requires(cond)]，默认运行检查；AYANAMI_CHECKS=0 时退化为 assume
        let checks = crate::hir::contracts::checks_enabled();
        for (cond, line, col) in crate::hir::contracts::requires_conditions(&attrs) {
            let hir_cond = self.lower_expr(cond)?;
            crate::hir::contracts::ensure_bool_condition(&hir_cond, "requires", line, col)?;
            if checks {
                prelude.push(HirStmt::Contract { kind: ContractKind::Require, cond: hir_cond, line, col });
            } else {
                prelude.push(HirStmt::Assume(hir_cond));
            }
        }

        let mut hir_body = self.lower_block(body)?;

        // A2e：后置条件注入（result 绑定返回值）
        self.inject_ensures(&mut hir_body, &attrs, &return_type, span)?;

        if !prelude.is_empty() {
            prelude.append(&mut hir_body.stmts);
            hir_body.stmts = prelude;
        }

        self.pending_stmts = saved_pending;
        let locals = std::mem::take(&mut self.locals);
        Ok(HirFn {
            span,
            attrs,
            effects,
            is_pub,
            inferred: crate::hir::effects::EffectSet::default(),
            param_attrs,
            fn_id,
            name,
            is_inline,
            extern_c,
            params: hir_params,
            return_type,
            locals,
            body: hir_body,
        })
    }

    // ----------------------------------------------------------------
    //  块/语句降级：lower_block → lower_stmt → lower_for
    // ----------------------------------------------------------------

    pub(crate) fn lower_block(&mut self, block: &Block) -> Result<HirBlock> {
        self.push_scope();
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            let mark = self.pending_stmts.len();
            let lowered = self.lower_stmt(stmt)?;
            // A3d：表达式内联语句（如 `?`）必须先于本语句执行
            let pending: Vec<HirStmt> = self.pending_stmts.split_off(mark);
            stmts.extend(pending);
            stmts.push(lowered);
        }
        self.pop_scope();
        Ok(HirBlock::new(stmts))
    }
}
