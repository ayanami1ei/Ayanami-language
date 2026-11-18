//! M6：为跨包泛型源码保留来源包的自由函数作用域。
//!
//! `.lcl` 中的泛型函数以源码形式在消费方重新实例化。若其函数体调用来源包
//! 的非泛型私有 helper，裸名会落到消费方的同名函数，或链接到不可见的 internal
//! 符号。这里把实际依赖闭包收进包私有命名空间；导入侧注入该命名空间，泛型
//! 特化时优先解析来源作用域。

use std::collections::{HashMap, HashSet};

use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Program, Stmt};

use self::walk::calls_in_block;

mod walk;

pub(crate) const SCOPE_PREFIX: &str = "__ayanami_scope_";

pub(crate) fn scope_name(package: &str) -> String {
    let safe: String = package
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    format!("{}{}", SCOPE_PREFIX, safe)
}

impl super::Package {
    pub(crate) fn add_generic_scope_source(&mut self, stmts: &[Stmt]) {
        let mut roots = HashSet::new();
        if collect_generic_calls(stmts, &mut roots) == 0 {
            return;
        }

        let (free_fns, by_name) = collect_free_fns(stmts);
        let mut pending: Vec<Symbol> = roots.into_iter().collect();
        let mut selected = HashSet::new();
        while let Some(name) = pending.pop() {
            let Some(indices) = by_name.get(&name) else { continue };
            for &idx in indices {
                if !selected.insert(idx) {
                    continue;
                }
                if let Stmt::FnDecl { body, .. } = &free_fns[idx] {
                    let mut calls = HashSet::new();
                    calls_in_block(body, &mut calls);
                    pending.extend(calls);
                }
            }
        }

        let mut items: Vec<Stmt> = selected.into_iter()
            .map(|idx| make_linkonce(&free_fns[idx]))
            .collect();
        items.sort_by_key(|s| s.span().start_byte);
        if items.is_empty() {
            return;
        }

        let scope = scope_name(&self.name);
        if std::env::var_os("AYANAMI_MODULE_TRACE").is_some() {
            eprintln!(
                "module trace: package {} embeds {} generic dependency function(s) in {}",
                self.name,
                items.len(),
                scope
            );
        }
        let source = crate::formatter::format_program(&Program::new(vec![Stmt::Namespace {
            vis: Visibility::Private,
            name: Symbol::intern(&scope),
            items,
            span: crate::span::Span::default(),
        }]));
        if !self.generic_sources.iter().any(|s| s == &source) {
            self.generic_sources.push(source);
        }
    }
}

fn collect_free_fns(stmts: &[Stmt]) -> (Vec<Stmt>, HashMap<Symbol, Vec<usize>>) {
    let mut fns = Vec::new();
    let mut by_name: HashMap<Symbol, Vec<usize>> = HashMap::new();
    for stmt in stmts {
        if let Stmt::FnDecl { name, generic_params, .. } = stmt {
            if generic_params.is_empty() {
                let idx = fns.len();
                fns.push(stmt.clone());
                by_name.entry(*name).or_default().push(idx);
            }
        }
    }
    (fns, by_name)
}

fn collect_generic_calls(stmts: &[Stmt], out: &mut HashSet<Symbol>) -> usize {
    let mut count = 0;
    for stmt in stmts {
        match stmt {
            Stmt::FnDecl { generic_params, body, .. } if !generic_params.is_empty() => {
                count += 1;
                calls_in_block(body, out);
            }
            Stmt::Namespace { items, .. } => count += collect_generic_calls(items, out),
            Stmt::ImplBlock { generic_params, methods, .. } => {
                for method in methods {
                    if let Stmt::FnDecl { generic_params: method_gp, body, .. } = method {
                        if !generic_params.is_empty() || !method_gp.is_empty() {
                            count += 1;
                            calls_in_block(body, out);
                        }
                    }
                }
            }
            Stmt::Attributed { stmt, .. } => {
                count += collect_generic_calls(std::slice::from_ref(stmt), out)
            }
            _ => {}
        }
    }
    count
}

fn make_linkonce(stmt: &Stmt) -> Stmt {
    let mut stmt = stmt.clone();
    if let Stmt::FnDecl { attrs, is_inline, .. } = &mut stmt {
        *is_inline = true;
        if !attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == "inline") {
            attrs.push(crate::parser::ast::Attr {
                qualifier: Vec::new(),
                name: Symbol::intern("inline"),
                args: Vec::new(),
                span: crate::span::Span::default(),
            });
        }
    }
    stmt
}
