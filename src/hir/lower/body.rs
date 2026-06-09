use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;
use crate::hir::ir::*;
use super::helpers::*;
use super::{FnSig, InterfaceReg, Ctx};

impl super::Ctx {
    // ----------------------------------------------------------------
    //  阶段 1：收集函数和接口签名
    // ----------------------------------------------------------------

    pub(super) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for s in stmts {
            if let Stmt::Import { path, .. } = s {
            }
        }
        self.collect_fns_with_ns(stmts, "")
    }

    pub(super) fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<(), String> {
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
                Stmt::Import { path, .. } => {
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
                        .map_err(|e| format!("import error for '{}': {}", path, e))?;

                    // Merge struct definitions from the package's LIR data
                    if !lir_binary.is_empty() {
                        let dep_lir = crate::lir::serialize::program_from_bytes(&lir_binary);
                        if let Err(e) = &dep_lir {
                        }
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

                    // Parse and register generic function ASTs and interfaces from the package
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
                            Err(e) => { continue; }
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
                            crate::package::ImportedSymbol::Fn { name, sig } => {
                                // sig format: "fnName(param_types...)->ret_type"
                                let sig_body = sig.trim_start_matches(name.as_str());
                                let arrow_pos = sig_body.find(")->")
                                    .ok_or_else(|| format!("invalid fn sig in package '{}': sig body `{}`", name, sig_body))?;
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
                                self.fns.push(FnSig {
                                    name: sym_name,
                                    params: hir_params,
                                    return_type: hir_ret,
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
                }
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
                            });
                            self.fn_map.entry(*name).or_default().push(fn_id);
                        } else {
                            let s = method.span();
                            return Err(format!("unexpected non-FnDecl inside impl block (at {}:{})", s.start_line, s.start_col));
                        }
                    }
                }
                _ => {
                    let s = stmt.span();
                    return Err(format!("unexpected top-level statement outside function, namespace, interface, or impl block (at {}:{})", s.start_line, s.start_col));
                }
            }
        }
        Ok(())
    }

    // ----------------------------------------------------------------
    //  阶段 1.5：构建虚函数表（vtable）
    //  遍历接口定义和 impl 块，验证方法签名一致性，建立虚函数映射
    // ----------------------------------------------------------------

    /// For each impl block type, check which interfaces it satisfies
    /// (structural typing: methods with same name + compatible signatures)
    pub(super) fn build_vtables(&mut self) -> Result<(), String> {
        // Collect all impl types and their methods (owned copies to avoid borrow conflicts)
        let mut impl_methods: HashMap<Symbol, Vec<FnSig>> = HashMap::new();
        for sig in &self.fns {
            if !sig.params.is_empty() {
                let inner_ty = match &sig.params[0].1 {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                let type_name = match inner_ty {
                    HirType::Named(n) => Some(*n),
                    HirType::Int => Some(Symbol::intern("int")),
                    HirType::Float => Some(Symbol::intern("float")),
                    HirType::Char => Some(Symbol::intern("char")),
                    HirType::Bool => Some(Symbol::intern("bool")),
                    HirType::Void => Some(Symbol::intern("void")),
                    _ => None,
                };
                if let Some(tn) = type_name {
                    impl_methods.entry(tn).or_default().push(sig.clone());
                }
            }
        }

        // Also add generic_fns methods for interface matching
        for (gf_name, _, gf_stmt) in &self.generic_fns {
            if let Stmt::FnDecl { params, return_type, .. } = gf_stmt {
                if params.is_empty() { continue; }
                let param_ty = ast_type_to_hir(&params[0].1, &self.interfaces);
                let inner_ty = match &param_ty {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                if let HirType::Named(n) = inner_ty {
                    let base = crate::hir::lower::strip_generic_name(n);
                    let hir_params: Vec<(Symbol, HirType)> = params.iter()
                        .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                        .collect();
                    let hir_return = ast_type_to_hir(return_type, &self.interfaces);
                    impl_methods.entry(base).or_default().push(FnSig {
                        name: *gf_name,
                        params: hir_params,
                        return_type: hir_return,
                    });
                }
            }
        }

        let iface_list: Vec<(Symbol, Vec<(Symbol, Option<Symbol>)>, Vec<HirInterfaceMethod>)> = self.interfaces.iter()
            .map(|(name, reg)| (*name, reg.generic_params.clone(), reg.methods.clone()))
            .collect();
        for (iface_name, iface_gp, iface_methods) in &iface_list {
            for (type_name, methods) in &impl_methods {
                let reg = InterfaceReg { generic_params: iface_gp.clone(), methods: iface_methods.clone() };
                if !iface_gp.is_empty() {
                    self.try_match_generic_interface(iface_name, &reg, type_name, methods)?;
                } else {
                    self.try_match_interface(iface_name, &reg, type_name, methods)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn try_match_interface(
        &mut self, iface_name: &Symbol, iface_reg: &InterfaceReg,
        type_name: &Symbol, methods: &[FnSig],
    ) -> Result<(), String> {
        let mut vtable_fns: Vec<FnId> = Vec::new();
        vtable_fns.push(FnId(usize::MAX));
        let mut all_match = true;
        for iface_method in &iface_reg.methods {
            let found = methods.iter().find(|m| m.name == iface_method.name);
            match found {
                Some(fsig) => {
                    let params_match = iface_method.params.len() == fsig.params.len() - 1
                        && iface_method.params.iter().zip(&fsig.params[1..])
                            .all(|((_, ift), (_, ft))| Self::type_matches(&ift, &ft))
                        && Self::type_matches(&iface_method.return_type, &fsig.return_type);
                    if params_match {
                        let fn_id = self.fn_map.get(&fsig.name)
                            .and_then(|ids| ids.iter().find(|id| {
                                let s = &self.fns[id.0];
                                s.params == fsig.params && s.return_type == fsig.return_type
                            }));
                        if let Some(&fid) = fn_id {
                            vtable_fns.push(fid);
                        } else {
                            all_match = false;
                            break;
                        }
                    } else {
                        let iface_sig = format!("({} self{}) -> {}",
                            iface_method.self_keyword.as_str(),
                            iface_method.params.iter().map(|(n, t)| format!(", {} {}", hir_type_display(t), n.as_str())).collect::<String>(),
                            hir_type_display(&iface_method.return_type));
                        let impl_sig = format!("({} self{}) -> {}",
                            iface_method.self_keyword.as_str(),
                            fsig.params[1..].iter().map(|(n, t)| format!(", {} {}", hir_type_display(t), n.as_str())).collect::<String>(),
                            hir_type_display(&fsig.return_type));
                        return Err(format!(
                            "method `{}` in impl `{}` has wrong signature for interface `{}`:\n  expected {}\n  found    {}",
                            iface_method.name.as_str(), type_name.as_str(), iface_name.as_str(),
                            iface_sig, impl_sig));
                    }
                }
                None => { all_match = false; break; }
            }
        }
        if all_match {
            self.vtables.push(VtableEntry {
                concrete_type: *type_name,
                interface: *iface_name,
                method_fn_ids: vtable_fns,
            });
            self.type_ifaces.entry(*type_name).or_default().push(*iface_name);
        }
        Ok(())
    }

    /// Try to match a generic interface against a type's methods.
    /// For each method in the interface, infer generic param substitutions
    /// and create a specialized interface for each valid substitution set.
    pub(super) fn try_match_generic_interface(
        &mut self, iface_name: &Symbol, iface_reg: &InterfaceReg,
        type_name: &Symbol, methods: &[FnSig],
    ) -> Result<(), String> {
        let gp_names: Vec<Symbol> = iface_reg.generic_params.iter().map(|(n, _)| *n).collect();
        let mut results: Vec<HashMap<Symbol, HirType>> = vec![HashMap::new()];

        for iface_method in &iface_reg.methods {
            let mut next_results = Vec::new();
            for impl_method in methods.iter().filter(|m| m.name == iface_method.name) {
                if iface_method.params.len() != impl_method.params.len() - 1 { continue; }
                for subst in &results {
                    let mut local = subst.clone();
                    let mut ok = true;
                    for ((_, ift), (_, impt)) in iface_method.params[1..].iter().zip(&impl_method.params[1..]) {
                        if !Self::infer_iface_generic(ift, impt, &gp_names, &mut local) { ok = false; break; }
                    }
                    if !ok { continue; }
                    if !Self::infer_iface_generic(&iface_method.return_type, &impl_method.return_type, &gp_names, &mut local) {
                        continue;
                    }
                    if local.iter().any(|(k, _)| gp_names.contains(k)) {
                        let fn_id = self.fn_map.get(&impl_method.name)
                            .and_then(|ids| ids.iter().find(|id| {
                                let s = &self.fns[id.0];
                                s.params == impl_method.params && s.return_type == impl_method.return_type
                            }));
                        if let Some(&fid) = fn_id {
                            local.insert(Symbol::intern(&format!("__fn{}", iface_method.name.as_str())), HirType::Named(Symbol::intern(&format!("id{}", fid.0))));
                            next_results.push((local, fid));
                        }
                    }
                }
            }
            if next_results.is_empty() { return Ok(()); }
            results = next_results.iter().map(|(s, _)| s.clone()).collect();
        }

        // Build specialized interface name for each result
        for subst in &results {
            let args_str: Vec<String> = gp_names.iter()
                .map(|n| subst.get(n).map(|t| hir_type_display(t)).unwrap_or_else(|| n.to_string()))
                .collect();
            let specialized_name = format!("{}<{}>", iface_name.as_str(), args_str.join(","));
            let specialized_sym = Symbol::intern(&specialized_name);

            // Register the specialized interface
            if !self.interfaces.contains_key(&specialized_sym) {
                let subst_methods: Vec<HirInterfaceMethod> = iface_reg.methods.iter().map(|m| {
                    HirInterfaceMethod {
                        name: m.name,
                        self_keyword: m.self_keyword,
                        params: m.params.iter().map(|(n, t)| (*n, Self::substitute_iface_type(t, subst, &gp_names))).collect(),
                        return_type: Self::substitute_iface_type(&m.return_type, subst, &gp_names),
                    }
                }).collect();
                self.interfaces.insert(specialized_sym, InterfaceReg { generic_params: vec![], methods: subst_methods });
            }

            // Build vtable for this specialized interface
            let mut vtable_fns: Vec<FnId> = vec![FnId(usize::MAX)];
            let mut ok = true;
            for iface_method in &iface_reg.methods {
                let substituted_params: Vec<HirType> = iface_method.params.iter().map(|(_, t)| Self::substitute_iface_type(t, subst, &gp_names)).collect();
                let substituted_ret = Self::substitute_iface_type(&iface_method.return_type, subst, &gp_names);
                let found = methods.iter().filter(|m| m.name == iface_method.name).find(|m| {
                    m.params.len() - 1 == substituted_params.len()
                    && m.params[1..].iter().zip(&substituted_params).all(|((_, pt), st)| pt == st)
                    && Self::type_matches(&m.return_type, &substituted_ret)
                });
                match found {
                    Some(fsig) => {
                        let fn_id = self.fn_map.get(&fsig.name).and_then(|ids| ids.iter().find(|id| {
                            let s = &self.fns[id.0]; s.params == fsig.params && s.return_type == fsig.return_type
                        }));
                        if let Some(&fid) = fn_id { vtable_fns.push(fid); } else { ok = false; break; }
                    }
                    None => { ok = false; break; }
                }
            }
            if ok {
                self.vtables.push(VtableEntry { concrete_type: *type_name, interface: specialized_sym, method_fn_ids: vtable_fns });
                self.type_ifaces.entry(*type_name).or_default().push(specialized_sym);
            }
        }
        Ok(())
    }

    pub(super) fn infer_iface_generic(
        expected: &HirType, actual: &HirType,
        gp_names: &[Symbol], subst: &mut HashMap<Symbol, HirType>,
    ) -> bool {
        match (expected, actual) {
            (HirType::Named(n), _) if gp_names.contains(n) => {
                if let Some(existing) = subst.get(n) { existing == actual }
                else { subst.insert(*n, actual.clone()); true }
            }
            _ => Self::type_matches(expected, actual),
        }
    }

    pub(super) fn substitute_iface_type(ty: &HirType, subst: &HashMap<Symbol, HirType>, gp_names: &[Symbol]) -> HirType {
        match ty {
            HirType::Named(n) if gp_names.contains(n) => subst.get(n).cloned().unwrap_or_else(|| ty.clone()),
            HirType::Shared(inner) => HirType::Shared(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::Unique(inner) => HirType::Unique(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::Weak(inner) => HirType::Weak(Box::new(Self::substitute_iface_type(inner, subst, gp_names))),
            HirType::FatPtr { name, kind } => HirType::FatPtr { name: *name, kind: Box::new(Self::substitute_iface_type(kind, subst, gp_names)) },
            _ => ty.clone(),
        }
    }

    pub(super) fn type_matches(a: &HirType, b: &HirType) -> bool {
        if a == b { return true; }
        match (a, b) {
            (HirType::Named(an), HirType::Int) if an.as_str() == "int" => true,
            (HirType::Named(an), HirType::Float) if an.as_str() == "float" => true,
            (HirType::Named(an), HirType::Char) if an.as_str() == "char" => true,
            (HirType::Named(an), HirType::Void) if an.as_str() == "void" => true,
            (HirType::Named(an), HirType::Bool) if an.as_str() == "bool" => true,
            _ => false,
        }
    }

    /// Find a function by exact name + param type match (for Phase 2 lookup)
    pub(super) fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId> {
        for (i, sig) in self.fns.iter().enumerate() {
            if sig.name == name
                && sig.params.len() == param_types.len()
                && sig.params.iter().zip(param_types).all(|((_, pt), at)| pt == at)
            {
                return Some(FnId(i));
            }
        }
        None
    }

    /// Extract a concrete type name from an HirType (stripping ownership).
    fn extract_concrete_type_name(ty: &HirType) -> Option<Symbol> {
        match ty {
            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                Self::extract_concrete_type_name(inner)
            }
            HirType::Named(n) => Some(*n),
            HirType::Int => Some(Symbol::intern("int")),
            HirType::Float => Some(Symbol::intern("float")),
            HirType::Char => Some(Symbol::intern("char")),
            HirType::Bool => Some(Symbol::intern("bool")),
            _ => None,
        }
    }

    /// Check if an arg type can be passed to a param type (accounting for FatPtr wrapping)
    pub(super) fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        // Check if param is FatPtr and arg is a concrete type that implements the interface
        if let HirType::FatPtr { name: iface_name, .. } = param_ty {
            let concrete = match arg_ty {
                HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                    if let HirType::Named(n) = inner.as_ref() { Some(*n) } else { None }
                }
                HirType::Named(n) => Some(*n),
                HirType::Int => Some(Symbol::intern("int")),
                HirType::Float => Some(Symbol::intern("float")),
                HirType::Char => Some(Symbol::intern("char")),
                HirType::Bool => Some(Symbol::intern("bool")),
                _ => None,
            };
            if let Some(ct) = concrete {
                if let Some(ifaces) = self.type_ifaces.get(&ct) {
                    return ifaces.contains(iface_name);
                }
                let base_ct = crate::hir::lower::strip_generic_name(&ct);
                if base_ct != ct {
                    if let Some(ifaces) = self.type_ifaces.get(&base_ct) {
                        return ifaces.contains(iface_name);
                    }
                }
                // Fallback: check generic_fns for matching impl methods
                // Only allow if the concrete type has generic params (<...>) or the struct isn't generic
                let has_gp = self.generic_struct_params.contains_key(&base_ct);
                if has_gp && !ct.as_str().contains('<') {
                    return false;
                }
                return self.check_generic_fns_for_iface(&base_ct, iface_name);
            }
        }
        false
    }

    /// Check if a type's generic_fns methods structurally match an interface.
    /// This enables on-the-fly interface matching for generic impls.
    fn check_generic_fns_for_iface(&self, type_name: &Symbol, iface_name: &Symbol) -> bool {
        let iface_reg = match self.interfaces.get(iface_name) {
            Some(r) => r,
            None => return false,
        };
        for iface_method in &iface_reg.methods {
            let has_match = self.generic_fns.iter().any(|(gf_name, _, gf_stmt)| {
                if *gf_name != iface_method.name { return false; }
                if let Stmt::FnDecl { params, .. } = gf_stmt {
                    if params.is_empty() { return false; }
                    let self_ty = ast_type_to_hir(&params[0].1, &self.interfaces);
                    let self_inner = match &self_ty {
                        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                        other => other,
                    };
                    let self_base = match self_inner {
                        HirType::Named(n) => crate::hir::lower::strip_generic_name(n),
                        _ => return false,
                    };
                    if self_base != *type_name { return false; }
                    // iface.params excludes self, impl.params includes self
                    if params.len() - 1 != iface_method.params.len() { return false; }
                    params[1..].iter().zip(&iface_method.params).all(|((_, pt), (_, ift))| {
                        let pt_hir = ast_type_to_hir(pt, &self.interfaces);
                        Self::type_matches(&pt_hir, ift)
                    })
                } else { false }
            });
            if !has_match { return false; }
        }
        true
    }

    /// Register vtable for a generic impl type → interface relationship.
    /// Specializes methods on the fly.
    fn register_generic_vtable(
        &mut self,
        concrete_type: &Symbol,
        base_type: &Symbol,
        iface_name: &Symbol,
    ) -> Result<(), String> {
        let iface_reg = match self.interfaces.get(iface_name) {
            Some(r) => r.clone(),
            None => return Ok(()),
        };
        let mut vtable_fns: Vec<FnId> = vec![FnId(usize::MAX)];
        let span = crate::span::Span::default();
        for iface_method in &iface_reg.methods {
            let self_ty = HirType::Shared(Box::new(HirType::Named(*concrete_type)));
            let mut arg_types = vec![self_ty];
            for (_, ift) in &iface_method.params {
                arg_types.push(ift.clone());
            }
            let fid = self.specialize_generic_call(&iface_method.name, &arg_types, &span)?;
            vtable_fns.push(fid);
        }
        self.vtables.push(VtableEntry {
            concrete_type: *concrete_type,
            interface: *iface_name,
            method_fn_ids: vtable_fns,
        });
        self.type_ifaces.entry(*concrete_type).or_default().push(*iface_name);
        // Also register under base type for future lookups
        if *base_type != *concrete_type {
            self.type_ifaces.entry(*base_type).or_default().push(*iface_name);
        }
        Ok(())
    }

    pub(super) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        if param_ty == arg_ty { return true; }
        // Shared/Unique value types: allow passing plain T to shared T
        if let HirType::Shared(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        if let HirType::Unique(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        // Allow passing Shared(T)/Unique(T)/Weak(T) to plain T
        // (ownership wrapper is transparent for primitives)
        if let HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) = arg_ty {
            if param_ty == inner.as_ref() { return true; }
        }
        // FatPtr compatibility
        self.is_fatptr_compatible(param_ty, arg_ty)
    }

    /// Resolve a function call by name and argument types (overload-aware)
    pub(super) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(name);
        let candidates = match candidates {
            Some(c) => c,
            None => {
                return None;
            }
        };
        let matches: Vec<FnId> = candidates.iter().copied()
            .filter(|&fn_id| {
                let sig = &self.fns[fn_id.0];
                sig.params.len() == arg_types.len()
                    && sig.params.iter().zip(arg_types).all(|((_, pt), at)| {
                        self.param_compatible(pt, at)
                    })
            })
            .collect();
        if matches.len() == 1 {
            Some(matches[0])
        } else {
            if matches.len() > 1 {
            }
            None
        }
    }

    /// 检查接收者类型是否匹配方法的 self 参数类型
    ///
    /// 支持多层所有权包装的自动剥离，例如：
    /// - `Unique(Shared(LinkedList))` 匹配 `Unique(LinkedList)`
    /// - `Shared(LinkedList)` 匹配 `LinkedList`
    /// - `Unique(Shared(LinkedList))` 匹配 `LinkedList`
    pub(super) fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool {
        if receiver == param { return true; }
        // 逐层剥离接收者的所有权包装（Unique/Shared/Weak）
        // 处理多层包装如 Unique(Shared(T)) 的情况
        let recv_inner = match receiver {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        // 如果剥离后与参数完全相等，则匹配
        if recv_inner == param { return true; }
        // 再剥离参数的所有权包装
        let param_inner = match param {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if recv_inner == param_inner { return true; }
        // 接收者仍有包装层未剥离？递归处理（如 Unique(Shared(T)) → Shared(T)）
        if recv_inner != receiver {
            return Self::receiver_matches_param(recv_inner, param);
        }
        // 允许向 shared/unique self 传入裸类型（自动包装）
        match param {
            HirType::Shared(inner) | HirType::Unique(inner) => {
                if receiver == inner.as_ref() { return true; }
            }
            _ => {}
        }
        false
    }

    pub(super) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(method_name)?;
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let remaining = &sig.params[1..];
            if remaining.len() != arg_types.len() { continue; }
            if remaining.iter().zip(arg_types).all(|((_, pt), at)| pt == at) {
                return Some(fn_id);
            }
        }
        None
    }

    // ----------------------------------------------------------------
    //  阶段：泛型特化（单态化）
    //  当函数调用匹配到泛型函数时，根据具体参数类型生成特化版本
    // ----------------------------------------------------------------

    /// Try to resolve a call by specializing a generic function.
    /// Returns the FnId of the newly-created specialized function on success.
    pub(super) fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId, String> {
        // Find matching generic function
        let gf_idx = self.generic_fns.iter().position(|(gf_name, _, _)| gf_name == name);
        let gf_idx = match gf_idx {
            Some(i) => i,
            None => {
                return Err(if self.fn_map.contains_key(name) {
                    let ats: Vec<String> = arg_types.iter().map(|t| format!("{:?}", t)).collect();
                    format!("no matching overload of `{}` for argument types ({}); candidate(s) exist at {}:{}",
                        name, ats.join(", "), span.start_line, span.start_col)
                } else {
                    format!("undefined function `{}` at {}:{}", name, span.start_line, span.start_col)
                });
            }
        };

        let (gf_name, gf_params, gf_stmt) = &self.generic_fns[gf_idx];
        let Stmt::FnDecl { params, return_type, body, is_inline, extern_c, .. } = gf_stmt else {
            return Err(format!("internal error: generic function `{}` is not a FnDecl at {}:{}", gf_name, span.start_line, span.start_col));
        };

        if params.len() != arg_types.len() {
            return Err(format!(
                "generic function `{}` takes {} argument(s) but {} given at {}:{}",
                gf_name, params.len(), arg_types.len(), span.start_line, span.start_col
            ));
        }

        // Step 1: Infer concrete type for each generic parameter
        let generic_names: Vec<Symbol> = gf_params.iter().map(|(n, _)| *n).collect();
        let mut generic_mappings: HashMap<Symbol, HirType> = HashMap::new();
        for ((_, param_ty), arg_ty) in params.iter().zip(arg_types.iter()) {
            let result = infer_generic_from_param(param_ty, arg_ty);
            if let Some((n, _)) = &result {
            }
            if let Some((gp_name, hir_concrete)) = result {
                if generic_names.contains(&gp_name) && !generic_mappings.contains_key(&gp_name) {
                    generic_mappings.insert(gp_name, hir_concrete.clone());
                }
            }
        }
        // Ensure all generic params were resolved
        for (gp_name, _) in gf_params {
            if !generic_mappings.contains_key(gp_name) {
                return Err(format!(
                    "cannot infer generic parameter `{}` for function `{}` at {}:{}",
                    gp_name, gf_name, span.start_line, span.start_col
                ));
            }
        }

        // Step 1.5: Check interface constraints
        for (gp_name, constraint) in gf_params {
            if let Some(iface_name) = constraint {
                let concrete_ty = generic_mappings.get(gp_name)
                    .ok_or_else(|| format!("internal error: generic param `{}` not resolved at {}:{}", gp_name, span.start_line, span.start_col))?;
                // 递归剥离所有权包装（Unique/Shared/Weak），
                // 处理多层包装如 Unique(Shared(LinkedList)) → LinkedList
                let mut concrete_inner = concrete_ty;
                while matches!(concrete_inner, HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_)) {
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
                            return Err(format!(
                                "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                                hir_type_display(concrete_ty), iface_name, gp_name,
                                span.start_line, span.start_col
                            ));
                        }
                        // 直接标记为已实现，跳过后续 type_ifaces 检查
                        continue;
                    }
                    HirType::Int => Symbol::intern("int"),
                    HirType::Float => Symbol::intern("float"),
                    HirType::Char => Symbol::intern("char"),
                    HirType::Bool => Symbol::intern("bool"),
                    HirType::Array(_) => Symbol::intern("[int]"), // simplified
                    _ => return Err(format!(
                        "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                        hir_type_display(concrete_ty), iface_name, gp_name,
                        span.start_line, span.start_col
                    )),
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
                        return Err(format!(
                            "type `{}` does not implement interface `{}` required by generic parameter `{}` at {}:{}",
                            hir_type_display(concrete_ty), iface_name, gp_name,
                            span.start_line, span.start_col
                        ));
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
        });
        self.fn_map.entry(*name).or_default().push(fid);

        // Step 5: Lower the specialized function body
        let saved_current_fn = self.current_fn;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::take(&mut self.scopes);

        let hir_fn = self.lower_fn(fid, *name, &new_params, &new_return_type, &new_body, *is_inline, *extern_c, Span::default())?;

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

    pub(super) fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>, String> {
        self.lower_items_with_ns(stmts, "")
    }

    pub(super) fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>, String> {
        let mut items = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, params, return_type, body, is_inline, extern_c, generic_params, span, .. } => {
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
                        .ok_or_else(|| format!("internal error: function `{}` not found at {}:{}", full_name, span.start_line, span.start_col))?;
                    let hir_fn = self.lower_fn(fn_id, full_name, params, return_type, body, *is_inline, *extern_c, *span)?;
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
                Stmt::Import { .. } => {} // already handled in collect_fns
                Stmt::ImplBlock { methods, generic_params: impl_gp, .. } => {
                    // Flatten impl block: lower each method as a regular Fn
                    for method_stmt in methods {
                        if let Stmt::FnDecl { name, params, return_type, body, generic_params, .. } = method_stmt {
                            if !generic_params.is_empty() || !impl_gp.is_empty() {
                                continue; // generic methods are lowered during specialization
                            }
                            let ptypes: Vec<HirType> = params.iter()
                                .map(|(_, t)| ast_type_to_hir(t, &self.interfaces))
                                .collect();
                            let fn_id = self.find_fn_by_sig(*name, &ptypes)
                                .ok_or_else(|| {
                                    let s = method_stmt.span();
                                    format!("internal error: method `{}` not found at {}:{}", name, s.start_line, s.start_col)
                                })?;
                            let hir_fn = self.lower_fn(fn_id, *name, params, return_type, body, false, false, Span::default())?;
                            items.push(HirItem::Fn(hir_fn));
                        }
                    }
                }
                _ => {
                    let s = stmt.span();
                    return Err(format!("unexpected top-level statement (at {}:{})", s.start_line, s.start_col));
                }
            }
        }
        Ok(items)
    }

    /// 从 HirType 中递归收集泛型参数名（如 T，含编码名 LinkedListNode[T] 里的 T）
    pub(super) fn collect_gp_from_type(ty: &HirType, out: &mut Vec<Symbol>) {
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
            HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
                Self::collect_gp_from_type(inner, out);
            }
            HirType::Array(inner) => Self::collect_gp_from_type(inner, out),
            HirType::Ref(inner, _) => Self::collect_gp_from_type(inner, out),
            HirType::FatPtr { kind, .. } => Self::collect_gp_from_type(kind, out),
            _ => {}
        }
    }

    pub(super) fn lower_fn(
        &mut self,
        fn_id: FnId,
        name: Symbol,
        ast_params: &[(Symbol, Type)],
        _return_type: &Type,
        body: &Block,
        is_inline: bool,
        extern_c: bool,
        span: Span,
    ) -> Result<HirFn, String> {
        self.current_fn = fn_id;
        self.locals = Vec::new();
        self.scopes = Vec::new();

        let sig = &self.fns[fn_id.0];
        let return_type = sig.return_type.clone();

        // Push global scope for params
        self.push_scope();

        let mut hir_params = Vec::new();
        for (param_name, param_type) in ast_params {
            let hir_ty = ast_type_to_hir(param_type, &self.interfaces);
            let var_id = VarId(self.locals.len());
            self.locals.push(HirLocal::new(*param_name, hir_ty.clone(), false));
            self.bind_var(*param_name, var_id, hir_ty.clone(), false);
            hir_params.push((*param_name, hir_ty));
        }

        let hir_body = self.lower_block(body)?;

        let locals = std::mem::take(&mut self.locals);
        Ok(HirFn {
            span,
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

    pub(super) fn lower_block(&mut self, block: &Block) -> Result<HirBlock, String> {
        self.push_scope();
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.lower_stmt(stmt)?);
        }
        self.pop_scope();
        Ok(HirBlock::new(stmts))
    }

    pub(super) fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt, String> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                let hir_value = self.lower_expr(value)?;
                let hir_value = implicit_move(hir_value);
                let value_ty = expr_type(&hir_value);
                let (var_id, ty, _) = self.register_or_lookup(*name, value_ty);
                Ok(HirStmt::Assign {
                    target: HirExpr::Local(var_id, ty.clone()),
                    value: hir_value,
                })
            }
            Stmt::FieldAssign { object, field, value, span: stmt_span } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                let field_index = self.find_field_index(&object_ty, field, stmt_span)?;
                let field_ty = self.find_field_type(&object_ty, field, stmt_span)?;
                let hir_value = self.lower_expr(value)?;
                let hir_value = implicit_move(hir_value);
                Ok(HirStmt::FieldAssign {
                    object: Box::new(hir_object),
                    field: *field,
                    field_index,
                    field_ty,
                    value: hir_value,
                })
            }
            Stmt::IndexAssign { object, index, value, .. } => {
                let hir_object = self.lower_expr(object)?;
                let hir_index = self.lower_expr(index)?;
                let hir_value = self.lower_expr(value)?;
                let hir_value = implicit_move(hir_value);
                Ok(HirStmt::IndexAssign {
                    object: Box::new(hir_object),
                    index: Box::new(hir_index),
                    value: hir_value,
                })
            }
            Stmt::Return { value, .. } => {
                let hir_value = match value {
                    Some(v) => {
                        let expr = self.lower_expr(v)?;
                        let expr_ty = expr_type(&expr);
                        // 若函数返回 unique T，但表达式是裸 T，自动包装为 ToUnique
                        let fn_ret = &self.fns[self.current_fn.0].return_type;
                        let wrapped = match (fn_ret, &expr_ty) {
                            (HirType::Unique(pt), _) if *pt.as_ref() == expr_ty => {
                                HirExpr::ToUnique(Box::new(expr), fn_ret.clone())
                            }
                            _ => expr,
                        };
                        Some(implicit_move(wrapped))
                    }
                    None => None,
                };
                Ok(HirStmt::Return { value: hir_value })
            }
            Stmt::If { cond, then_block, elifs, else_block, .. } => {
                let hir_cond = self.lower_expr(cond)?;
                let hir_then = self.lower_block(then_block)?;
                let hir_elifs = elifs.iter()
                    .map(|(ec, eb)| {
                        let c = self.lower_expr(ec);
                        let b = self.lower_block(eb);
                        c.and_then(|c| b.map(|b| (c, b)))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let hir_else = match else_block {
                    Some(b) => Some(self.lower_block(b)?),
                    None => None,
                };
                Ok(HirStmt::If {
                    cond: hir_cond,
                    then_block: hir_then,
                    elifs: hir_elifs,
                    else_block: hir_else,
                })
            }
            Stmt::For { iterator, start, end, step, body, .. } => {
                self.lower_for(*iterator, start, end, step.as_ref(), body)
            }
            Stmt::While { cond, body, .. } => {
                let hir_cond = self.lower_expr(cond)?;
                let hir_body = self.lower_block(body)?;
                Ok(HirStmt::While { cond: hir_cond, body: hir_body })
            }
            Stmt::Break { .. } => Ok(HirStmt::Break),
            Stmt::Continue { .. } => Ok(HirStmt::Continue),
            Stmt::ExprStmt { expr, .. } => {
                let hir_expr = self.lower_expr(expr)?;
                Ok(HirStmt::Expr(hir_expr))
            }
            Stmt::Namespace { .. } | Stmt::FnDecl { .. } | Stmt::StructDef { .. } | Stmt::InterfaceDef { .. } | Stmt::ImplBlock { .. } | Stmt::Import { .. } => {
                let s = stmt.span();
                Err(format!("unexpected declaration inside function body (at {}:{})", s.start_line, s.start_col))
            }
        }
    }

    pub(super) fn lower_for(
        &mut self,
        iter_name: Symbol,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        body: &Block,
    ) -> Result<HirStmt, String> {
        self.push_scope();

        let hir_start = self.lower_expr(start)?;
        let ty = expr_type(&hir_start);
        let (var_id, _, _) = self.register_or_lookup(iter_name, ty.clone());

        let init = HirStmt::Assign {
            target: HirExpr::Local(var_id, ty.clone()),
            value: hir_start,
        };

        let hir_end = self.lower_expr(end)?;
        let cond = HirExpr::Binary {
            op: BinaryOp::Lt,
            lhs: Box::new(HirExpr::Local(var_id, ty.clone())),
            rhs: Box::new(hir_end),
            ty: ty.clone(),
        };

        let hir_body = self.lower_block(body)?;
        let mut body_stmts = hir_body.stmts;

        let step_expr = match step {
            Some(s) => self.lower_expr(s)?,
            None => HirExpr::Literal(HirLiteral::Int(1), HirType::Int),
        };
        let step_ty = expr_type(&step_expr);
        body_stmts.push(HirStmt::Assign {
            target: HirExpr::Local(var_id, ty.clone()),
            value: HirExpr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(HirExpr::Local(var_id, ty.clone())),
                rhs: Box::new(step_expr),
                ty: step_ty,
            },
        });

        self.pop_scope();

        Ok(HirStmt::Block(vec![
            init,
            HirStmt::While { cond, body: HirBlock::new(body_stmts) },
        ]))
    }

    // ----------------------------------------------------------------
    //  表达式降级：将 AST 表达式递归降级为 HIR 表达式
    //  处理字面量、标识符、二元/一元运算、函数/方法调用、
    //  字段访问、结构体/数组字面量、指针比较等
    // ----------------------------------------------------------------

    pub(super) fn lower_expr(&mut self, expr: &Expr) -> Result<HirExpr, String> {
        match expr {
            Expr::Literal(lit) => self.lower_literal(lit),
            Expr::Ident(name, span) => {
                let (var_id, ty, _) = self.lookup_var(name)
                    .ok_or_else(|| format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col))?;
                Ok(HirExpr::Local(var_id, ty))
            }
            Expr::Binary { op, lhs, rhs, span } => {
                let hir_lhs = self.lower_expr(lhs)?;
                let hir_rhs = self.lower_expr(rhs)?;
                let lhs_ty = expr_type(&hir_lhs);
                let rhs_ty = expr_type(&hir_rhs);
                let inner_ty = strip_ownership(lhs_ty.clone());
                // Detect null-vs-pointer comparison (null is lowered to Int(0))
                // Only treat as pointer comparison when the non-null side's inner type is NOT primitive
                // (e.g. Shared(Node) vs null, but NOT Unique(Int) == 0 — int is passed by value)
                let lhs_is_null = is_null_literal(&hir_lhs);
                let rhs_is_null = is_null_literal(&hir_rhs);
                let non_null_ty = if lhs_is_null { &rhs_ty } else { &lhs_ty };
                let is_null_ptr_cmp = (lhs_is_null || rhs_is_null)
                    && is_pointer_type_for_cmp(non_null_ty)
                    && !matches!(strip_ownership(non_null_ty.clone()), HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
                // Try operator overloading first: look for a matching function
                // Primitive types use built-in operators, not overloading
                // Null-vs-pointer comparisons use built-in ptr comparison, not overloading
                let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
                if !is_primitive && !is_null_ptr_cmp {
                    if let Some(op_fn_name) = binary_op_to_fn_name(op) {
                        let param_types = [lhs_ty.clone(), rhs_ty];
                        let fn_id = match self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                            Some(fid) => fid,
                            None => {
                                match self.specialize_generic_call(&Symbol::intern(op_fn_name), &param_types, span) {
                                    Ok(fid) => fid,
                                    Err(msg) => { return Err(msg); }
                                }
                            }
                        };
                        let ret_ty = self.fns[fn_id.0].return_type.clone();
                        let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                        let args = vec![hir_lhs, hir_rhs].into_iter().enumerate().map(|(i, arg)| {
                            if i >= param_tys.len() { return arg; }
                            wrap_arg_for_param(arg, &param_tys[i])
                        }).collect();
                        return Ok(HirExpr::Call { fn_id, args, ty: ret_ty });
                    }
                }
                // Fall back to built-in operator
                let binop_ty = if is_null_ptr_cmp {
                    // Use the non-null side's type so the LIR emitter detects pointer comparison
                    if lhs_is_null { rhs_ty.clone() } else { lhs_ty.clone() }
                } else {
                    inner_ty
                };
                Ok(HirExpr::Binary {
                    op: *op,
                    lhs: Box::new(hir_lhs),
                    rhs: Box::new(hir_rhs),
                    ty: binop_ty,
                })
            }
            Expr::Unary { op, arg, .. } => {
                let hir_arg = self.lower_expr(arg)?;
                let arg_ty = expr_type(&hir_arg);
                let inner_ty = strip_ownership(arg_ty.clone());
                // Try operator overloading (skip for primitive types)
                let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
                if !is_primitive {
                    if let Some(op_fn_name) = unary_op_to_fn_name(op) {
                        let param_types = [arg_ty.clone()];
                        if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                            let ret_ty = self.fns[fn_id.0].return_type.clone();
                            return Ok(HirExpr::Call { fn_id, args: vec![implicit_move(hir_arg)], ty: ret_ty });
                        }
                    }
                }
                let ty = strip_ownership(arg_ty);
                Ok(HirExpr::Unary {
                    op: *op,
                    arg: Box::new(hir_arg),
                    ty,
                })
            }
            Expr::FnCall { name, args, span } => {
                // Step 1: lower all arguments
                let mut hir_args: Vec<HirExpr> = args.iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // Step 2: extract arg types
                let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();

                // Step 3: resolve overloaded function
                let fn_id = match self.resolve_fn_call(name, &arg_types) {
                    Some(fid) => fid,
                    None => {
                        // Step 3b: try generic specialization
                        self.specialize_generic_call(name, &arg_types, span)?
                    }
                };

                // Step 4: wrap args into fat pointers where needed, apply implicit moves
                // Pre-register vtables for generic impl → interface (before the closure that can't use ?)
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                for (i, arg) in hir_args.iter().enumerate() {
                    if i >= param_tys.len() { break; }
                    if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                        let arg_ty = expr_type(arg);
                        if let Some(ct) = Self::extract_concrete_type_name(&arg_ty) {
                            let base_ct = crate::hir::lower::strip_generic_name(&ct);
                            if !self.type_ifaces.contains_key(&ct) || !self.type_ifaces[&ct].contains(iface_name) {
                                if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                                    && self.type_ifaces[&base_ct].contains(iface_name) {
                                    // already registered
                                } else if self.check_generic_fns_for_iface(&base_ct, iface_name) {
                                    self.register_generic_vtable(&ct, &base_ct, iface_name)?;
                                }
                            }
                        }
                    }
                }
                hir_args = hir_args.into_iter().enumerate().map(|(i, arg)| {
                    if i >= param_tys.len() { return arg; }
                    let arg_ty = expr_type(&arg);
                    // Check if param expects FatPtr and arg is a concrete type that implements the interface
                    if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                        let concrete_type = match &arg_ty {
                            HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => {
                                if let HirType::Named(n) = inner.as_ref() { Some(*n) } else { None }
                            }
                            HirType::Named(n) => Some(*n),
                            HirType::Int => Some(Symbol::intern("int")),
                            HirType::Float => Some(Symbol::intern("float")),
                            HirType::Char => Some(Symbol::intern("char")),
                            HirType::Bool => Some(Symbol::intern("bool")),
                            _ => None,
                        };
                        if let Some(ct) = concrete_type {
                            if self.type_ifaces.contains_key(&ct)
                                && self.type_ifaces[&ct].contains(iface_name) {
                                // Build fat pointer
                                let fatptr_ty = param_tys[i].clone();
                                return HirExpr::MakeFatPtr {
                                    value: Box::new(arg),
                                    concrete_type: ct,
                                    interface_name: *iface_name,
                                    ty: fatptr_ty,
                                };
                            }
                            // Also check stripped base name
                            let base_ct = crate::hir::lower::strip_generic_name(&ct);
                            if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                                && self.type_ifaces[&base_ct].contains(iface_name) {
                                let fatptr_ty = param_tys[i].clone();
                                return HirExpr::MakeFatPtr {
                                    value: Box::new(arg),
                                    concrete_type: ct,
                                    interface_name: *iface_name,
                                    ty: fatptr_ty,
                                };
                            }
                            // try full concrete type (already registered by pre-check above)
                            if self.type_ifaces.contains_key(&ct) && self.type_ifaces[&ct].contains(iface_name) {
                                let fatptr_ty = param_tys[i].clone();
                                return HirExpr::MakeFatPtr {
                                    value: Box::new(arg),
                                    concrete_type: ct,
                                    interface_name: *iface_name,
                                    ty: fatptr_ty,
                                };
                            }
                        }
                    }
                    if matches!(param_tys[i], HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_)) {
                        wrap_arg_for_param(arg, &param_tys[i])
                    } else {
                        arg
                    }
                }).collect();

                let ty = self.fns[fn_id.0].return_type.clone();
                Ok(HirExpr::Call { fn_id, args: hir_args, ty })
            }
            Expr::CallExpr { target, args, span } => {
                // Function call on arbitrary expression: look for `call` method
                let hir_target = self.lower_expr(target)?;
                let target_ty = expr_type(&hir_target);
                let hir_args: Vec<HirExpr> = args.iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
                let all_types = std::iter::once(target_ty.clone()).chain(arg_types.clone()).collect::<Vec<_>>();
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern("call"), &all_types) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                    let mut all_args = vec![hir_target];
                    all_args.extend(hir_args);
                    all_args = all_args.into_iter().enumerate().map(|(i, arg)| {
                        if i >= param_tys.len() { return arg; }
                        wrap_arg_for_param(arg, &param_tys[i])
                    }).collect();
                    return Ok(HirExpr::Call { fn_id, args: all_args, ty: ret_ty });
                }
                Err(format!("type `{}` cannot be called as a function at {}:{}",
                    hir_type_display(&target_ty), span.start_line, span.start_col))
            }
            Expr::TryOp(inner, span) => {
                let hir_inner = self.lower_expr(inner)?;
                let inner_ty = expr_type(&hir_inner);
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern("try_unwrap"), &[inner_ty.clone()]) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    return Ok(HirExpr::Call { fn_id, args: vec![hir_inner], ty: ret_ty });
                }
                Err(format!("type `{:?}` cannot use `?` operator at {}:{}", inner_ty, span.start_line, span.start_col))
            }
            Expr::MethodCall { object, method, args, span } => {
                // Lower the receiver first
                let receiver = self.lower_expr(object)?;
                let receiver_ty = expr_type(&receiver);

                // Lower call arguments
                let hir_args: Vec<HirExpr> = args.iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();

                // Check if receiver type is a fat pointer (interface dispatch)
                // receiver_ty may be wrapped in ownership (e.g. Shared(FatPtr))
                let receiver_inner = strip_ownership_ref(&receiver_ty);
                if let HirType::FatPtr { name: iface, .. } = receiver_inner {
                    // Virtual dispatch through interface
                    let iface_name = *iface;
                    let iface_reg = self.interfaces.get(&iface_name)
                        .ok_or_else(|| format!("unknown interface `{}` used as type (at {}:{})", iface_name, span.start_line, span.start_col))?;

                    let method_idx = iface_reg.methods.iter()
                        .position(|m| m.name == *method)
                        .ok_or_else(|| format!("interface `{}` has no method `{}` (at {}:{})", iface_name, method, span.start_line, span.start_col))?;

                    let ret_ty = iface_reg.methods[method_idx].return_type.clone();
                    return Ok(HirExpr::VirtualCall {
                        receiver: Box::new(receiver),
                        interface: iface_name,
                        method_index: method_idx,
                        args: hir_args,
                        ty: ret_ty,
                    });
                }

                // Static dispatch: find method by receiver type
                let fn_id = match self.resolve_method(&receiver_ty, method, &arg_types) {
                    Some(id) => id,
                    None => {
                        let mut all_param_types = vec![receiver_ty.clone()];
                        all_param_types.extend(arg_types.iter().cloned());
                        let fid = self.specialize_generic_call(method, &all_param_types, span)?;
                        // 泛型推导成功后，尝试更新接收者变量的类型
                        if let HirExpr::Local(var_id, _) = &receiver {
                            let spec_param_ty = &self.fns[fid.0].params[0].1;
                            let recv_stripped = strip_ownership_ref(&receiver_ty);
                            let spec_stripped = strip_ownership_ref(spec_param_ty);
                            if let (HirType::Named(rn), HirType::Named(sn)) = (recv_stripped, spec_stripped) {
                                let rs = rn.as_str();
                                let ss = sn.as_str();
                                if !rs.contains('<') && ss.contains('<') {
                                    let base = crate::hir::lower::strip_generic_name(&sn);
                                    if base.as_str() == rs {
                                        self.update_var_type(*var_id, spec_param_ty.clone());
                                    }
                                }
                            }
                        }
                        fid
                    }
                };

                // Apply implicit moves and ownership conversions on all args (including receiver)
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                let mut all_args: Vec<HirExpr> = vec![receiver];
                all_args.extend(hir_args);
                all_args = all_args.into_iter().enumerate().map(|(i, arg)| {
                    if i >= param_tys.len() { return arg; }
                    wrap_arg_for_param(arg, &param_tys[i])
                }).collect();

                let ty = self.fns[fn_id.0].return_type.clone();
                Ok(HirExpr::Call { fn_id, args: all_args, ty })
            }
            Expr::Move(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = expr_type(&hir_inner);
                Ok(HirExpr::Move(Box::new(hir_inner), ty))
            }
            Expr::Clone(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = expr_type(&hir_inner);
                // For heap types (Shared/Unique wrapping Named/Array), deep copy via ToUnique/ToShared
                match &ty {
                    HirType::Shared(inner_ty) if needs_deep_copy(inner_ty) => {
                        let new_ty = HirType::Shared(inner_ty.clone());
                        Ok(HirExpr::ToShared(Box::new(HirExpr::Clone(Box::new(hir_inner), ty.clone())), new_ty))
                    }
                    HirType::Unique(inner_ty) if needs_deep_copy(inner_ty) => {
                        let new_ty = HirType::Unique(inner_ty.clone());
                        Ok(HirExpr::ToUnique(Box::new(HirExpr::Clone(Box::new(hir_inner), ty.clone())), new_ty))
                    }
                    _ if needs_deep_copy(&ty) => {
                        // Plain heap value: clone produces a Unique copy
                        let new_ty = HirType::Unique(Box::new(ty.clone()));
                        Ok(HirExpr::ToUnique(Box::new(HirExpr::Clone(Box::new(hir_inner), ty.clone())), new_ty))
                    }
                    _ => Ok(HirExpr::Clone(Box::new(hir_inner), ty)),
                }
            }
            Expr::ToUnique(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = expr_type(&hir_inner);
                let ty = HirType::Unique(Box::new(strip_ownership(inner_ty)));
                Ok(HirExpr::ToUnique(Box::new(hir_inner), ty))
            }
            Expr::ToShared(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = expr_type(&hir_inner);
                let ty = HirType::Shared(Box::new(strip_ownership(inner_ty)));
                Ok(HirExpr::ToShared(Box::new(hir_inner), ty))
            }
            Expr::ToWeak(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = expr_type(&hir_inner);
                let ty = HirType::Weak(Box::new(strip_ownership(inner_ty)));
                Ok(HirExpr::ToWeak(Box::new(hir_inner), ty))
            }
            Expr::FieldAccess { object, field, span: expr_span } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                // Resolve field index from struct definition
                let field_index = self.find_field_index(&object_ty, field, expr_span)?;
                let field_ty = self.find_field_type(&object_ty, field, expr_span)?;
                Ok(HirExpr::FieldAccess {
                    object: Box::new(hir_object),
                    field: *field,
                    field_index,
                    ty: field_ty,
                })
            }
            Expr::StructLiteral { type_name, generic_args, fields, .. } => {
                // Handle generic struct instantiation
                let concrete_name = if !generic_args.is_empty() {
                    let args_str: Vec<String> = generic_args.iter()
                        .map(|a| type_to_string_generic(a, &self.interfaces))
                        .collect();
                    let encoded = format!("{}<{}>", type_name, args_str.join(","));
                    let name_sym = Symbol::intern(&encoded);
                    // Monomorphize: create concrete struct def if not exists
                    if !self.struct_defs.contains_key(&name_sym) {
                        if let Some(generic_fields) = self.struct_defs.get(type_name) {
                            // Build substitution map: T → concrete type
                            let mut generic_params = self.collected_generic_params(type_name);
                            // 若 generic_struct_params 未从 .lcl 合并，则从字段类型推断 GP 名称
                            if generic_params.is_empty() && !generic_args.is_empty() {
                                generic_params = generic_args.iter().enumerate()
                                    .map(|(i, _)| (Symbol::intern(&format!("_G{}", i)), None))
                                    .collect();
                                // 尝试从字段类型中提取实际 GP 名称（单字母大写名）
                                if let Some(fields) = self.struct_defs.get(type_name) {
                                    for field in fields {
                                        if let HirType::Named(n) = strip_ownership_ref(&field.ty) {
                                            let s = n.as_str();
                                            if s.len() == 1 && s.chars().all(|c| c.is_uppercase()) {
                                                generic_params = vec![(*n, None)];
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            let mut subst: HashMap<Symbol, HirType> = HashMap::new();
                            for ((gp_name, _), concrete_ty) in generic_params.iter().zip(generic_args.iter()) {
                                let hir_ty = ast_type_to_hir(concrete_ty, &self.interfaces);
                                subst.insert(*gp_name, hir_ty);
                            }
                            // Substitute field types
                            let concrete_fields: Vec<HirStructField> = generic_fields.iter()
                                .map(|f| {
                                    let new_ty = substitute_hir_type(&f.ty, &subst);
                                    if &f.ty != &new_ty {
                                    }
                                    HirStructField { name: f.name, ty: new_ty }
                                })
                                .collect();
                            self.struct_defs.insert(name_sym, concrete_fields);
                        }
                    }
                    name_sym
                } else {
                    *type_name
                };
                let struct_ty = HirType::Named(concrete_name);
                let mut hir_fields = Vec::new();
                for (name, expr) in fields {
                    let hir_val = self.lower_expr(expr)?;
                    hir_fields.push((*name, hir_val));
                }
                Ok(HirExpr::StructLiteral {
                    type_name: concrete_name,
                    fields: hir_fields,
                    ty: struct_ty,
                })
            }
            Expr::ArrayLiteral(elems, span) => {
                if !self.allow_bare_array {
                    return Err(format!("array literal must be prefixed with `shared`, `unique`, or `weak` (at {}:{})", span.start_line, span.start_col));
                }
                let mut hir_elems = Vec::new();
                for e in elems {
                    hir_elems.push(self.lower_expr(e)?);
                }
                let elem_ty = if !hir_elems.is_empty() {
                    strip_ownership(expr_type(&hir_elems[0]))
                } else {
                    HirType::Int
                };
                Ok(HirExpr::ArrayLiteral(hir_elems, HirType::Array(Box::new(elem_ty))))
            }
            Expr::Index { object, index, span } => {
                let hir_object = self.lower_expr(object)?;
                let hir_index = self.lower_expr(index)?;
                let object_ty = expr_type(&hir_object);
                let inner_ty = strip_ownership(object_ty.clone());
                // Virtual dispatch through interface
                if let HirType::FatPtr { name: iface, .. } = &inner_ty {
                    let iface_reg = self.interfaces.get(iface)
                        .ok_or_else(|| format!("unknown interface `{}` used as type (at {}:{})", iface, span.start_line, span.start_col))?;
                    let method_idx = iface_reg.methods.iter()
                        .position(|m| m.name == Symbol::intern("index"))
                        .ok_or_else(|| format!("interface `{}` has no method `index` (at {}:{})", iface, span.start_line, span.start_col))?;
                    let ret_ty = iface_reg.methods[method_idx].return_type.clone();
                    return Ok(HirExpr::VirtualCall {
                        receiver: Box::new(hir_object),
                        interface: *iface,
                        method_index: method_idx,
                        args: vec![hir_index],
                        ty: ret_ty,
                    });
                }
                // Try operator overloading: index(self, index)
                let index_ty = expr_type(&hir_index);
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern("index"), &[object_ty.clone(), index_ty.clone()]) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                    let args = vec![hir_object, hir_index].into_iter().enumerate().map(|(i, arg)| {
                        if i >= param_tys.len() { return arg; }
                        wrap_arg_for_param(arg, &param_tys[i])
                    }).collect();
                    return Ok(HirExpr::Call { fn_id, args, ty: ret_ty });
                }
                if let Ok(fn_id) = self.specialize_generic_call(&Symbol::intern("index"), &[object_ty.clone(), index_ty.clone()], span) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                    let args = vec![hir_object, hir_index].into_iter().enumerate().map(|(i, arg)| {
                        if i >= param_tys.len() { return arg; }
                        wrap_arg_for_param(arg, &param_tys[i])
                    }).collect();
                    return Ok(HirExpr::Call { fn_id, args, ty: ret_ty });
                }
                // Fallback to built-in array index
                let elem_ty = match &inner_ty {
                    HirType::Array(inner) => *inner.clone(),
                    _ => return Err(format!("index on non-array type at {}:{}", span.start_line, span.start_col)),
                };
                Ok(HirExpr::Index {
                    object: Box::new(hir_object),
                    index: Box::new(hir_index),
                    ty: elem_ty,
                })
            }
            Expr::Null(_) => {
                // Null value — lowered as a zero int; will be cast to ptr at use site
                Ok(HirExpr::Literal(HirLiteral::Int(0), HirType::Int))
            }
            Expr::Ref(inner, mutable, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let inner_ty = expr_type(&hir_inner);
                let ty = HirType::Ref(Box::new(inner_ty), *mutable);
                Ok(HirExpr::Ref { expr: Box::new(hir_inner), mutable: *mutable, ty })
            }
            Expr::ArraySized { elem_type, count, .. } => {
                let hir_count = self.lower_expr(count)?;
                let elem_ty = ast_type_to_hir(elem_type, &self.interfaces);
                let ty = HirType::Array(Box::new(elem_ty.clone()));
                Ok(HirExpr::ArraySized { count: Box::new(hir_count), elem_ty, ty })
            }
            Expr::Asm { template, outputs, inputs, .. } => {
                let lowered_outputs: Vec<(String, Box<HirExpr>)> = outputs.iter().map(|(c, e)| {
                    (c.clone(), Box::new(self.lower_expr(e).unwrap()))
                }).collect();
                let lowered_inputs: Vec<(String, Box<HirExpr>)> = inputs.iter().map(|(c, e)| {
                    (c.clone(), Box::new(self.lower_expr(e).unwrap()))
                }).collect();
                let ty = if !lowered_outputs.is_empty() {
                    expr_type(&lowered_outputs[0].1)
                } else {
                    HirType::Void
                };
                Ok(HirExpr::Asm {
                    template: template.clone(),
                    outputs: lowered_outputs,
                    inputs: lowered_inputs,
                    ty,
                })
            }
        }
    }

    pub(super) fn lower_literal(&mut self, lit: &Literal) -> Result<HirExpr, String> {
        match lit {
            Literal::Int(n, _) => Ok(HirExpr::Literal(HirLiteral::Int(*n), HirType::Int)),
            Literal::Float(n, _) => Ok(HirExpr::Literal(HirLiteral::Float(*n), HirType::Float)),
            Literal::Char(c, _) => Ok(HirExpr::Literal(HirLiteral::Char(*c), HirType::Char)),
            Literal::String(s, _) => Ok(HirExpr::Literal(HirLiteral::String(s.clone()), HirType::Named(Symbol::intern("String")))),
            Literal::Bool(b, _) => Ok(HirExpr::Literal(HirLiteral::Bool(*b), HirType::Bool)),
        }
    }
}