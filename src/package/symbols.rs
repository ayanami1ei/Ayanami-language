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
        }
    }

    /// Collect symbols from top-level AST statements.
    /// If `all` is true, include all functions (ignore visibility).
    pub fn collect_symbols(&mut self, stmts: &[Stmt]) {
        self.collect_symbols_with_prefix(stmts, false, "")
    }

    /// Collect ALL symbols (including private), used for .aya→.lcl compilation.
    pub fn collect_all_symbols(&mut self, stmts: &[Stmt]) {
        self.collect_symbols_with_prefix(stmts, true, "")
    }

    fn collect_symbols_with_prefix(&mut self, stmts: &[Stmt], all: bool, ns_prefix: &str) {
        for stmt in stmts {
            self.collect_stmt_symbols(stmt, all, ns_prefix);
        }
    }

    fn collect_stmt_symbols(&mut self, stmt: &Stmt, all: bool, ns_prefix: &str) {
        match stmt {
            Stmt::FnDecl { vis, name, params, return_type, generic_params, .. } => {
                if !generic_params.is_empty() {
                    return; // Generic functions stored in generic_sources via ImplBlock or parent
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
                    self.symbols.push(PackageSymbol::Fn {
                        name: full_name,
                        signature: sig,
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
            Stmt::InterfaceDef { name, methods, generic_params, .. } => {
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
            Stmt::EnumDef { .. } => {}
            Stmt::ImplBlock { methods, .. } => {
                for m in methods {
                    self.collect_stmt_symbols(m, all, ns_prefix);
                }
            }
            Stmt::Namespace { vis, name, items, .. } => {
                if all || vis.is_public() {
                    self.symbols.push(PackageSymbol::Namespace {
                        name: name.as_str().to_string(),
                    });
                }
                let nested = if ns_prefix.is_empty() {
                    name.as_str().to_string()
                } else {
                    format!("{}.{}", ns_prefix, name)
                };
                for item in items {
                    self.collect_stmt_symbols(item, all, &nested);
                }
            }
            _ => {}
        }
    }
}
