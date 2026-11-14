//! A5d：导入包的注解表（宏 / 优化 pass / 只读检查）与裸名解析。
//!
//! - 注解表来自各 import 的 `.lcl`：`macro=` / `pass=` / `check=`；
//! - 裸名自动作用域：`import "pkg"` 后该包导出的注解全部进入作用域；
//!   重名时报错并提示全限定 `pkg::name`；
//! - 全限定始终可用（要求包已 import）。
//!
//! 解析规则（宏展开与 HIR 校验共用，历史上两套实现曾漂移）：
//! - 同一包内注解名跨类别唯一（打包/加载时校验，见 `package::symbols` / `package::load`）；
//! - 跨包同名允许，裸名解析到多个包 → 歧义错误，必须全限定。

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, Stmt};

/// 注解类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnnKind {
    Macro,
    Pass,
    Check,
}

/// 一个包导出的注解
#[derive(Default, Clone)]
pub(crate) struct PkgAnnotations {
    pub lcl_path: String,
    /// (宏名, 是否 token 宏；mcl.1)
    pub macros: Vec<(String, bool)>,
    pub passes: Vec<String>,
    pub checks: Vec<String>,
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

    /// HIR 侧：从已收集的导入映射构造（只关心类别，token 宏标记在此不需要）。
    pub(crate) fn from_kind_maps(
        macros: &HashMap<Symbol, Vec<Symbol>>,
        passes: &HashMap<Symbol, Vec<Symbol>>,
        checks: &HashMap<Symbol, Vec<Symbol>>,
    ) -> Self {
        let mut pkgs: HashMap<String, PkgAnnotations> = HashMap::new();
        for (pkg, names) in macros {
            let p = pkgs.entry(pkg.as_str().to_string()).or_default();
            for n in names {
                p.macros.push((n.as_str().to_string(), false));
            }
        }
        for (pkg, names) in passes {
            let p = pkgs.entry(pkg.as_str().to_string()).or_default();
            for n in names {
                p.passes.push(n.as_str().to_string());
            }
        }
        for (pkg, names) in checks {
            let p = pkgs.entry(pkg.as_str().to_string()).or_default();
            for n in names {
                p.checks.push(n.as_str().to_string());
            }
        }
        Self { pkgs }
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
                            crate::package::ImportedSymbol::Macro { name, tokens } => pkg.macros.push((name, tokens)),
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
    pub fn resolve(&self, a: &Attr) -> Result<Option<(String, String, AnnKind, bool)>> {
        let name = a.name.as_str();
        if !a.qualifier.is_empty() {
            let pkg = a.qualifier[0].as_str();
            let rest: Vec<String> = a.qualifier[1..].iter().map(|s| s.as_str()).collect();
            let ann = if rest.is_empty() { name.clone() } else { format!("{}.{}", rest.join("."), name) };
            let Some(p) = self.pkgs.get(&pkg) else { return Ok(None) };
            if let Some((_, tokens)) = p.macros.iter().find(|(m, _)| *m == ann) {
                return Ok(Some((pkg.to_string(), ann, AnnKind::Macro, *tokens)));
            }
            if p.passes.iter().any(|m| *m == ann) {
                return Ok(Some((pkg.to_string(), ann, AnnKind::Pass, false)));
            }
            if p.checks.iter().any(|m| *m == ann) {
                return Ok(Some((pkg.to_string(), ann, AnnKind::Check, false)));
            }
            return Ok(None);
        }
        // 每包最多一个命中（同包跨类别唯一由打包/加载校验保证）。
        let mut hits: Vec<(String, AnnKind, bool)> = Vec::new();
        for (pkg, p) in &self.pkgs {
            if let Some((_, tokens)) = p.macros.iter().find(|(m, _)| *m == name) {
                hits.push((pkg.clone(), AnnKind::Macro, *tokens));
            } else if p.passes.iter().any(|m| *m == name) {
                hits.push((pkg.clone(), AnnKind::Pass, false));
            } else if p.checks.iter().any(|m| *m == name) {
                hits.push((pkg.clone(), AnnKind::Check, false));
            }
        }
        match hits.len() {
            0 => Ok(None),
            1 => {
                let (pkg, kind, tokens) = hits.pop().unwrap();
                Ok(Some((pkg, name, kind, tokens)))
            }
            _ => {
                let list: Vec<String> = hits.iter().map(|(p, _, _)| format!("{}::{}", p, name)).collect();
                Err(Error::Compile(format!(
                    "ambiguous annotation #[{}]: exported by multiple packages ({}); use a qualified name (at {}:{})",
                    name, list.join(", "), a.span.start_line, a.span.start_col
                )))
            }
        }
    }

    /// 只要类别（HIR 校验用）。
    pub(crate) fn resolve_kind(&self, a: &Attr) -> Result<Option<AnnKind>> {
        Ok(self.resolve(a)?.map(|(_, _, k, _)| k))
    }

    pub fn lcl_path(&self, pkg: &str) -> Option<String> {
        self.pkgs.get(pkg).map(|p| p.lcl_path.clone())
    }
}
