use std::collections::HashMap;

use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;

use super::ir::*;

pub fn lower_program(program: &Program) -> Result<HirProgram, String> {
    let mut ctx = Ctx::new();

    // Phase 1: register all function and interface signatures
    ctx.collect_fns(&program.stmts)?;

    // Phase 1.5: build vtable map (validates impl signatures against interfaces)
    ctx.build_vtables()?;

    // Phase 2: lower all top-level items
    let mut items = ctx.lower_items(&program.stmts)?;

    // Collect imported function sigs (those not in HirItems)
    let defined_ids: std::collections::HashSet<_> = items.iter().filter_map(|item| {
        if let HirItem::Fn(f) = item { Some(f.fn_id) } else { None }
    }).collect();
    let imported_fns: Vec<ImportedFnSig> = ctx.fns.iter().enumerate()
        .filter(|(i, _)| !defined_ids.contains(&FnId(*i)))
        .map(|(i, sig)| ImportedFnSig {
            fn_id: FnId(i),
            name: sig.name,
            params: sig.params.clone(),
            return_type: sig.return_type.clone(),
        })
        .collect();

    // Append any specialized generic functions discovered during lowering
    for f in ctx.specialized_fns.drain(..) {
        items.push(HirItem::Fn(f));
    }

    Ok(HirProgram { items, vtables: ctx.vtables.clone(), struct_defs: ctx.struct_defs.clone(), imported_fns })
}

// ====================================================================
//  Internal context
// ====================================================================

struct FnSig {
    name: Symbol,
    params: Vec<(Symbol, HirType)>,
    return_type: HirType,
}

#[derive(Clone)]
struct InterfaceReg {
    methods: Vec<HirInterfaceMethod>,
}

struct Ctx {
    /// All registered functions (indexed by FnId)
    fns: Vec<FnSig>,
    /// Map from function name → list of FnIds (supports overloading)
    fn_map: HashMap<Symbol, Vec<FnId>>,
    /// Registered interfaces: name → method signatures
    interfaces: HashMap<Symbol, InterfaceReg>,
    /// Vtable map: (concrete_type, interface) → list of FnIds (index 0 = drop, 1.. = methods)
    vtables: Vec<VtableEntry>,
    /// Quick lookup: concrete_type_name → which interfaces it implements
    type_ifaces: HashMap<Symbol, Vec<Symbol>>,

    /// Struct definitions: name → fields
    struct_defs: HashMap<Symbol, Vec<HirStructField>>,
    /// Generic struct params: name → [(param_name, constraint)]
    generic_struct_params: HashMap<Symbol, Vec<(Symbol, Option<Symbol>)>>,

    /// Generic function ASTs: (name, generic_params, Stmt::FnDecl)
    generic_fns: Vec<(Symbol, Vec<(Symbol, Option<Symbol>)>, Stmt)>,
    /// Specialized (monomorphized) functions created during lowering
    specialized_fns: Vec<HirFn>,

    /// Current function being lowered
    current_fn: FnId,
    /// Locals of current function
    locals: Vec<HirLocal>,
    /// Scope stack: each scope maps name → (VarId, type, mutable)
    scopes: Vec<HashMap<Symbol, (VarId, HirType, bool)>>,
    /// Whether bare array literals are allowed (set inside ToShared/ToUnique/ToWeak)
    allow_bare_array: bool,
}

impl Ctx {
    fn new() -> Self {
        Self {
            fns: Vec::new(),
            fn_map: HashMap::new(),
            interfaces: HashMap::new(),
            vtables: Vec::new(),
            type_ifaces: HashMap::new(),
            struct_defs: HashMap::new(),
            generic_struct_params: HashMap::new(),
            generic_fns: Vec::new(),
            specialized_fns: Vec::new(),
            current_fn: FnId(0),
            locals: Vec::new(),
            scopes: Vec::new(),
            allow_bare_array: false,
        }
    }

    // ----------------------------------------------------------------
    // Phase 1: collect function and interface signatures
    // ----------------------------------------------------------------

    fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        self.collect_fns_with_ns(stmts, "")
    }

    fn collect_fns_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<(), String> {
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
                Stmt::InterfaceDef { name, methods, .. } => {
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
                    self.interfaces.insert(*name, InterfaceReg { methods: hir_methods });
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
                    let (imported_syms, _, _, _) = crate::package::load_package(path)
                        .map_err(|e| format!("import error: {}", e))?;
                    let _ = imported_syms;
                    for sym in &imported_syms {
                        match sym {
                            crate::package::ImportedSymbol::Fn { name, sig } => {
                                // sig format: "fnName(param_types...)->ret_type"
                                let sig_body = sig.trim_start_matches(name.as_str());
                                let arrow_pos = sig_body.find(")->")
                                    .ok_or_else(|| format!("invalid fn sig in package '{}'", name))?;
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
                                let fn_id = FnId(self.fns.len());
                                self.fns.push(FnSig {
                                    name: Symbol::intern(name),
                                    params: hir_params,
                                    return_type: hir_ret,
                                });
                                self.fn_map.entry(Symbol::intern(name)).or_default().push(fn_id);
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
                                self.struct_defs.entry(Symbol::intern(&struct_name)).or_insert_with(Vec::new).extend(fields);
                            }
                            crate::package::ImportedSymbol::Namespace { .. } => {
                                // Handled by lowering; just register the path
                            }
                        }
                    }
                }
                Stmt::ImplBlock { methods, .. } => {
                    // Register impl block methods as regular functions
                    for method in methods {
                        // methods inside impl blocks are already Stmt::FnDecl from the parser
                        if let Stmt::FnDecl { name, params, return_type, .. } = method {
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
                            return Err("unexpected non-FnDecl inside impl block".into());
                        }
                    }
                }
                _ => {
                    return Err(format!("unexpected top-level statement outside function, namespace, interface, or impl block"));
                }
            }
        }
        Ok(())
    }

    // ----------------------------------------------------------------
    // Phase 1.5: build vtable map
    // ----------------------------------------------------------------

    /// For each impl block type, check which interfaces it satisfies
    /// (structural typing: methods with same name + compatible signatures)
    fn build_vtables(&mut self) -> Result<(), String> {
        // Collect all impl types and their methods
        let mut impl_methods: HashMap<Symbol, Vec<&FnSig>> = HashMap::new();
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
                    impl_methods.entry(tn).or_default().push(sig);
                }
            }
        }

        for (iface_name, iface_reg) in &self.interfaces {
            for (type_name, methods) in &impl_methods {
                // Check if this type partially matches: has all methods by name
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
                        None => {
                            all_match = false;
                            break;
                        }
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
            }
        }
        Ok(())
    }

    fn type_matches(a: &HirType, b: &HirType) -> bool {
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
    fn find_fn_by_sig(&self, name: Symbol, param_types: &[HirType]) -> Option<FnId> {
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

    /// Check if an arg type can be passed to a param type (accounting for FatPtr wrapping)
    fn is_fatptr_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
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
            }
        }
        false
    }

    fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        if param_ty == arg_ty { return true; }
        // Shared/Unique value types: allow passing plain T to shared T
        if let HirType::Shared(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        if let HirType::Unique(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        // FatPtr compatibility
        self.is_fatptr_compatible(param_ty, arg_ty)
    }

    /// Resolve a function call by name and argument types (overload-aware)
    fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(name)?;
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
            None
        }
    }

    /// Resolve a method call: find function where first param matches receiver type
    /// Check if a receiver type matches a method's self param type (allowing Shared auto-wrap)
    fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool {
        if receiver == param { return true; }
        // Check inner types match (allowing ownership wrapper mismatch)
        let recv_inner = match receiver {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        let param_inner = match param {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if recv_inner == param_inner { return true; }
        // Allow passing plain T to shared/unique self (auto-wrap)
        match param {
            HirType::Shared(inner) | HirType::Unique(inner) => {
                if receiver == inner.as_ref() { return true; }
            }
            _ => {}
        }
        false
    }

    fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
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
    // Generic specialization
    // ----------------------------------------------------------------

    /// Try to resolve a call by specializing a generic function.
    /// Returns the FnId of the newly-created specialized function on success.
    fn specialize_generic_call(&mut self, name: &Symbol, arg_types: &[HirType], span: &crate::span::Span) -> Result<FnId, String> {
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
            return Err("internal error: generic function is not a FnDecl".into());
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
            if let Some((gp_name, hir_concrete)) = infer_generic_from_param(param_ty, arg_ty) {
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
                    .ok_or_else(|| format!("internal error: generic param `{}` not resolved", gp_name))?;
                let concrete_inner = strip_ownership_ref(concrete_ty);
                let concrete_type_name = match concrete_inner {
                    HirType::Named(n) => *n,
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
                    .map(|ifaces| ifaces.contains(iface_name))
                    .unwrap_or(false);
                if !implements {
                    // Also check if there's an impl block with matching methods
                    // For now, report the error
                    return Err(format!(
                        "type `{}` does not implement interface `{}` required by generic parameter `{}` at {}:{}",
                        hir_type_display(concrete_ty), iface_name, gp_name,
                        span.start_line, span.start_col
                    ));
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

        let hir_fn = self.lower_fn(fid, *name, &new_params, &new_return_type, &new_body, *is_inline, *extern_c)?;

        self.current_fn = saved_current_fn;
        self.locals = saved_locals;
        self.scopes = saved_scopes;

        self.specialized_fns.push(hir_fn);

        Ok(fid)
    }

    // ----------------------------------------------------------------
    // Phase 2: lower items
    // ----------------------------------------------------------------

    fn lower_items(&mut self, stmts: &[Stmt]) -> Result<Vec<HirItem>, String> {
        self.lower_items_with_ns(stmts, "")
    }

    fn lower_items_with_ns(&mut self, stmts: &[Stmt], ns_prefix: &str) -> Result<Vec<HirItem>, String> {
        let mut items = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, params, return_type, body, is_inline, extern_c, generic_params, .. } => {
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
                        .ok_or_else(|| format!("internal error: function `{}` not found", full_name))?;
                    let hir_fn = self.lower_fn(fn_id, full_name, params, return_type, body, *is_inline, *extern_c)?;
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
                Stmt::InterfaceDef { name, methods, .. } => {
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
                    items.push(HirItem::InterfaceDef { name: *name, methods: hir_methods });
                }
                Stmt::StructDef { name, fields, .. } => {
                    let hir_fields: Vec<HirStructField> = fields.iter()
                        .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                        .collect();
                    items.push(HirItem::StructDef(HirStructDef { name: *name, fields: hir_fields }));
                }
                Stmt::Import { .. } => {} // already handled in collect_fns
                Stmt::ImplBlock { methods, .. } => {
                    // Flatten impl block: lower each method as a regular Fn
                    for method_stmt in methods {
                        if let Stmt::FnDecl { name, params, return_type, body, .. } = method_stmt {
                            let ptypes: Vec<HirType> = params.iter()
                                .map(|(_, t)| ast_type_to_hir(t, &self.interfaces))
                                .collect();
                            let fn_id = self.find_fn_by_sig(*name, &ptypes)
                                .ok_or_else(|| format!("internal error: method `{}` not found", name))?;
                            let hir_fn = self.lower_fn(fn_id, *name, params, return_type, body, false, false)?;
                            items.push(HirItem::Fn(hir_fn));
                        }
                    }
                }
                _ => {
                    return Err(format!("unexpected top-level statement"));
                }
            }
        }
        Ok(items)
    }

    fn lower_fn(
        &mut self,
        fn_id: FnId,
        name: Symbol,
        ast_params: &[(Symbol, Type)],
        _return_type: &Type,
        body: &Block,
        is_inline: bool,
        extern_c: bool,
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
    // Scope helpers
    // ----------------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind_var(&mut self, name: Symbol, id: VarId, ty: HirType, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, (id, ty, mutable));
        }
    }

    fn lookup_var(&self, name: &Symbol) -> Option<(VarId, HirType, bool)> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn find_field_index(&self, struct_ty: &HirType, field: &Symbol) -> Result<usize, String> {
        let inner = strip_ownership_ref(struct_ty);
        let type_name = match inner {
            HirType::Named(n) => *n,
            _ => return Err(format!("cannot access field on non-struct type {:?}", struct_ty)),
        };
        let def = self.struct_defs.get(&type_name)
            .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
        def.iter().position(|f| &f.name == field)
            .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, field))
    }

    fn find_field_type(&self, struct_ty: &HirType, field: &Symbol) -> Result<HirType, String> {
        let inner = strip_ownership_ref(struct_ty);
        let type_name = match inner {
            HirType::Named(n) => *n,
            _ => return Err(format!("cannot access field on non-struct type {:?}", struct_ty)),
        };
        let def = self.struct_defs.get(&type_name)
            .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
        def.iter().find(|f| &f.name == field)
            .map(|f| f.ty.clone())
            .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, field))
    }

    fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Option<Symbol>)> {
        self.generic_struct_params.get(type_name).cloned().unwrap_or_default()
    }

    fn register_or_lookup(&mut self, name: Symbol, inferred_ty: HirType) -> (VarId, HirType, bool) {
        if let Some(existing) = self.lookup_var(&name) {
            return existing;
        }
        let var_id = VarId(self.locals.len());
        self.locals.push(HirLocal::new(name, inferred_ty.clone(), false));
        self.bind_var(name, var_id, inferred_ty.clone(), false);
        (var_id, inferred_ty, false)
    }

    // ----------------------------------------------------------------
    // Lower blocks & statements
    // ----------------------------------------------------------------

    fn lower_block(&mut self, block: &Block) -> Result<HirBlock, String> {
        self.push_scope();
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.lower_stmt(stmt)?);
        }
        self.pop_scope();
        Ok(HirBlock::new(stmts))
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt, String> {
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
            Stmt::FieldAssign { object, field, value, .. } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                let field_index = self.find_field_index(&object_ty, field)?;
                let field_ty = self.find_field_type(&object_ty, field)?;
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
                        Some(implicit_move(expr))
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
            Stmt::ExprStmt { expr, .. } => {
                let hir_expr = self.lower_expr(expr)?;
                Ok(HirStmt::Expr(hir_expr))
            }
            Stmt::Namespace { .. } | Stmt::FnDecl { .. } | Stmt::StructDef { .. } | Stmt::InterfaceDef { .. } | Stmt::ImplBlock { .. } | Stmt::Import { .. } => {
                Err("unexpected declaration inside function body".into())
            }
        }
    }

    fn lower_for(
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
            ty: HirType::Bool,
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
    // Lower expressions
    // ----------------------------------------------------------------

    fn lower_expr(&mut self, expr: &Expr) -> Result<HirExpr, String> {
        match expr {
            Expr::Literal(lit) => self.lower_literal(lit),
            Expr::Ident(name, span) => {
                let (var_id, ty, _) = self.lookup_var(name)
                    .ok_or_else(|| format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col))?;
                Ok(HirExpr::Local(var_id, ty))
            }
            Expr::Binary { op, lhs, rhs, .. } => {
                let hir_lhs = self.lower_expr(lhs)?;
                let hir_rhs = self.lower_expr(rhs)?;
                let lhs_ty = expr_type(&hir_lhs);
                let inner_ty = strip_ownership(lhs_ty.clone());
                // Try operator overloading first: look for a matching function
                // Primitive types use built-in operators, not overloading
                let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool);
                if !is_primitive {
                    if let Some(op_fn_name) = binary_op_to_fn_name(op) {
                        let rhs_ty = expr_type(&hir_rhs);
                        let param_types = [lhs_ty.clone(), rhs_ty];
                        if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                            let ret_ty = self.fns[fn_id.0].return_type.clone();
                            let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
                            let args = vec![hir_lhs, hir_rhs].into_iter().enumerate().map(|(i, arg)| {
                                if i >= param_tys.len() { return arg; }
                                wrap_arg_for_param(arg, &param_tys[i])
                            }).collect();
                            return Ok(HirExpr::Call { fn_id, args, ty: ret_ty });
                        }
                    }
                }
                // Fall back to built-in operator
                Ok(HirExpr::Binary {
                    op: *op,
                    lhs: Box::new(hir_lhs),
                    rhs: Box::new(hir_rhs),
                    ty: inner_ty,
                })
            }
            Expr::Unary { op, arg, .. } => {
                let hir_arg = self.lower_expr(arg)?;
                let arg_ty = expr_type(&hir_arg);
                // Try operator overloading
                if let Some(op_fn_name) = unary_op_to_fn_name(op) {
                    let param_types = [arg_ty.clone()];
                    if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                        let ret_ty = self.fns[fn_id.0].return_type.clone();
                        return Ok(HirExpr::Call { fn_id, args: vec![implicit_move(hir_arg)], ty: ret_ty });
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
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
                    .map(|(_, t)| t.clone())
                    .collect();
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
                        }
                    }
                    if matches!(param_tys[i], HirType::Unique(_)) {
                        wrap_for_unique_param(arg, &param_tys[i])
                    } else {
                        arg
                    }
                }).collect();

                let ty = self.fns[fn_id.0].return_type.clone();
                Ok(HirExpr::Call { fn_id, args: hir_args, ty })
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
                if let HirType::FatPtr { name: iface, .. } = &receiver_ty {
                    // Virtual dispatch through interface
                    let iface_name = *iface;
                    let iface_reg = self.interfaces.get(&iface_name)
                        .ok_or_else(|| format!("unknown interface `{}` used as type", iface_name))?;

                    let method_idx = iface_reg.methods.iter()
                        .position(|m| m.name == *method)
                        .ok_or_else(|| format!("interface `{}` has no method `{}`", iface_name, method))?;

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
                let fn_id = self.resolve_method(&receiver_ty, method, &arg_types)
                    .ok_or_else(|| format!("no method `{}` found for type {:?} at {}:{}", method, receiver_ty, span.start_line, span.start_col))?;

                // Apply implicit moves and ownership conversions on all args (including receiver)
                let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                let mut all_args: Vec<HirExpr> = vec![receiver];
                all_args.extend(hir_args);
                all_args = all_args.into_iter().enumerate().map(|(i, arg)| {
                    if i >= param_tys.len() { return arg; }
                    let arg_ty = expr_type(&arg);
                    match &param_tys[i] {
                        HirType::Unique(pt) | HirType::Shared(pt) | HirType::Weak(pt) => {
                            // If arg is a plain struct value and param expects ownership, auto-wrap
                            if arg_ty == *pt.as_ref() {
                                match &param_tys[i] {
                                    HirType::Unique(_) => {
                                        HirExpr::ToUnique(Box::new(arg), param_tys[i].clone())
                                    }
                                    HirType::Shared(_) => {
                                        HirExpr::ToShared(Box::new(arg), param_tys[i].clone())
                                    }
                                    HirType::Weak(_) => {
                                        HirExpr::ToWeak(Box::new(arg), param_tys[i].clone())
                                    }
                                    _ => arg,
                                }
                            } else if matches!(param_tys[i], HirType::Unique(_)) {
                                wrap_for_unique_param(arg, &param_tys[i])
                            } else {
                                arg
                            }
                        }
                        _ => arg,
                    }
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
            Expr::FieldAccess { object, field, .. } => {
                let hir_object = self.lower_expr(object)?;
                let object_ty = expr_type(&hir_object);
                // Resolve field index from struct definition
                let field_index = self.find_field_index(&object_ty, field)?;
                let field_ty = self.find_field_type(&object_ty, field)?;
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
                            let generic_params = self.collected_generic_params(type_name);
                            let mut subst: HashMap<Symbol, HirType> = HashMap::new();
                            for ((gp_name, _), concrete_ty) in generic_params.iter().zip(generic_args.iter()) {
                                subst.insert(*gp_name, ast_type_to_hir(concrete_ty, &self.interfaces));
                            }
                            // Substitute field types
                            let concrete_fields: Vec<HirStructField> = generic_fields.iter()
                                .map(|f| {
                                    let new_ty = substitute_hir_type(&f.ty, &subst);
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
            Expr::ArrayLiteral(elems, _) => {
                if !self.allow_bare_array {
                    return Err("array literal must be prefixed with `shared`, `unique`, or `weak`".into());
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
            Expr::Index { object, index, .. } => {
                let hir_object = self.lower_expr(object)?;
                let hir_index = self.lower_expr(index)?;
                let object_ty = strip_ownership(expr_type(&hir_object));
                let elem_ty = match &object_ty {
                    HirType::Array(inner) => *inner.clone(),
                    _ => return Err("index on non-array type".into()),
                };
                Ok(HirExpr::Index {
                    object: Box::new(hir_object),
                    index: Box::new(hir_index),
                    ty: elem_ty,
                })
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

    fn lower_literal(&mut self, lit: &Literal) -> Result<HirExpr, String> {
        match lit {
            Literal::Int(n, _) => Ok(HirExpr::Literal(HirLiteral::Int(*n), HirType::Int)),
            Literal::Float(n, _) => Ok(HirExpr::Literal(HirLiteral::Float(*n), HirType::Float)),
            Literal::Char(c, _) => Ok(HirExpr::Literal(HirLiteral::Char(*c), HirType::Char)),
            Literal::String(s, _) => Ok(HirExpr::Literal(HirLiteral::String(s.clone()), HirType::Named(Symbol::intern("String")))),
            Literal::Bool(b, _) => Ok(HirExpr::Literal(HirLiteral::Bool(*b), HirType::Bool)),
        }
    }
}

fn implicit_move(expr: HirExpr) -> HirExpr {
    let ty = expr_type(&expr);
    if matches!(ty, HirType::Unique(_))
        && !matches!(expr, HirExpr::Move(_, _) | HirExpr::Clone(_, _))
    {
        HirExpr::Move(Box::new(expr), ty)
    } else {
        expr
    }
}

/// Wrap an argument to match the expected parameter type (handles ownership conversion).
fn wrap_arg_for_param(arg: HirExpr, param_ty: &HirType) -> HirExpr {
    let arg_ty = expr_type(&arg);
    match param_ty {
        HirType::Unique(pt) | HirType::Shared(pt) | HirType::Weak(pt) => {
            if arg_ty == *pt.as_ref() {
                match param_ty {
                    HirType::Unique(_) => {
                        HirExpr::ToUnique(Box::new(arg), param_ty.clone())
                    }
                    HirType::Shared(_) => {
                        HirExpr::ToShared(Box::new(arg), param_ty.clone())
                    }
                    HirType::Weak(_) => {
                        HirExpr::ToWeak(Box::new(arg), param_ty.clone())
                    }
                    _ => arg,
                }
            } else if matches!(param_ty, HirType::Unique(_)) {
                wrap_for_unique_param(arg, param_ty)
            } else {
                arg
            }
        }
        _ => arg,
    }
}

/// Like implicit_move, but also wraps plain values when the param expects Unique.
fn wrap_for_unique_param(expr: HirExpr, param_ty: &HirType) -> HirExpr {
    let ty = expr_type(&expr);
    if matches!(param_ty, HirType::Unique(_))
        && !matches!(expr, HirExpr::Move(_, _) | HirExpr::Clone(_, _))
    {
        HirExpr::Move(Box::new(expr), ty)
    } else {
        expr
    }
}

// ====================================================================
//  Generic substitution: AST-level type replacement
// ====================================================================

/// Convert a HirType back to an AST Type (for substitution into generic body).
/// Uses Span::default() for synthetic nodes.
fn hir_type_to_ast_type(ty: &HirType) -> Type {
    let s = Span::default();
    match ty {
        HirType::Int => Type::Int(s),
        HirType::Float => Type::Float(s),
        HirType::Char => Type::Char(s),
        HirType::Bool => Type::Bool(s),
        HirType::Void => Type::Void(s),
        HirType::Named(n) => Type::Named(*n, s),
        HirType::Unique(inner) => Type::Unique(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Shared(inner) => Type::Shared(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Weak(inner) => Type::Weak(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Array(inner) => Type::Array(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FatPtr { name, .. } => Type::Named(*name, s),
        HirType::Ref(inner, mutable) => Type::Ref(Box::new(hir_type_to_ast_type(inner)), *mutable, s),
    }
}

/// Given an AST param type and the corresponding HirType from the lowered arg,
/// extract the binding for a generic parameter name (if the param type uses it).
fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Option<(Symbol, HirType)> {
    match (param_ty, arg_ty) {
        (Type::Named(name, _), _) => Some((*name, arg_ty.clone())),
        (Type::Unique(inner, _), HirType::Unique(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        (Type::Shared(inner, _), HirType::Shared(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        (Type::Weak(inner, _), HirType::Weak(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        // Param expects wrapper but arg is unwrapped (auto-wrap will handle)
        (Type::Unique(inner, _) | Type::Shared(inner, _) | Type::Weak(inner, _), _) => {
            infer_generic_from_param(inner, arg_ty)
        }
        _ => None,
    }
}

/// Substitute generic type parameters in an AST Type node.
fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type {
    let s = Span::default();
    match ty {
        Type::Named(name, _) => {
            if let Some(concrete) = subst.get(name) {
                concrete.clone()
            } else {
                Type::Named(*name, s)
            }
        }
        Type::Unique(inner, _) => Type::Unique(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Shared(inner, _) => Type::Shared(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Weak(inner, _) => Type::Weak(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Array(inner, _) => Type::Array(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Ref(inner, mutable, _) => Type::Ref(Box::new(substitute_type_in_type(inner, subst)), *mutable, s),
        Type::Int(_) => Type::Int(s),
        Type::Float(_) => Type::Float(s),
        Type::Char(_) => Type::Char(s),
        Type::Bool(_) => Type::Bool(s),
        Type::Void(_) => Type::Void(s),
        Type::Generic(name, args, _) => Type::Generic(*name, args.iter().map(|a| substitute_type_in_type(a, subst)).collect(), s),
        Type::Default => ty.clone(),
        Type::Self_(_) => ty.clone(),
    }
}

/// Substitute generic type parameters in an AST Expr.
fn substitute_type_in_expr(expr: &Expr, subst: &HashMap<Symbol, Type>) -> Expr {
    match expr {
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary {
            op: *op,
            lhs: Box::new(substitute_type_in_expr(lhs, subst)),
            rhs: Box::new(substitute_type_in_expr(rhs, subst)),
            span: *span,
        },
        Expr::Unary { op, arg, span } => Expr::Unary {
            op: *op,
            arg: Box::new(substitute_type_in_expr(arg, subst)),
            span: *span,
        },
        Expr::Literal(_) => expr.clone(),
        Expr::Ident(_, _) => expr.clone(),
        Expr::FnCall { name, args, span } => Expr::FnCall {
            name: *name,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
            span: *span,
        },
        Expr::MethodCall { object, method, args, span } => Expr::MethodCall {
            object: Box::new(substitute_type_in_expr(object, subst)),
            method: *method,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
            span: *span,
        },
        Expr::StructLiteral { type_name, generic_args, fields, span } => Expr::StructLiteral {
            type_name: *type_name,
            generic_args: generic_args.clone(),
            fields: fields.iter().map(|(n, e)| (*n, substitute_type_in_expr(e, subst))).collect(),
            span: *span,
        },
        Expr::ArrayLiteral(elems, span) => Expr::ArrayLiteral(
            elems.iter().map(|e| substitute_type_in_expr(e, subst)).collect(),
            *span,
        ),
        Expr::ArraySized { elem_type, count, span } => Expr::ArraySized {
            elem_type: substitute_type_in_type(elem_type, subst),
            count: Box::new(substitute_type_in_expr(count, subst)),
            span: *span,
        },
        Expr::Move(inner, span) => Expr::Move(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::Clone(inner, span) => Expr::Clone(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::ToUnique(inner, span) => Expr::ToUnique(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::ToShared(inner, span) => Expr::ToShared(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::ToWeak(inner, span) => Expr::ToWeak(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::FieldAccess { object, field, span } => Expr::FieldAccess {
            object: Box::new(substitute_type_in_expr(object, subst)),
            field: *field,
            span: *span,
        },
        Expr::Ref(inner, mutable, span) => Expr::Ref(
            Box::new(substitute_type_in_expr(inner, subst)),
            *mutable,
            *span,
        ),
        Expr::Asm { template, outputs, inputs, span } => Expr::Asm {
            template: template.clone(),
            outputs: outputs.iter().map(|(c, e)| (c.clone(), Box::new(substitute_type_in_expr(e, subst)))).collect(),
            inputs: inputs.iter().map(|(c, e)| (c.clone(), Box::new(substitute_type_in_expr(e, subst)))).collect(),
            span: *span,
        },
        Expr::Index { object, index, span } => Expr::Index {
            object: Box::new(substitute_type_in_expr(object, subst)),
            index: Box::new(substitute_type_in_expr(index, subst)),
            span: *span,
        },
    }
}

/// Substitute generic type parameters in an AST Block.
fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block {
    Block {
        stmts: block.stmts.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
        span: block.span,
    }
}

/// Substitute generic type parameters in an AST Stmt.
fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt {
    match stmt {
        Stmt::FnDecl { vis, is_inline, extern_c, name, generic_params: _, params, return_type, body, span } => {
            Stmt::FnDecl {
                vis: *vis,
                is_inline: *is_inline,
                extern_c: *extern_c,
                name: *name,
                generic_params: Vec::new(), // cleared: all generics are now concrete
                params: params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
                return_type: substitute_type_in_type(return_type, subst),
                body: substitute_type_in_block(body, subst),
                span: *span,
            }
        }
        Stmt::Assign { name, is_mut, value, span } => Stmt::Assign {
            name: *name,
            is_mut: *is_mut,
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::FieldAssign { object, field, value, span } => Stmt::FieldAssign {
            object: Box::new(substitute_type_in_expr(object, subst)),
            field: *field,
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::IndexAssign { object, index, value, span } => Stmt::IndexAssign {
            object: Box::new(substitute_type_in_expr(object, subst)),
            index: Box::new(substitute_type_in_expr(index, subst)),
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::Return { value, span } => Stmt::Return {
            value: value.as_ref().map(|v| substitute_type_in_expr(v, subst)),
            span: *span,
        },
        Stmt::If { cond, then_block, elifs, else_block, span } => Stmt::If {
            cond: substitute_type_in_expr(cond, subst),
            then_block: substitute_type_in_block(then_block, subst),
            elifs: elifs.iter().map(|(c, b)| (substitute_type_in_expr(c, subst), substitute_type_in_block(b, subst))).collect(),
            else_block: else_block.as_ref().map(|b| substitute_type_in_block(b, subst)),
            span: *span,
        },
        Stmt::For { iterator, start, end, step, body, span } => Stmt::For {
            iterator: *iterator,
            start: substitute_type_in_expr(start, subst),
            end: substitute_type_in_expr(end, subst),
            step: step.as_ref().map(|s| substitute_type_in_expr(s, subst)),
            body: substitute_type_in_block(body, subst),
            span: *span,
        },
        Stmt::While { cond, body, span } => Stmt::While {
            cond: substitute_type_in_expr(cond, subst),
            body: substitute_type_in_block(body, subst),
            span: *span,
        },
        Stmt::ExprStmt { expr, span } => Stmt::ExprStmt {
            expr: substitute_type_in_expr(expr, subst),
            span: *span,
        },
        Stmt::Namespace { vis, name, items, span } => Stmt::Namespace {
            vis: *vis,
            name: *name,
            items: items.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
        Stmt::StructDef { vis, name, fields, span, .. } => Stmt::StructDef {
            vis: *vis,
            name: *name,
            generic_params: Vec::new(),
            fields: fields.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
            span: *span,
        },
        Stmt::InterfaceDef { name, methods, span } => Stmt::InterfaceDef {
            name: *name,
            methods: methods.iter().map(|m| {
                InterfaceMethod {
                    name: m.name,
                    self_keyword: m.self_keyword,
                    params: m.params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
                    return_type: substitute_type_in_type(&m.return_type, subst),
                }
            }).collect(),
            span: *span,
        },
        Stmt::ImplBlock { type_name, methods, span } => Stmt::ImplBlock {
            type_name: *type_name,
            methods: methods.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
        Stmt::Import { path, span } => Stmt::Import { path: path.clone(), span: *span },
    }
}

// ====================================================================
//  Type helpers
// ====================================================================

/// Parse a type from a package signature string like "int", "shared Point", "[int]", etc.
/// Convert an AST Type to a string for generic encoding.
fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String {
    match ty {
        Type::Default => "?".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Generic(name, args, _) => {
            let a: Vec<String> = args.iter().map(|a| type_to_string_generic(a, interfaces)).collect();
            format!("{}<{}>", name, a.join(","))
        }
        Type::Array(inner, _) => format!("[{}]", type_to_string_generic(inner, interfaces)),
        Type::Ref(inner, mutable, _) => format!("ref{}{}",
            if *mutable { " mut" } else { "" },
            type_to_string_generic(inner, interfaces)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string_generic(inner, interfaces)),
        Type::Shared(inner, _) => format!("shared {}", type_to_string_generic(inner, interfaces)),
        Type::Weak(inner, _) => format!("weak {}", type_to_string_generic(inner, interfaces)),
        Type::Self_(_) => "Self".into(),
    }
}

/// Substitute generic type parameters in a HirType.
fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType {
    match ty {
        HirType::Named(s) => {
            if let Some(replacement) = subst.get(s) {
                replacement.clone()
            } else {
                ty.clone()
            }
        }
        HirType::Unique(inner) => HirType::Unique(Box::new(substitute_hir_type(inner, subst))),
        HirType::Shared(inner) => HirType::Shared(Box::new(substitute_hir_type(inner, subst))),
        HirType::Weak(inner) => HirType::Weak(Box::new(substitute_hir_type(inner, subst))),
        HirType::Array(inner) => HirType::Array(Box::new(substitute_hir_type(inner, subst))),
        HirType::Ref(inner, mutable) => HirType::Ref(Box::new(substitute_hir_type(inner, subst)), *mutable),
        HirType::FatPtr { name, kind } => HirType::FatPtr {
            name: *name,
            kind: Box::new(substitute_hir_type(kind, subst)),
        },
        _ => ty.clone(),
    }
}

fn sig_str_to_hir(s: &str) -> HirType {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("shared ") {
        HirType::Shared(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("unique ") {
        HirType::Unique(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("weak ") {
        HirType::Weak(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        HirType::Array(Box::new(sig_str_to_hir(inner)))
    } else {
        match s {
            "int" => HirType::Int,
            "float" => HirType::Float,
            "char" => HirType::Char,
            "bool" => HirType::Bool,
            "void" => HirType::Void,
            other => HirType::Named(Symbol::intern(other)),
        }
    }
}

fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType {
    match ty {
        Type::Default | Type::Int(_) => HirType::Int,
        Type::Float(_) => HirType::Float,
        Type::Char(_) => HirType::Char,
        Type::Bool(_) => HirType::Bool,
        Type::Void(_) => HirType::Void,
        Type::Array(inner, _) => HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Generic(name, args, _) => {
            // Encode generic instantiation as a unique named type
            let args_str: Vec<String> = args.iter()
                .map(|a| type_to_string_generic(a, interfaces))
                .collect();
            HirType::Named(Symbol::intern(&format!("{}<{}>", name, args_str.join(","))))
        }
        Type::Named(s, _) => {
            let name = s.as_str();
            if name == "int" { HirType::Int }
            else if name == "float" { HirType::Float }
            else if name == "char" { HirType::Char }
            else if name == "void" { HirType::Void }
            else if name == "bool" { HirType::Bool }
            else { HirType::Named(*s) }
        }
        Type::Unique(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if matches!(&inner_hir, HirType::Named(n) if interfaces.contains_key(n)) {
                // unique Interface → fat pointer
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Unique(Box::new(HirType::Void))) }
            } else {
                HirType::Unique(Box::new(inner_hir))
            }
        }
        Type::Shared(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if matches!(&inner_hir, HirType::Named(n) if interfaces.contains_key(n)) {
                // shared Interface → fat pointer
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Shared(Box::new(HirType::Void))) }
            } else {
                HirType::Shared(Box::new(inner_hir))
            }
        }
        Type::Weak(inner, _) => HirType::Weak(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Ref(inner, mutable, _) => HirType::Ref(Box::new(ast_type_to_hir(inner, interfaces)), *mutable),
        Type::Generic(name, args, _) => {
            let hir_args: Vec<HirType> = args.iter().map(|a| ast_type_to_hir(a, interfaces)).collect();
            HirType::Named(Symbol::intern(&format!("{}<{}>", name, hir_args.iter().map(hir_type_display).collect::<Vec<_>>().join(","))))
        }
        Type::Self_(_) => {
            // Self_ should not appear outside impl blocks since the parser
            // already fills in the concrete type
            HirType::Void
        }
    }
}

fn extract_named(ty: &HirType) -> Option<&Symbol> {
    match ty {
        HirType::Named(s) => Some(s),
        _ => None,
    }
}

fn hir_type_display(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::Named(s) => s.as_str().to_string(),
        HirType::Unique(inner) => format!("unique {}", hir_type_display(inner)),
        HirType::Shared(inner) => format!("shared {}", hir_type_display(inner)),
        HirType::Weak(inner) => format!("weak {}", hir_type_display(inner)),
        HirType::FatPtr { name, kind } => format!("{} {}", hir_type_display(kind), name.as_str()),
        HirType::Array(inner) => format!("[{}]", hir_type_display(inner)),
        HirType::Ref(inner, mutable) => {
            if *mutable {
                format!("ref mut {}", hir_type_display(inner))
            } else {
                format!("ref {}", hir_type_display(inner))
            }
        }
    }
}

/// Check if a type needs deep copy (heap-allocated data).
fn needs_deep_copy(ty: &HirType) -> bool {
    matches!(ty, HirType::Named(_) | HirType::Array(_) | HirType::FatPtr { .. })
}

/// Map binary operators to function names for operator overloading.
fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("add"),
        BinaryOp::Sub => Some("sub"),
        BinaryOp::Mul => Some("mul"),
        BinaryOp::Div => Some("div"),
        BinaryOp::Mod => Some("rem"),
        BinaryOp::Eq => Some("eq"),
        BinaryOp::Neq => Some("ne"),
        BinaryOp::Lt => Some("lt"),
        BinaryOp::Gt => Some("gt"),
        BinaryOp::Le => Some("le"),
        BinaryOp::Ge => Some("ge"),
        BinaryOp::And | BinaryOp::Or => None, // logical ops not overloadable
    }
}

fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str> {
    match op {
        UnaryOp::Neg => Some("neg"),
        UnaryOp::Not => Some("not"),
    }
}

fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => *inner,
        other => other,
    }
}

fn strip_ownership_ref(ty: &HirType) -> &HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
        other => other,
    }
}

fn expr_type(expr: &HirExpr) -> HirType {
    match expr {
        HirExpr::Literal(_, ty)
        | HirExpr::Local(_, ty)
        | HirExpr::Binary { ty, .. }
        | HirExpr::Unary { ty, .. }
        | HirExpr::Call { ty, .. }
        | HirExpr::Move(_, ty)
        | HirExpr::Clone(_, ty)
        | HirExpr::ToUnique(_, ty)
        | HirExpr::ToShared(_, ty)
        | HirExpr::ToWeak(_, ty)
        | HirExpr::VirtualCall { ty, .. }
        | HirExpr::MakeFatPtr { ty, .. }
        | HirExpr::FieldAccess { ty, .. }
        | HirExpr::StructLiteral { ty, .. }
        | HirExpr::ArraySized { ty, .. }
        | HirExpr::ArrayLiteral(_, ty)
        | HirExpr::Index { ty, .. }
        | HirExpr::Ref { ty, .. }
        | HirExpr::Asm { ty, .. } => ty.clone(),
    }
}
