use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

/// A0 属性白名单（编译器内置）。A1 起逐个接入 LLVM 语义；未知属性一律报错（ADR-2）。
/// A5a 起内置名保留裸名，`core::name` 为等价别名；库宏走 `pkg::macro`（provider 解析）。
pub const ALLOWED: &[&str] = &[
    "assume",
    "requires",
    "cfg",
    "inline",
    "cold",
    "noreturn",
    "pure",
    "readonly",
    "nounwind",
    "willreturn",
    "noalias",
    "nonnull",
];

/// 形参位置允许的属性（A1b）。
pub const PARAM_ALLOWED: &[&str] = &["noalias", "nonnull"];

/// A5a：程序内 import 汇总，用于标注 provider 解析。
#[derive(Default)]
pub struct Imports {
    /// 所有 import 的包名（路径 stem，如 `std/math.aya` → `math`）
    pub packages: HashSet<String>,
    /// `import "pkg" { a, b }` 的短名列表：pkg → [a, b]
    pub macros: HashMap<String, Vec<String>>,
}

impl Imports {
    pub fn collect(program: &Program) -> Self {
        let mut s = Self::default();
        for stmt in &program.stmts {
            s.collect_stmt(stmt);
        }
        s
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Import { path, macros, .. } => {
                let stem = pkg_stem(path);
                if !macros.is_empty() {
                    self.macros
                        .entry(stem.clone())
                        .or_default()
                        .extend(macros.iter().map(|m| m.as_str()));
                }
                self.packages.insert(stem);
            }
            Stmt::Namespace { items, .. } => {
                for s in items {
                    self.collect_stmt(s);
                }
            }
            _ => {}
        }
    }
}

/// 导入路径 → 包名（`std/math.aya` → `math`）。
pub fn pkg_stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// 属性是否出现在列表中（仅内置名，忽略 `core::` 前缀）。
pub fn has(attrs: &[Attr], name: &str) -> bool {
    attrs.iter().any(|a| a.is_builtin() && a.name.as_str() == name)
}

/// 校验属性列表（A5a provider 解析）。
pub fn validate(attrs: &[Attr], imports: &Imports) -> Result<()> {
    for a in attrs {
        resolve(a, imports)?;
    }
    Ok(())
}

/// 解析单个标注：内置白名单 / `core::` 别名 / 库宏（A5b 落地执行）。
fn resolve(a: &Attr, imports: &Imports) -> Result<()> {
    let name = a.name.as_str();
    if a.is_builtin() {
        if ALLOWED.contains(&name.as_str()) {
            return Ok(());
        }
        // `import "pkg" { name }` 短名 → 库宏
        if a.qualifier.is_empty() {
            if let Some(pkg) = imports
                .macros
                .iter()
                .find_map(|(p, ms)| ms.iter().any(|m| m == &name).then(|| p.clone()))
            {
                return Err(pending_macro(&format!("{}::{}", pkg, name), a));
            }
        }
        return Err(Error::Hir(format!(
            "unknown attribute #[{}] (at {}:{})",
            a.path_str(),
            a.span.start_line,
            a.span.start_col
        )));
    }
    // 库宏：必须全限定 `pkg::macro`，且 pkg 已 import
    let pkg = a.qualifier.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("::");
    let stem = a.qualifier.last().unwrap().as_str();
    if !imports.packages.contains(&stem) && !imports.packages.contains(&pkg) {
        return Err(Error::Hir(format!(
            "unknown attribute #[{}]: package `{}` is not imported (at {}:{})",
            a.path_str(),
            pkg,
            a.span.start_line,
            a.span.start_col
        )));
    }
    Err(pending_macro(&a.path_str(), a))
}

/// A5a：库宏已能解析到包，但宏插件执行在 A5b。
fn pending_macro(path: &str, a: &Attr) -> Error {
    Error::Hir(format!(
        "library macro #[{}] is recognized but macro expansion lands in A5b (at {}:{})",
        path, a.span.start_line, a.span.start_col
    ))
}

/// 递归校验整个程序的声明属性（含命名空间/impl/接口方法）。
pub fn validate_program(program: &Program) -> Result<()> {
    let imports = Imports::collect(program);
    for stmt in &program.stmts {
        validate_stmt(stmt, &imports)?;
    }
    Ok(())
}

/// 校验形参标注：仅内置 noalias/nonnull，且作用于指针类型（ref/unique/[T]/fn）。
fn validate_param(attrs: &[Attr], ty: &crate::parser::ast::Type) -> Result<()> {
    use crate::parser::ast::Type;
    for a in attrs {
        let name = a.name.as_str();
        if !a.is_builtin() || !PARAM_ALLOWED.contains(&name.as_str()) {
            return Err(Error::Hir(format!(
                "attribute #[{}] is not allowed on parameters (at {}:{})",
                a.path_str(),
                a.span.start_line,
                a.span.start_col
            )));
        }
        let is_ptr = matches!(ty, Type::Ref(..) | Type::Unique(..) | Type::Array(..) | Type::FnPtr(..));
        if !is_ptr {
            return Err(Error::Hir(format!(
                "#[{}] requires a pointer parameter (at {}:{})",
                name, a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

fn validate_stmt(stmt: &Stmt, imports: &Imports) -> Result<()> {
    match stmt {
        Stmt::FnDecl { attrs, params, param_attrs, .. } => {
            validate(attrs, imports)?;
            crate::hir::contracts::validate_fn_attrs(attrs)?;
            for (i, pa) in param_attrs.iter().enumerate() {
                if let Some((_, ty)) = params.get(i) {
                    validate_param(pa, ty)?;
                }
            }
        }
        Stmt::StructDef { attrs, .. }
        | Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => validate(attrs, imports)?,
        _ => {}
    }
    match stmt {
        Stmt::Namespace { items, .. } => {
            for s in items {
                validate_stmt(s, imports)?;
            }
        }
        Stmt::ImplBlock { methods, .. } => {
            for s in methods {
                validate_stmt(s, imports)?;
            }
        }
        Stmt::InterfaceDef { methods, .. } => {
            for m in methods {
                validate(&m.attrs, imports)?;
            }
        }
        _ => {}
    }
    Ok(())
}
