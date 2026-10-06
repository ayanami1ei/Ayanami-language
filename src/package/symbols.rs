use super::*;
use super::load::type_to_string;

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            target_types: Vec::new(),
            symbols: Vec::new(),
            generic_sources: Vec::new(),
            lir_data: Vec::new(),
            deps: Vec::new(),
            effect_summaries: std::collections::HashMap::new(),
        }
    }

    /// Collect symbols from top-level AST statements.
    /// If `all` is true, include all functions (ignore visibility).
    /// A3c：注入 HIR 推断摘要（函数全名 → 摘要）。
    pub fn set_effect_summaries(
        &mut self,
        map: std::collections::HashMap<String, crate::hir::effects::EffectSummary>,
    ) {
        self.effect_summaries = map;
    }

    /// M6.1b：登记可导出常量（编码为 `PackageSymbol::Const`）
    pub fn set_consts(&mut self, consts: &[(crate::intern::Symbol, crate::hir::ir::HirType, crate::hir::ir::HirLiteral)]) {
        for (name, ty, lit) in consts {
            if let Some((ty_str, value)) = crate::package::const_codec::encode(ty, lit) {
                self.symbols.push(PackageSymbol::Const {
                    name: name.as_str(),
                    ty: ty_str,
                    value,
                });
            }
        }
    }

    /// M6.2b：登记可导出全局变量（`all=false` 时仅 pub；标量类型）
    pub fn set_statics(&mut self, statics: &[crate::hir::HirStatic], all: bool) {
        for s in statics {
            if !all && !s.is_pub { continue; }
            if let Some(ty_str) = crate::package::const_codec::type_str(&s.ty) {
                self.symbols.push(PackageSymbol::Static {
                    name: s.name.as_str(),
                    ty: ty_str,
                    is_mut: s.is_mut,
                });
            }
        }
    }

    pub fn collect_symbols(&mut self, stmts: &[Stmt]) {
        self.collect_symbols_with_prefix(stmts, false, "")
    }

    /// Collect ALL symbols (including private), used for .aya→.lcl compilation.
    pub fn collect_all_symbols(&mut self, stmts: &[Stmt]) {
        self.collect_symbols_with_prefix(stmts, true, "")
    }

    fn collect_symbols_with_prefix(&mut self, stmts: &[Stmt], all: bool, ns_prefix: &str) {
        for stmt in stmts {
            if !crate::hir::cfg::stmt_enabled(stmt) { continue; }
            self.collect_stmt_symbols(stmt, all, ns_prefix);
        }
    }

    fn collect_stmt_symbols(&mut self, stmt: &Stmt, all: bool, ns_prefix: &str) {
        match stmt {
            Stmt::FnDecl { vis, name, params, return_type, generic_params, attrs, .. } => {
                if !generic_params.is_empty() {
                    // #145：顶层泛型自由函数序列化进 generic_sources（导入侧注册 + 调用点单态化）
                    if all || vis.is_public() {
                        let prog = crate::parser::ast::Program { stmts: vec![stmt.clone()] };
                        let src = crate::formatter::format_program(&prog);
                        self.generic_sources.push(src);
                    }
                    return;
                }
                // A5b：宏单独入宏表，不作为普通函数导出
                if attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "macro") {
                    if all || vis.is_public() {
                        let full_name = if ns_prefix.is_empty() {
                            name.as_str().to_string()
                        } else {
                            format!("{}.{}", ns_prefix, name)
                        };
                        self.symbols.push(PackageSymbol::Macro { name: full_name });
                    }
                    return;
                }
                // A5d：优化注解单独入 pass 表，不作为普通函数导出
                if attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "pass") {
                    if all || vis.is_public() {
                        let full_name = if ns_prefix.is_empty() {
                            name.as_str().to_string()
                        } else {
                            format!("{}.{}", ns_prefix, name)
                        };
                        self.symbols.push(PackageSymbol::Pass { name: full_name });
                    }
                    return;
                }
                // A5d：只读检查注解入 check 表
                if attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "check") {
                    if all || vis.is_public() {
                        let full_name = if ns_prefix.is_empty() {
                            name.as_str().to_string()
                        } else {
                            format!("{}.{}", ns_prefix, name)
                        };
                        self.symbols.push(PackageSymbol::Check { name: full_name });
                    }
                    return;
                }
                if all || vis.is_public() {
                    let full_name = if ns_prefix.is_empty() {
                        name.as_str().to_string()
                    } else {
                        format!("{}.{}", ns_prefix, name)
                    };
                    let sig = format!("{}({})->{}",
                        full_name,
                        params.iter().map(|(_, t)| type_to_string(t)).collect::<Vec<_>>().join(","),
                        type_to_string(return_type));
                    let declared = crate::hir::effects::parse(attrs).unwrap_or_default();
                    let inferred = self.effect_summaries.get(&full_name)
                        .map(|s| s.inferred.clone())
                        .unwrap_or_default();
                    let summary = crate::hir::effects::EffectSummary { declared, inferred };
                    let mut flags = summary.tokens();
                    // M1.9：`#[noreturn]` 随包导出（导入侧调用类型为 `!`）
                    if crate::hir::attrs::has(attrs, "noreturn") {
                        flags.push("noreturn".to_string());
                    }
                    // 函数级 track_caller：末尾保留参数 __line/__col/__file
                    let hidden = params.iter().rev()
                        .take_while(|(n, _)| matches!(n.as_str().as_str(), "__line" | "__col" | "__file"))
                        .count();
                    if hidden > 0 { flags.push("caller".to_string()); }
                    self.symbols.push(PackageSymbol::Fn {
                        name: full_name,
                        signature: sig,
                        flags,
                    });
                    if !generic_params.is_empty() {
                        // Serialize generic function AST to source code
                        let prog = crate::parser::ast::Program {
                            stmts: vec![stmt.clone()],
                        };
                        let src = crate::formatter::format_program(&prog);
                        self.generic_sources.push(src);
                    }
                }
            }
            Stmt::EnumDef { .. } => {}
            Stmt::ImplBlock { type_name, methods, generic_params, .. } => {
                // 方法表：类型名 + 方法签名（编辑器补全/悬停用；编译器导入忽略）
                let type_base = type_name.as_str().to_string();
                for m in methods {
                    if !crate::hir::cfg::stmt_enabled(m) { continue; }
                    if let Stmt::FnDecl { name, params, return_type, vis, .. } = m {
                        if all || vis.is_public() {
                            // hover 展示用：去掉末尾保留参数（__line/__col/__file）
                            let visible = params.len()
                                - params.iter().rev()
                                    .take_while(|(n, _)| matches!(n.as_str().as_str(), "__line" | "__col" | "__file"))
                                    .count();
                            let sig = format!("{}({})->{}",
                                name,
                                params[..visible].iter().map(|(_, t)| type_to_string(t)).collect::<Vec<_>>().join(","),
                                type_to_string(return_type))
                                .replace("Self", &type_base)
                                .replace("???", "fn(...)");
                            self.symbols.push(PackageSymbol::Method {
                                type_name: type_base.clone(),
                                name: name.as_str().to_string(),
                                signature: sig,
                            });
                        }
                    }
                }
                let has_generic = !generic_params.is_empty()
                    || methods.iter().any(|m| matches!(m, Stmt::FnDecl { generic_params, .. } if !generic_params.is_empty()));
                if has_generic {
                    // Serialize the entire impl block as generic source (needed for self syntax)
                    let prog = crate::parser::ast::Program {
                        stmts: vec![stmt.clone()],
                    };
                    let src = crate::formatter::format_program(&prog);
                    self.generic_sources.push(src);
                }
                // 如果 impl 级有泛型参数，所有方法都引用这些参数，不能作为独立符号导出
                if !generic_params.is_empty() {
                    // All methods reference the impl's generic params; skip individual symbols
                } else {
                    for m in methods {
                        if !crate::hir::cfg::stmt_enabled(m) { continue; }
                        if let Stmt::FnDecl { generic_params, .. } = m {
                            if !generic_params.is_empty() {
                                continue; // 方法级泛型已包含在 generic_sources 中
                            }
                        }
                        self.collect_stmt_symbols(m, all, ns_prefix);
                    }
                }
            }
            Stmt::StructDef { vis, name, fields, .. } => {
                if vis.is_public() {
                    let fields_str: Vec<String> = fields.iter()
                        .map(|(fn_name, fty)| format!("{}:{}", fn_name, type_to_string(fty)))
                        .collect();
                    self.symbols.push(PackageSymbol::Struct {
                        name: format!("{}({})", name, fields_str.join(",")),
                    });
                }
            }
            Stmt::InterfaceDef { name, methods: _, generic_params: _, .. } => {
                self.symbols.push(PackageSymbol::Interface {
                    name: name.as_str().to_string(),
                });
                // Serialize interface to generic sources for method resolution
                let prog = crate::parser::ast::Program {
                    stmts: vec![stmt.clone()],
                };
                let src = crate::formatter::format_program(&prog);
                self.generic_sources.push(src);
            }
            Stmt::Namespace { vis, name, items, .. } => {
                if all || vis.is_public() {
                    self.symbols.push(PackageSymbol::Namespace {
                        name: name.as_str().to_string(),
                    });
                }
                // 命名空间内含泛型函数时，整体序列化到 generic_sources，
                // 供导入侧按 `Name.fn` 前缀注册
                let has_generic = items.iter().any(|it| match it {
                    Stmt::FnDecl { generic_params, .. } => !generic_params.is_empty(),
                    Stmt::ImplBlock { generic_params, methods, .. } => {
                        !generic_params.is_empty()
                            || methods.iter().any(|m| matches!(m, Stmt::FnDecl { generic_params, .. } if !generic_params.is_empty()))
                    }
                    _ => false,
                });
                if has_generic && (all || vis.is_public()) {
                    let prog = crate::parser::ast::Program { stmts: vec![stmt.clone()] };
                    let src = crate::formatter::format_program(&prog);
                    self.generic_sources.push(src);
                }
                let nested = if ns_prefix.is_empty() {
                    name.as_str().to_string()
                } else {
                    format!("{}.{}", ns_prefix, name)
                };
                for item in items {
                    if !crate::hir::cfg::stmt_enabled(item) { continue; }
                    self.collect_stmt_symbols(item, all, &nested);
                }
            }
            _ => {}
        }
    }
}
