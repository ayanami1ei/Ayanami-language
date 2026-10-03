//! A5d-1：导入包的注解表（宏 / 优化 pass）与裸名解析。
//!
//! - 注解表来自各 import 的 `.lcl`：`macro="name"` / `pass="name"`；
//! - 裸名自动作用域：`import "pkg"` 后该包导出的注解全部进入作用域；
//!   重名时报错并提示全限定 `pkg::name`；
//! - 全限定始终可用（要求包已 import）。

use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Stmt};

/// 一个包导出的注解
#[derive(Default, Clone)]
pub(super) struct PkgAnnotations {
    pub lcl_path: String,
    pub macros: Vec<String>,
    pub passes: Vec<String>,
    pub checks: Vec<String>,
}

/// 注解类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnnKind {
    Macro,
    Pass,
    Check,
}

/// 已导入包的注解表
#[derive(Default)]
pub(crate) struct AnnotationTables {
    pkgs: HashMap<String, PkgAnnotations>,
}

impl AnnotationTables {
    pub fn is_empty(&self) -> bool {
        self.pkgs.is_empty()
    }

    pub fn collect(stmts: &[Stmt]) -> Result<Self> {
        let mut t = Self::default();
        t.collect_stmts(stmts)?;
        Ok(t)
    }

    fn collect_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::Import { path, .. } => {
                    let stem = crate::hir::attrs::pkg_stem(path);
                    let (syms, _src, _lir, _tt) = crate::package::load_package(path)
                        .map_err(|e| Error::Compile(format!("annotation import '{}': {}", path, e)))?;
                    let mut pkg = PkgAnnotations { lcl_path: path.clone(), ..Default::default() };
                    for s in syms {
                        match s {
                            crate::package::ImportedSymbol::Macro { name } => pkg.macros.push(name),
                            crate::package::ImportedSymbol::Pass { name } => pkg.passes.push(name),
                            crate::package::ImportedSymbol::Check { name } => pkg.checks.push(name),
                            _ => {}
                        }
                    }
                    // 同名包重复 import：合并（保留首个 lcl 路径）
                    match self.pkgs.get_mut(&stem) {
                        Some(existing) => {
                            existing.macros.extend(pkg.macros);
                            existing.passes.extend(pkg.passes);
                            existing.checks.extend(pkg.checks);
                        }
                        None => { self.pkgs.insert(stem, pkg); }
                    }
                }
                Stmt::Namespace { items, .. } => self.collect_stmts(items)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// 解析标注：
    /// - 全限定 `pkg::name`：查该包表；未导入/未找到返回 None；
    /// - 裸名：跨所有已导入包查找；0 个 → None，多个 → 歧义错误。
    pub fn resolve(&self, a: &Attr) -> Result<Option<(String, String, AnnKind)>> {
        let name = a.name.as_str();
        if !a.qualifier.is_empty() {
            let pkg = a.qualifier[0].as_str();
            let rest: Vec<String> = a.qualifier[1..].iter().map(|s| s.as_str()).collect();
            let ann = if rest.is_empty() { name.clone() } else { format!("{}.{}", rest.join("."), name) };
            let Some(p) = self.pkgs.get(&pkg) else { return Ok(None) };
            if p.macros.iter().any(|m| *m == ann) {
                return Ok(Some((pkg, ann, AnnKind::Macro)));
            }
            if p.passes.iter().any(|m| *m == ann) {
                return Ok(Some((pkg, ann, AnnKind::Pass)));
            }
            if p.checks.iter().any(|m| *m == ann) {
                return Ok(Some((pkg, ann, AnnKind::Check)));
            }
            return Ok(None);
        }
        let mut hits: Vec<(String, AnnKind)> = Vec::new();
        for (pkg, p) in &self.pkgs {
            if p.macros.iter().any(|m| *m == name) {
                hits.push((pkg.clone(), AnnKind::Macro));
            } else if p.passes.iter().any(|m| *m == name) {
                hits.push((pkg.clone(), AnnKind::Pass));
            } else if p.checks.iter().any(|m| *m == name) {
                hits.push((pkg.clone(), AnnKind::Check));
            }
        }
        match hits.len() {
            0 => Ok(None),
            1 => {
                let (pkg, kind) = hits.pop().unwrap();
                Ok(Some((pkg, name, kind)))
            }
            _ => {
                let list: Vec<String> = hits.iter().map(|(p, _)| format!("{}::{}", p, name)).collect();
                Err(Error::Compile(format!(
                    "ambiguous annotation #[{}]: exported by multiple packages ({}); use a qualified name (at {}:{})",
                    name, list.join(", "), a.span.start_line, a.span.start_col
                )))
            }
        }
    }

    pub fn lcl_path(&self, pkg: &str) -> Option<String> {
        self.pkgs.get(pkg).map(|p| p.lcl_path.clone())
    }
}
