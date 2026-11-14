//! A5g：统一注解注册表。
//!
//! 内建标注与外部包导出的 macro/pass/check/attr 插件都走 provider 解析；
//! 这里集中登记编译器保留标注的类别与声明标记，替代散落的 `ALLOWED` 白名单。

use crate::parser::ast::Attr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Config,
    Contract,
    Effect,
    Ffi,
    Codegen,
    Optimization,
    Safety,
    Declaration,
    Transform,
}

#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    /// 供工具/文档消费的元数据；编译器核心路径暂不读取。
    #[allow(dead_code)]
    pub category: Category,
    /// `#[macro]` / `#[pass]` / `#[check]` / `#[attr]` 等声明标记
    pub declaration: bool,
}

macro_rules! b {
    ($name:literal, $cat:ident) => {
        Builtin { name: $name, category: Category::$cat, declaration: false }
    };
    ($name:literal, $cat:ident, decl) => {
        Builtin { name: $name, category: Category::$cat, declaration: true }
    };
}

/// 编译器保留标注注册表（effects 由 `EffectRegistry` 动态提供，另行识别）。
pub const BUILTINS: &[Builtin] = &[
    b!("async", Transform),
    b!("sync", Transform),
    b!("cfg", Config),
    b!("assume", Contract),
    b!("requires", Contract),
    b!("ensures", Contract),
    b!("invariant", Contract),
    b!("throws", Effect),
    b!("no_error", Effect),
    b!("inline", Optimization),
    b!("cold", Optimization),
    b!("noreturn", Optimization),
    b!("pure", Optimization),
    b!("readonly", Optimization),
    b!("nounwind", Optimization),
    b!("willreturn", Optimization),
    b!("noalias", Safety),
    b!("nonnull", Safety),
    b!("follow_with", Codegen),
    b!("export", Ffi),
    b!("extern", Ffi),
    b!("copy", Codegen),
    b!("compile_time", Codegen),
    b!("macro", Declaration, decl),
    b!("pass", Declaration, decl),
    b!("check", Declaration, decl),
    b!("attr", Declaration, decl),
    b!("transform", Declaration, decl),
];

pub fn builtin(name: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.name == name)
}

/// 内建名（含动态 effect 注册表）。
pub fn is_builtin_name(name: &str) -> bool {
    builtin(name).is_some() || crate::hir::effects::is_effect(name)
}

pub fn is_declaration_marker(name: &str) -> bool {
    builtin(name).is_some_and(|b| b.declaration)
}

/// 该标注是否由编译器（而非外部包）处理；`core::` 为内建别名，`env::` 为环境命名空间。
pub fn is_compiler_attr(a: &Attr) -> bool {
    if a.qualifier.is_empty() && a.name.as_str() == "async" {
        return true;
    }
    if a.qualifier.len() == 1 && a.qualifier[0].as_str() == "env" {
        return true;
    }
    let name = a.name.as_str();
    if crate::hir::effects::is_effect(&name) {
        return true;
    }
    a.is_builtin() && is_builtin_name(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_markers_are_registered() {
        for name in ["macro", "pass", "check", "attr"] {
            assert!(is_declaration_marker(name), "{name}");
        }
        assert!(!is_declaration_marker("pure"));
    }

    #[test]
    fn effects_are_builtin_providers() {
        assert!(is_builtin_name("alloc"));
        assert!(is_builtin_name("pure"));
        assert!(!is_builtin_name("not_a_real_annotation"));
    }

    #[test]
    fn compiler_attrs_accept_core_alias_and_env() {
        let core = crate::parser::parse_source("#[core::pure]\nfn f() { }\n")
            .expect("parse")
            .stmts;
        let crate::parser::ast::Stmt::FnDecl { attrs, .. } = &core[0] else {
            panic!("fn");
        };
        assert!(is_compiler_attr(&attrs[0]));

        let env = crate::parser::parse_source("#[env::alloc(x)]\nfn f() { }\n")
            .expect("parse")
            .stmts;
        let crate::parser::ast::Stmt::FnDecl { attrs, .. } = &env[0] else {
            panic!("fn");
        };
        assert!(is_compiler_attr(&attrs[0]));
    }
}
