use super::*;

/// #149：泛型参数名不得重复（多约束语法暂不支持；定义期报错，避免调用期推断失败/误编译）
pub(super) fn check_unique_generic_params(gp: &[(Symbol, Option<Symbol>)], span: &Span) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for (n, _) in gp {
        if !seen.insert(*n) {
            return Err(Error::Hir(format!(
                "generic parameter `{}` is declared more than once (multi-bound syntax is not supported yet; use a single constraint) (at {}:{})",
                n.as_str(), span.start_line, span.start_col
            )));
        }
    }
    Ok(())
}

impl crate::hir::lower::Ctx {
    pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, params, return_type, generic_params, extern_c, span, attrs, .. } => {
                    let full_name = if ns_prefix.is_empty() {
                        *name
                    } else {
                        Symbol::intern(&format!("{}.{}", ns_prefix, name))
                    };
                    if !generic_params.is_empty() {
                        check_unique_generic_params(generic_params, span)?;
                        // Store generic function AST for later monomorphization;
                        // do NOT register it as a normal callable function.
                        self.generic_fns.push((full_name, generic_params.clone(), stmt.clone()));
                        continue;
                    }
                    let is_extern = *extern_c || crate::hir::attrs::has(attrs, "export");
                    let hir_return = if is_extern {
                        ast_type_to_hir_extern(return_type, &self.interfaces)
                    } else {
                        ast_type_to_hir(return_type, &self.interfaces)
                    };
                    let hir_params: Vec<(Symbol, crate::hir::ty::HirType)> = params.iter()
                        .map(|(n, t)| (*n, if is_extern {
                            ast_type_to_hir_extern(t, &self.interfaces)
                        } else {
                            ast_type_to_hir(t, &self.interfaces)
                        }))
                        .collect();
                    // extern 声明若与已导入的同签名函数重复，直接复用（避免重载歧义；
                    // 导入侧形参名为空，比较时只看类型）
                    if *extern_c && self.fns.iter().any(|s| {
                        s.name == full_name
                            && s.params.len() == hir_params.len()
                            && s.params.iter().zip(&hir_params).all(|(a, b)| a.1 == b.1)
                    }) {
                        continue;
                    }
                    let fn_id = FnId(self.fns.len());
                    let hidden = count_hidden_params(&hir_params);
                    let is_noreturn = crate::hir::attrs::has(attrs, "noreturn")
                        || matches!(hir_return, HirType::Never);
                    self.fns.push(FnSig {
                        name: full_name,
                        params: hir_params,
                        return_type: hir_return,
                        effects: crate::hir::effects::EffectDecl::default(),
                        inferred: Default::default(),
                        span: *span,
                        hidden,
                        is_noreturn,
                    });
                    self.fn_map.entry(full_name).or_default().push(fn_id);
                }
                Stmt::Namespace { name, items, .. } => {
                    let nested = if ns_prefix.is_empty() {
                        name.as_str().to_string()
                    } else {
                        format!("{}.{}", ns_prefix, name)
                    };
                    self.collect_fns_with_ns(items, &nested)?;
                }
                Stmt::ConstDecl { name, ty, value, span, .. } => {
                    self.collect_const_decl(*name, ty.as_ref(), value, span)?;
                    // M6.1b：顶层导出裸名；命名空间内同时注册/导出限定名 `ns.NAME`
                    let export = if ns_prefix.is_empty() {
                        Some(*name)
                    } else {
                        let qualified = Symbol::intern(&format!("{}.{}", ns_prefix, name));
                        self.consts.get(name).cloned().map(|e| {
                            self.consts.insert(qualified, e);
                            qualified
                        })
                    };
                    if let Some(export) = export {
                        if let Some((ty, lit)) = self.consts.get(&export).cloned() {
                            self.const_exports.push((export, ty, lit));
                        }
                    }
                }
                Stmt::StaticDecl { vis, name, is_mut, ty, value, span, .. } => {
                    self.collect_static_decl(*name, *is_mut, vis.is_public(), ty.as_ref(), value, span)?;
                    // M6.2b：命名空间 static：限定名 `ns.NAME` 为发射符号；裸名是同符号别名
                    if !ns_prefix.is_empty() {
                        let qualified = Symbol::intern(&format!("{}.{}", ns_prefix, name));
                        if let Some(st) = self.statics.get(name).cloned() {
                            self.statics.insert(*name, crate::hir::HirStatic { name: qualified, ..st.clone() });
                            self.statics.insert(qualified, crate::hir::HirStatic { name: qualified, ..st });
                        }
                    }
                }
                Stmt::InterfaceDef { name, methods, generic_params, span, .. } => {
                    check_unique_generic_params(generic_params, span)?;
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
                    self.interfaces.insert(*name, InterfaceReg { generic_params: generic_params.clone(), methods: hir_methods });
                }
                Stmt::StructDef { name, fields, generic_params, span, .. } => {
                    check_unique_generic_params(generic_params, span)?;
                    let hir_fields: Vec<HirStructField> = fields.iter()
                        .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                        .collect();
                    self.struct_defs.insert(*name, hir_fields);
                    if !generic_params.is_empty() {
                        self.generic_struct_params.insert(*name, generic_params.clone());
                    }
                }
                Stmt::EnumDef { name, variants, generic_params, span, .. } => {
                    check_unique_generic_params(generic_params, span)?;
                    self.collect_enum_def(name, variants, generic_params, ns_prefix)?
                }
            Stmt::Import { path, span, .. } => self.collect_import(path, ns_prefix, *span)?,
                Stmt::ImplBlock { methods, generic_params: impl_gp, span, .. } => {
                    check_unique_generic_params(impl_gp, span)?;
                    for method in methods {
                        if let Stmt::FnDecl { name, params, return_type, generic_params: method_gp, span: method_span, attrs: method_attrs, .. } = method {
                            // 合并 impl 级和方法级泛型参数：impl[T] LinkedList[T] { fn push[T: Ord](...) }
                            let combined_gp: Vec<(Symbol, Option<Symbol>)> = {
                                let mut all = impl_gp.clone();
                                all.extend(method_gp.iter().cloned());
                                all
                            };
                            check_unique_generic_params(&combined_gp, method_span)?;
                            if !combined_gp.is_empty() {
                                self.generic_fns.push((*name, combined_gp, method.clone()));
                                continue;
                            }
                            let hir_return = ast_type_to_hir(return_type, &self.interfaces);
                            let hir_params: Vec<(Symbol, HirType)> = params.iter()
                                .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                                .collect();
                            let fn_id = FnId(self.fns.len());
                            let hidden = count_hidden_params(&hir_params);
                            let is_noreturn = crate::hir::attrs::has(method_attrs, "noreturn")
                                || matches!(hir_return, HirType::Never);
                            self.fns.push(FnSig {
                                name: *name,
                                params: hir_params,
                                return_type: hir_return,
                                span: *method_span,
                                effects: crate::hir::effects::EffectDecl::default(),
                        inferred: Default::default(),
                        hidden,
                                is_noreturn,
                            });
                            self.fn_map.entry(*name).or_default().push(fn_id);
                        } else {
                            let s = method.span();
                            return Err(Error::Hir(format!("unexpected non-FnDecl inside impl block (at {}:{})", s.start_line, s.start_col)));
                        }
                    }
                }
                _ => {
                    let s = stmt.span();
                    return Err(Error::Hir(format!("unexpected top-level statement outside function, namespace, interface, or impl block (at {}:{})", s.start_line, s.start_col)));
                }
            }
        }
        Ok(())
    }

    // ----------------------------------------------------------------
    //  阶段 1.5：构建虚函数表（vtable）
    //  遍历接口定义和 impl 块，验证方法签名一致性，建立虚函数映射
    // ----------------------------------------------------------------
}
