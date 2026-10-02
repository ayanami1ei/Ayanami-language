use super::*;

impl crate::hir::lower::Ctx {
    pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, params, return_type, generic_params, .. } => {
                    let full_name = if ns_prefix.is_empty() {
                        *name
                    } else {
                        Symbol::intern(&format!("{}.{}", ns_prefix, name))
                    };
                    if !generic_params.is_empty() {
                        // Store generic function AST for later monomorphization;
                        // do NOT register it as a normal callable function.
                        self.generic_fns.push((full_name, generic_params.clone(), stmt.clone()));
                        continue;
                    }
                    let hir_return = ast_type_to_hir(return_type, &self.interfaces);
                    let hir_params = params.iter()
                        .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                        .collect();
                    let fn_id = FnId(self.fns.len());
                    self.fns.push(FnSig {
                        name: full_name,
                        params: hir_params,
                        return_type: hir_return,
                        effects: crate::hir::effects::EffectDecl::default(),
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
                    self.interfaces.insert(*name, InterfaceReg { generic_params: generic_params.clone(), methods: hir_methods });
                }
                Stmt::StructDef { name, fields, generic_params, .. } => {
                    let hir_fields: Vec<HirStructField> = fields.iter()
                        .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                        .collect();
                    self.struct_defs.insert(*name, hir_fields);
                    if !generic_params.is_empty() {
                        self.generic_struct_params.insert(*name, generic_params.clone());
                    }
                }
                Stmt::EnumDef { name, variants, generic_params, .. } => self.collect_enum_def(name, variants, generic_params, ns_prefix)?,
            Stmt::Import { path, .. } => self.collect_import(path, ns_prefix)?,
                Stmt::ImplBlock { methods, generic_params: impl_gp, .. } => {
                    for method in methods {
                        if let Stmt::FnDecl { name, params, return_type, generic_params: method_gp, .. } = method {
                            // 合并 impl 级和方法级泛型参数：impl[T] LinkedList[T] { fn push[T: Ord](...) }
                            let combined_gp: Vec<(Symbol, Option<Symbol>)> = {
                                let mut all = impl_gp.clone();
                                all.extend(method_gp.iter().cloned());
                                all
                            };
                            if !combined_gp.is_empty() {
                                self.generic_fns.push((*name, combined_gp, method.clone()));
                                continue;
                            }
                            let hir_return = ast_type_to_hir(return_type, &self.interfaces);
                            let hir_params = params.iter()
                                .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                                .collect();
                            let fn_id = FnId(self.fns.len());
                            self.fns.push(FnSig {
                                name: *name,
                                params: hir_params,
                                return_type: hir_return,
                                effects: crate::hir::effects::EffectDecl::default(),
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
