use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn collect_import(&mut self, path: &String, _ns_prefix: &str, stmt_span: crate::span::Span) -> Result<()> {
            let pkg_path = if std::path::Path::new(path).exists() {
                path.clone()
            } else {
                // Try with .lcl extension
                let with_lcl = format!("{}.lcl", path);
                if std::path::Path::new(&with_lcl).exists() {
                    with_lcl
                } else {
                    // Try with .aya extension
                    let with_aya = if path.ends_with(".aya") {
                        path.to_string()
                    } else {
                        format!("{}.aya", path)
                    };
                    if std::path::Path::new(&with_aya).exists() {
                        with_aya
                    } else {
                        // Try standard library directory (use filename only, strip any directory prefix)
                        let exe = std::env::current_exe().ok();
                        let mut found = path.clone();
                        if let Some(exe_dir) = exe.and_then(|p| p.parent().map(|d| d.to_path_buf())) {
                            let stem = std::path::Path::new(path).file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(path);
                            let std_candidates = [
                                exe_dir.join("std").join(format!("{}.lcl", stem)),
                                exe_dir.join("../std").join(format!("{}.lcl", stem)),
                                exe_dir.join("std").join(format!("{}.aya", stem)),
                                exe_dir.join("../std").join(format!("{}.aya", stem)),
                            ];
                            for candidate in &std_candidates {
                                if candidate.exists() {
                                    found = candidate.to_string_lossy().into_owned();
                                    break;
                                }
                            }
                        }
                        found
                    }
                }
            };
            let (imported_syms, sources, lir_binary, _) = crate::package::load_package(&pkg_path)
                .map_err(|e| Error::Import(format!("import error for '{}': {}", path, e)))?;

            // Merge struct definitions from the package's LIR data
            if !lir_binary.is_empty() {
                let dep_lir = crate::lir::serialize::program_from_bytes(&lir_binary);
                if let Ok(dep_lir) = dep_lir {
                    // 先保存 generic_struct_params（需在 struct_defs 被消费前读取）
                    let gsp_from_lir: HashMap<Symbol, Vec<(Symbol, Vec<Symbol>)>> =
                        dep_lir.generic_struct_params.clone();
                    for (name, fields) in dep_lir.struct_defs {
                        let hir_fields: Vec<HirStructField> = fields.iter()
                            .map(|(fn_name, ty)| HirStructField {
                                name: *fn_name,
                                // M2：旧包 struct 字段里的裸 `fn` 类型统一为 `Fn`
                                ty: crate::hir::lower::helpers::unify_legacy_fn_type(ty),
                            })
                            .collect();
                        self.struct_defs.insert(name, hir_fields);
                        // 字段扫描：从字段类型中推断泛型参数名
                        if !self.generic_struct_params.contains_key(&name) {
                            let mut gp_names: Vec<Symbol> = Vec::new();
                            for (_, fty) in &fields {
                                Self::collect_gp_from_type(fty, &mut gp_names);
                            }
                            if !gp_names.is_empty() {
                                gp_names.sort();
                                gp_names.dedup();
                                self.generic_struct_params.insert(name,
                                    gp_names.into_iter().map(|n| (n, Vec::new())).collect());
                            }
                        }
                    }
                    // 从 LIR binary 恢复 generic_struct_params（覆盖字段扫描结果）
                    for (gsp_name, gsp_params) in &gsp_from_lir {
                        self.generic_struct_params.insert(*gsp_name, gsp_params.clone());
                    }
                }
            }

            for src in &sources {
                let mut lexer = crate::lexer::Lexer::new(src);
                let tokens = lexer.tokenize_all();
                let filtered: Vec<_> = tokens.into_iter()
                    .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
                    .collect();
                if filtered.is_empty() { continue; }
                let mut parser = crate::parser::Parser::new(filtered);
                // M2：旧包源码可能仍写 `fn(...)` 类型；导入侧统一降级为 `Fn`
                parser.allow_legacy_fn_types();
                let parsed = match parser.parse_program() {
                    Ok(p) => p,
                    Err(_) => { continue; }
                };
                    for stmt in &parsed.stmts {
                        match stmt {
                            Stmt::FnDecl { name, generic_params, span, .. } if !generic_params.is_empty() => {
                                super::collect_ns::check_unique_generic_params(generic_params, span)?;
                                self.generic_fns.push((*name, generic_params.clone(), stmt.clone()));
                            }
                            Stmt::ImplBlock { methods, generic_params: impl_gp, span, .. } => {
                                super::collect_ns::check_unique_generic_params(impl_gp, span)?;
                                for m in methods {
                                    if let Stmt::FnDecl { name, generic_params, span: m_span, .. } = m {
                                        let combined: Vec<(Symbol, Vec<Symbol>)> = {
                                            let mut all = impl_gp.clone();
                                            all.extend(generic_params.iter().cloned());
                                            all
                                        };
                                        super::collect_ns::check_unique_generic_params(&combined, m_span)?;
                                        if !combined.is_empty() {
                                            self.generic_fns.push((*name, combined, m.clone()));
                                        }
                                    }
                                }
                            }
                            Stmt::Namespace { name, items, .. } => {
                                let prefix = name.as_str();
                                for it in items {
                                    if let Stmt::FnDecl { name: fn_name, generic_params, span, .. } = it {
                                        super::collect_ns::check_unique_generic_params(generic_params, span)?;
                                        if !generic_params.is_empty() {
                                            let full = Symbol::intern(&format!("{}.{}", prefix, fn_name));
                                            self.generic_fns.push((full, generic_params.clone(), it.clone()));
                                        }
                                    }
                                }
                            }
                            Stmt::InterfaceDef { name, methods, generic_params, span, .. } => {
                                super::collect_ns::check_unique_generic_params(generic_params, span)?;
                                let hir_methods = methods.iter().map(|m| {
                                    crate::hir::ir::HirInterfaceMethod {
                                        name: m.name,
                                        self_keyword: m.self_keyword,
                                        params: m.params.iter()
                                            .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                                            .collect(),
                                        return_type: ast_type_to_hir(&m.return_type, &self.interfaces),
                                    }
                                }).collect();
                                self.interfaces.insert(*name, InterfaceReg { generic_params: generic_params.clone(), methods: hir_methods });
                            }
                            _ => {}
                        }
                    }
            }
            for sym in &imported_syms {
                match sym {
                    crate::package::ImportedSymbol::Macro { name } => {
                        let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                        self.imported_macros.entry(stem).or_default().push(Symbol::intern(name));
                        // A5c-2：函数宏 `#name(args)` 解析用（裸名 + 全限定 pkg.name）
                        let lcl = path.clone();
                        self.imported_macro_lcls.entry(Symbol::intern(name)).or_default().push(lcl.clone());
                        let qualified = format!("{}.{}", stem.as_str(), name);
                        self.imported_macro_lcls.entry(Symbol::intern(&qualified)).or_default().push(lcl);
                    }
                    crate::package::ImportedSymbol::Pass { name } => {
                        let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                        self.imported_passes.entry(stem).or_default().push(Symbol::intern(name));
                    }
                    crate::package::ImportedSymbol::Check { name } => {
                        let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                        self.imported_checks.entry(stem).or_default().push(Symbol::intern(name));
                    }
                    crate::package::ImportedSymbol::Fn { name, sig, flags } => {
                        // sig format: "fnName(param_types...)->ret_type"
                        let sig_body = sig.trim_start_matches(name.as_str());
                        // M2：按配对的 ')' 找参数表结束（形参可能含 `Fn(...)` 内层括号）
                        let body = sig_body.strip_prefix('(').unwrap_or(sig_body);
                        let mut depth = 1i32;
                        let mut close = None;
                        for (i, c) in body.char_indices() {
                            match c {
                                '(' => depth += 1,
                                ')' => {
                                    depth -= 1;
                                    if depth == 0 { close = Some(i); break; }
                                }
                                _ => {}
                            }
                        }
                        let close = close.ok_or_else(|| Error::Hir(format!(
                            "invalid fn sig in package '{}': sig body `{}` (at {}:{})",
                            name, sig_body, stmt_span.start_line, stmt_span.start_col
                        )))?;
                        let params_str = &body[..close];
                        let after = body[close + 1..].trim_start();
                        let ret_str = after.strip_prefix("->").ok_or_else(|| Error::Hir(format!(
                            "invalid fn sig in package '{}': sig body `{}` (at {}:{})",
                            name, sig_body, stmt_span.start_line, stmt_span.start_col
                        )))?;
                        // params_str is "type1,type2"（已去外层括号）
                        let param_tys: Vec<&str> = if params_str.is_empty() {
                            Vec::new()
                        } else {
                            crate::hir::lower::helpers::split_generic_args(params_str)
                        };
                        let hir_params: Vec<(Symbol, HirType)> = param_tys.iter()
                            .map(|s| (Symbol::intern(""), sig_str_to_hir(s)))
                            .collect();
                        let hir_ret = sig_str_to_hir(ret_str);
                        // Dedup: skip if a FnSig with same name and param types already exists
                        let sym_name = Symbol::intern(name);
                        let already = self.fns.iter().any(|existing| {
                            existing.name == sym_name
                                && existing.params.len() == hir_params.len()
                                && existing.params.iter().zip(&hir_params).all(|(a, b)| a.1 == b.1)
                        });
                        if already {
                            continue;
                        }
                        let fn_id = FnId(self.fns.len());
                        let summary = crate::hir::effects::EffectSummary::from_tokens(flags);
                        let hidden = if flags.iter().any(|f| f == "caller") { 3 } else { 0 };
                        let is_noreturn = flags.iter().any(|f| f == "noreturn")
                            || matches!(hir_ret, HirType::Never);
                        let extern_c = flags.iter().any(|f| f == "extern");
                        self.fns.push(FnSig {
                            name: sym_name,
                            params: hir_params,
                            return_type: hir_ret,
                            effects: summary.declared,
                            inferred: summary.inferred,
                            span: crate::span::Span::default(),
                            hidden,
                            is_noreturn,
                            extern_c,
                        });
                        if extern_c { self.extern_fn_ids.insert(fn_id); }
                        self.fn_map.entry(sym_name).or_default().push(fn_id);
                    }
                    crate::package::ImportedSymbol::Const { name, ty, value } => {
                        // M6.1b：导入常量注册（限定名 `pkg.NAME` + 裸名；本地同名优先）
                        if let Some((hir_ty, lit)) = crate::package::const_codec::decode(ty, value) {
                            let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                            let qualified = Symbol::intern(&format!("{}.{}", stem.as_str(), name));
                            self.consts.entry(qualified).or_insert((hir_ty.clone(), lit.clone()));
                            self.consts.entry(Symbol::intern(name)).or_insert((hir_ty, lit));
                        }
                    }
                    crate::package::ImportedSymbol::Static { name, ty, is_mut } => {
                        // M6.2b：导入全局（裸名 + 限定名别名 `pkg.NAME`，都指向导出符号；
                        // 只发射 `external global` 声明，不定义）
                        if let Some(hir_ty) = crate::package::const_codec::type_from_str(ty) {
                            let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                            let bare = Symbol::intern(name);
                            let qualified = Symbol::intern(&format!("{}.{}", stem.as_str(), name));
                            let mk = |sym: Symbol| crate::hir::HirStatic {
                                name: sym, ty: hir_ty.clone(), value: crate::hir::ir::HirLiteral::Int(0),
                                is_mut: *is_mut, is_pub: true, is_external: true,
                            };
                            self.statics.entry(bare).or_insert_with(|| mk(bare));
                            self.statics.entry(qualified).or_insert_with(|| mk(bare));
                        }
                    }
                    crate::package::ImportedSymbol::Struct { name } => {
                        // Parse "Name(field1:type1,field2:type2)" format
                        let (struct_name, fields_str) = if let Some(paren) = name.find('(') {
                            let n = &name[..paren];
                            let f = name[paren+1..].trim_end_matches(')');
                            (n.to_string(), f)
                        } else {
                            (name.clone(), "")
                        };
                        let fields: Vec<HirStructField> = if fields_str.is_empty() {
                            Vec::new()
                        } else {
                            fields_str.split(',').filter_map(|s| {
                                let mut parts = s.splitn(2, ':');
                                let field_name = Symbol::intern(parts.next()?);
                                let field_type_str = parts.next()?;
                                // Parse field type string back to HirType
                                let mut field_ty = sig_str_to_hir(field_type_str);
                                // 签名串里 `[T]` 写作 Array（不含 unique）→ 补回拥有数组所有权
                                if matches!(field_ty, HirType::Array(_) | HirType::ArraySized(_, _)) {
                                    field_ty = HirType::Unique(Box::new(field_ty));
                                }
                                Some(HirStructField { name: field_name, ty: field_ty })
                            }).collect()
                        };
                        // LIR binary 已提供精确类型时不要覆盖（符号串为有损表示）
                        self.struct_defs.entry(Symbol::intern(&struct_name)).or_insert(fields);
                    }
                    crate::package::ImportedSymbol::Namespace { .. } => {
                        // Handled by lowering; just register the path
                    }
                    crate::package::ImportedSymbol::Interface { .. } => {
                        // Interface definition is loaded from generic sources above
                    }
                }
            }
        Ok(())
    }
}
