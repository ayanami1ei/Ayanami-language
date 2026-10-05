use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, Program, Stmt};

/// A0 属性白名单（编译器内置）。A1 起逐个接入 LLVM 语义；未知属性一律报错（ADR-2）。
/// A5a 起内置名保留裸名，`core::name` 为等价别名；库宏走 `pkg::macro`（provider 解析）。
pub const ALLOWED: &[&str] = &[
    "assume",
    "requires",
    "ensures",
    "invariant",
    "macro",
    "pass",
    "check",
    "follow_with",
    "throws",
    "no_error",
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
    // #84 ③：导出 C 符号（等价 `extern "C"` 定义）
    "export",
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

pub use super::attrs_macro::validate_macros;

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
        if ALLOWED.contains(&name.as_str()) || crate::hir::effects::is_effect(&name) {
            return Ok(());
        }
        // 裸名库注解（宏/pass）：import 自动作用域，存在性在导入加载后校验
        // （validate_annotations），这里无法提前判定，延后处理。
        if a.qualifier.is_empty() {
            return Ok(());
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
    // 存在性与展开检查推迟到 validate_macros（imports 已加载宏表）
    Ok(())
}

/// A4a：`#[follow_with(src, ...)]` 校验——至少一个来源，且为标识符。
fn validate_follow_with(a: &Attr) -> Result<()> {
    if a.args.is_empty() {
        return Err(Error::Hir(format!(
            "#[follow_with] requires at least one source name (at {}:{})",
            a.span.start_line, a.span.start_col
        )));
    }
    for arg in &a.args {
        let ok = matches!(arg, crate::parser::ast::AttrArg::Expr(e)
            if matches!(e.as_ref(), crate::parser::ast::Expr::Ident(..)));
        if !ok {
            return Err(Error::Hir(format!(
                "#[follow_with] expects parameter/type names (at {}:{})",
                a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

fn validate_follow_with_attrs(attrs: &[Attr]) -> Result<()> {
    for a in attrs.iter().filter(|a| a.is_builtin() && a.name.as_str() == "follow_with") {
        validate_follow_with(a)?;
    }
    Ok(())
}

fn reject_follow_with(attrs: &[Attr], place: &str) -> Result<()> {
    for a in attrs.iter().filter(|a| a.is_builtin() && a.name.as_str() == "follow_with") {
        return Err(Error::Hir(format!(
            "#[follow_with] is only allowed on functions and struct fields (not on {}) (at {}:{})",
            place, a.span.start_line, a.span.start_col
        )));
    }
    Ok(())
}

/// 校验形参标注：仅内置 noalias/nonnull，且作用于指针类型（ref/unique/[T]/fn）。
/// 递归校验整个程序的声明属性（含命名空间/impl/接口方法）。
pub fn validate_program(program: &Program) -> Result<()> {
    let imports = Imports::collect(program);
    for stmt in &program.stmts {
        validate_stmt(stmt, &imports)?;
    }
    Ok(())
}

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
        Stmt::FnDecl { attrs, params, param_attrs, generic_params, .. } => {
            validate(attrs, imports)?;
            crate::hir::contracts::validate_fn_attrs(attrs)?;
            validate_follow_with_attrs(attrs)?;
            crate::hir::attrs_export::validate_fn_export(attrs, !generic_params.is_empty())?;
            for (i, pa) in param_attrs.iter().enumerate() {
                if let Some((_, ty)) = params.get(i) {
                    validate_param(pa, ty)?;
                }
            }
        }
        Stmt::StructDef { attrs, field_attrs, .. } => {
            validate(attrs, imports)?;
            reject_follow_with(attrs, "struct declarations")?;
            crate::hir::attrs_export::reject_export(attrs, "struct declarations")?;
            for fa in field_attrs {
                for a in fa {
                    if !(a.is_builtin() && a.name.as_str() == "follow_with") {
                        return Err(Error::Hir(format!(
                            "attribute #[{}] is not allowed on struct fields (at {}:{})",
                            a.path_str(), a.span.start_line, a.span.start_col
                        )));
                    }
                    validate_follow_with(a)?;
                }
            }
        }
        Stmt::EnumDef { attrs, .. }
        | Stmt::InterfaceDef { attrs, .. }
        | Stmt::ImplBlock { attrs, .. } => {
            validate(attrs, imports)?;
            reject_follow_with(attrs, "this declaration")?;
            crate::hir::attrs_export::reject_export(attrs, "this declaration")?;
        }
        Stmt::Attributed { attrs, stmt, .. } => {
            validate(attrs, imports)?;
            for a in attrs {
                if !a.is_builtin() {
                    return Err(Error::Hir(format!(
                        "library macro #[{}] is not allowed on statements (at {}:{})",
                        a.path_str(), a.span.start_line, a.span.start_col
                    )));
                }
                match a.name.as_str().as_str() {
                    "cfg" => {}
                    "invariant" => {
                        crate::hir::contracts::validate_invariant_attrs(std::slice::from_ref(a))?;
                        if !matches!(stmt.as_ref(), Stmt::While { .. } | Stmt::For { .. }) {
                            return Err(Error::Hir(format!(
                                "#[invariant] is only allowed on while/for loops (at {}:{})",
                                a.span.start_line, a.span.start_col
                            )));
                        }
                    }
                    other => return Err(Error::Hir(format!(
                        "attribute #[{}] is not allowed on statements (at {}:{})",
                        other, a.span.start_line, a.span.start_col
                    ))),
                }
            }
            validate_stmt(stmt, imports)?;
        }
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
