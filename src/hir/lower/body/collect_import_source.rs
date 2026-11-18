use super::*;
use std::collections::HashSet;

impl crate::hir::lower::Ctx {
    pub(super) fn ingest_generic_sources(
        &mut self,
        path: &String,
        sources: &[String],
    ) -> Result<HashSet<Symbol>> {
        let mut programs = Vec::new();
        for source in sources {
            let tokens = crate::lexer::Lexer::new(source).tokenize_all();
            let filtered: Vec<_> = tokens
                .into_iter()
                .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
                .collect();
            if filtered.is_empty() {
                continue;
            }
            let mut parser = crate::parser::Parser::new(filtered);
            parser.allow_legacy_fn_types();
            match parser.parse_program() {
                Ok(program) => programs.push(program),
                Err(e) => eprintln!(
                    "warning: failed to re-parse package source for '{}': {}",
                    path, e
                ),
            }
        }

        let origin_scope = programs.iter()
            .flat_map(|p| &p.stmts)
            .find_map(|stmt| match stmt {
                Stmt::Namespace { name, .. }
                    if name.as_str().starts_with(crate::package::SCOPE_PREFIX) => Some(*name),
                _ => None,
            })
            .unwrap_or_else(|| {
                Symbol::intern(&crate::package::scope_name(
                    &crate::hir::attrs::pkg_stem(path),
                ))
            });

        let mut methods = HashSet::new();
        for program in &programs {
            for stmt in &program.stmts {
                self.ingest_source_stmt(stmt, origin_scope, "", &mut methods)?;
            }
        }
        Ok(methods)
    }

    fn ingest_source_stmt(
        &mut self,
        stmt: &Stmt,
        origin_scope: Symbol,
        ns_prefix: &str,
        methods: &mut HashSet<Symbol>,
    ) -> Result<()> {
        match stmt {
            Stmt::FnDecl { name, generic_params, span, .. } if !generic_params.is_empty() => {
                super::collect_ns::check_unique_generic_params(generic_params, span)?;
                let full = qualified_name(ns_prefix, *name);
                self.generic_fns.push((
                    full,
                    generic_params.clone(),
                    stmt.clone(),
                    false,
                    Some(origin_scope),
                ));
            }
            Stmt::ImplBlock { methods: items, generic_params: impl_gp, span, .. } => {
                super::collect_ns::check_unique_generic_params(impl_gp, span)?;
                for method in items {
                    if let Stmt::FnDecl {
                        name,
                        generic_params,
                        span: method_span,
                        ..
                    } = method
                    {
                        let mut combined = impl_gp.clone();
                        combined.extend(generic_params.iter().cloned());
                        super::collect_ns::check_unique_generic_params(&combined, method_span)?;
                        if combined.is_empty() {
                            methods.insert(qualified_name(ns_prefix, *name));
                        } else {
                            self.generic_fns.push((
                                qualified_name(ns_prefix, *name),
                                combined,
                                method.clone(),
                                true,
                                Some(origin_scope),
                            ));
                        }
                    }
                }
            }
            Stmt::Namespace { name, items, .. } => {
                let nested = if ns_prefix.is_empty() {
                    name.as_str().to_string()
                } else {
                    format!("{}.{}", ns_prefix, name)
                };
                for item in items {
                    self.ingest_source_stmt(item, origin_scope, &nested, methods)?;
                }
            }
            Stmt::InterfaceDef { name, methods: items, generic_params, span, .. } => {
                super::collect_ns::check_unique_generic_params(generic_params, span)?;
                let hir_methods = items.iter().map(|method| {
                    crate::hir::ir::HirInterfaceMethod {
                        name: method.name,
                        self_keyword: method.self_keyword,
                        params: method.params.iter()
                            .map(|(n, t)| (*n, ast_type_to_hir(t, &self.interfaces)))
                            .collect(),
                        return_type: ast_type_to_hir(&method.return_type, &self.interfaces),
                    }
                }).collect();
                self.interfaces.insert(*name, InterfaceReg {
                    generic_params: generic_params.clone(),
                    methods: hir_methods,
                });
            }
            _ => {}
        }
        Ok(())
    }
}

fn qualified_name(prefix: &str, name: Symbol) -> Symbol {
    if prefix.is_empty() {
        name
    } else {
        Symbol::intern(&format!("{}.{}", prefix, name))
    }
}
