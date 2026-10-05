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
                Stmt::ConstDecl { .. } => {} // M6.1：常量在 collect_ns 求值，无运行时项
                Stmt::ImplBlock { methods, generic_params: impl_gp, .. } => {
                    // Flatten impl block: lower each method as a regular Fn
                    for method_stmt in methods {
                        if let Stmt::FnDecl { name, params, return_type, body, generic_params, param_attrs, attrs, vis, .. } = method_stmt {
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
                            let hir_fn = self.lower_fn(fn_id, *name, params, return_type, body, false, false, method_stmt.span(), attrs.clone(), param_attrs.clone(), vis.is_public())?;
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
                if s.contains(['<', '[']) {
                    if let Some(inner) = generic_inner(&s) {
                        for part in split_generic_args(inner) {
                            let pty = sig_str_to_hir(part.trim());
                            Self::collect_gp_from_type(&pty, out);
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
        // #84 ③：`#[export]` 等价 `extern "C"` 定义（原始符号名 + C ABI）
        let extern_c = extern_c || crate::hir::attrs::has(&attrs, "export");
        // A1：`#[inline]`/`#[inline(always)]` 不再并入关键字标记，
        // 由 LIR 发射层按标注区分 inlinehint / alwaysinline。
        // A3a：解析效应注解（throws/eff；注解权威，推断在后续阶段）
        let effects = crate::hir::effects::parse(&attrs)?;
        let is_macro = attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "macro");
        let follow_sources: Vec<Symbol> = attrs.iter()
            .filter(|a| a.is_builtin() && a.name.as_str() == "follow_with")
            .flat_map(|a| a.args.iter())
            .filter_map(|arg| match arg {
                crate::parser::ast::AttrArg::Expr(e) => match e.as_ref() {
                    Expr::Ident(s, _) => Some(*s),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let saved_pending = std::mem::take(&mut self.pending_stmts);
        let saved_hints = std::mem::take(&mut self.usage_hints);
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
            prelude.push(HirStmt::Assume { cond: hir_cond, span });
        }
        // A2d：函数级 #[requires(cond)]，默认运行检查；AYANAMI_CHECKS=0 时退化为 assume
        let checks = crate::hir::contracts::checks_enabled();
        for (cond, line, col) in crate::hir::contracts::requires_conditions(&attrs) {
            let hir_cond = self.lower_expr(cond)?;
            crate::hir::contracts::ensure_bool_condition(&hir_cond, "requires", line, col)?;
            if checks {
                prelude.push(HirStmt::Contract { kind: ContractKind::Require, cond: hir_cond, line, col });
            } else {
                prelude.push(HirStmt::Assume { cond: hir_cond, span });
            }
        }

        let (mut hir_body, tail_value) = self.lower_block_impl(body, true)?;
        // 裸尾表达式：非 void 函数作为隐式返回值（coerce 到返回类型）；void 则求值丢弃
        if let Some(tail) = tail_value {
            if matches!(return_type, HirType::Void) {
                hir_body.stmts.push(HirStmt::Expr { expr: tail, span });
            } else {
                let coerced = coerce_expr(tail, &return_type, &span)?;
                hir_body.stmts.push(HirStmt::Return { value: Some(coerced), span });
            }
        }

        // A2e：后置条件注入（result 绑定返回值）
        self.inject_ensures(&mut hir_body, &attrs, &return_type, span)?;

        // `main` 缺省返回 0（C 语义）：显式 return 优先，末尾兜底
        if name.as_str() == "main" && matches!(return_type, HirType::Int) && body.tail.is_none() {
            hir_body.stmts.push(HirStmt::Return {
                value: Some(SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into()),
                span,
            });
        }

        if !prelude.is_empty() {
            prelude.append(&mut hir_body.stmts);
            hir_body.stmts = prelude;
        }

        self.pending_stmts = saved_pending;
        self.usage_hints = saved_hints;
        let locals = std::mem::take(&mut self.locals);
        Ok(HirFn {
            span,
            attrs,
            effects,
            is_pub,
            is_macro,
            follow_sources,
            inferred: crate::hir::effects::EffectSet::default(),
            param_attrs,
            fn_id,
            name,
            is_inline,
            extern_c,
            is_specialized: false,
            params: hir_params,
            return_type,
            locals,
            body: hir_body,
        })
    }
}
