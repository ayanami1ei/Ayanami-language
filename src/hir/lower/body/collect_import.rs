use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn collect_import(&mut self, path: &String, _ns_prefix: &str) -> Result<()> {
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
                    let gsp_from_lir: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>> =
                        dep_lir.generic_struct_params.clone();
                    for (name, fields) in dep_lir.struct_defs {
                        let hir_fields: Vec<HirStructField> = fields.iter()
                            .map(|(fn_name, ty)| HirStructField { name: *fn_name, ty: ty.clone() })
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
                                    gp_names.into_iter().map(|n| (n, None)).collect());
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
                let parsed = match parser.parse_program() {
                    Ok(p) => p,
                    Err(_) => { continue; }
                };
                    for stmt in &parsed.stmts {
                        match stmt {
                            Stmt::FnDecl { name, generic_params, .. } if !generic_params.is_empty() => {
                                self.generic_fns.push((*name, generic_params.clone(), stmt.clone()));
                            }
                            Stmt::ImplBlock { methods, generic_params: impl_gp, .. } => {
                                for m in methods {
                                    if let Stmt::FnDecl { name, generic_params, .. } = m {
                                        let combined: Vec<(Symbol, Option<Symbol>)> = {
                                            let mut all = impl_gp.clone();
                                            all.extend(generic_params.iter().cloned());
                                            all
                                        };
                                        if !combined.is_empty() {
                                            self.generic_fns.push((*name, combined, m.clone()));
                                        }
                                    }
                                }
                            }
                            Stmt::InterfaceDef { name, methods, generic_params, .. } => {
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
                    }
                    crate::package::ImportedSymbol::Pass { name } => {
                        let stem = Symbol::intern(&crate::hir::attrs::pkg_stem(path));
                        self.imported_passes.entry(stem).or_default().push(Symbol::intern(name));
                    }
                    crate::package::ImportedSymbol::Fn { name, sig, flags } => {
                        // sig format: "fnName(param_types...)->ret_type"
                        let sig_body = sig.trim_start_matches(name.as_str());
                        let arrow_pos = sig_body.find(")->")
                            .ok_or_else(|| Error::Hir(format!("invalid fn sig in package '{}': sig body `{}`", name, sig_body)))?;
                        let params_str = &sig_body[..arrow_pos];
                        let ret_str = &sig_body[arrow_pos + 3..];
                        // params_str is "(type1,type2" — strip leading '('
                        let params_str = params_str.strip_prefix('(').unwrap_or(params_str);
                        let param_tys: Vec<&str> = if params_str.is_empty() {
                            Vec::new()
                        } else {
                            params_str.split(',').collect()
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
                        self.fns.push(FnSig {
                            name: sym_name,
                            params: hir_params,
                            return_type: hir_ret,
                            effects: summary.declared,
                            inferred: summary.inferred,
                        });
                        self.fn_map.entry(sym_name).or_default().push(fn_id);
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
                                let field_ty = sig_str_to_hir(field_type_str);
                                Some(HirStructField { name: field_name, ty: field_ty })
                            }).collect()
                        };
                        self.struct_defs.insert(Symbol::intern(&struct_name), fields);
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
