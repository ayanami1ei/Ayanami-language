//! A3：效应注册表与注解解析（具体效应；可扩展，不设白名单）。
//!
//! - 内置效应：`#[io]` / `#[state]` / `#[alloc]`（注册表初始项）。
//! - 承诺：`#[pure]`（无任何效应）、`#[no_error]`（永不失败）。
//! - 错误槽位：`#[throws()]`（开放空集）/ `#[throws]`|`#[throws(_)]`（类型未知）/ `#[throws(E1,E2)]`。
//! - 默认最好情况：不写注解 = 空集起步，由推断补全（见 infer）。

use std::sync::{LazyLock, Mutex};

use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, AttrArg, Expr};

/// 内置效应（注册表初始项）。
pub const BUILTIN_EFFECTS: &[&str] = &["io", "state", "alloc"];

/// runtime / libc 中已知的 IO 函数（推断用）。
pub(crate) const IO_NAMES: &[&str] = &[
    "putchar", "printf", "getchar",
    "__ayanami_getchar", "__ayanami_putchar",
    "__ayanami_print_int", "__ayanami_print_str", "__ayanami_print_ln",
];

pub mod diag;
pub mod infer;
pub mod scan;

pub use infer::{analyze, set_verify_effects, verify_effects, EffectSet};

/// A5 插件扩展点：注册新效应名（编译器内注册；源码/插件接入在后续阶段）。
static EXTRA_EFFECTS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn register_effect(name: &str) {
    let mut v = EXTRA_EFFECTS.lock().unwrap();
    if !v.iter().any(|e| e == name) {
        v.push(name.to_string());
    }
}

/// 该名字是否为已注册效应。
pub fn is_effect(name: &str) -> bool {
    BUILTIN_EFFECTS.contains(&name) || EXTRA_EFFECTS.lock().unwrap().iter().any(|e| e == name)
}

/// 错误槽位声明。
#[derive(Debug, Clone, PartialEq)]
pub enum ThrowsDecl {
    /// `#[throws()]`：今天无具体错误 + 开放槽位（未来可能失败，符号不变）
    OpenEmpty,
    /// `#[throws]` / `#[throws(_)]`：可能失败，错误类型未知
    Unknown,
    /// `#[throws(E1, E2)]`：具体错误集合
    Types(Vec<Symbol>),
}

/// 函数声明的效应（注解权威；声明与推断分开存储/导出）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectDecl {
    /// 声明的具体效应（io/state/alloc/自定义）
    pub effects: Vec<Symbol>,
    /// 承诺无任何效应
    pub pure: bool,
    /// 承诺永不失败
    pub no_error: bool,
    /// 错误槽位（None = 未声明）
    pub throws: Option<ThrowsDecl>,
}

impl EffectDecl {
    pub fn has_effect(&self, name: &str) -> bool {
        self.effects.iter().any(|e| e.as_str() == name)
    }

    /// 显式承诺无效应（优化可跨边界信任）。
    pub fn no_effects(&self) -> bool {
        self.pure
    }

    /// 显式承诺无失败（优化可跨边界信任）。
    pub fn no_throws(&self) -> bool {
        self.no_error
    }
}

/// 打包/导入用的效应摘要（声明 + 本轮推断事实）。
#[derive(Debug, Clone, Default)]
pub struct EffectSummary {
    pub declared: EffectDecl,
    pub inferred: crate::hir::effects::infer::EffectSet,
}

impl EffectSummary {
    /// 编码为 `.lcl` flags tokens（无 `+`/`,`；名称均为标识符）。
    pub fn tokens(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut push = |t: String| {
            if !out.contains(&t) { out.push(t); }
        };
        for e in &self.declared.effects {
            push(format!("d:{}", e.as_str()));
        }
        if self.declared.pure { push("pure".into()); }
        if self.declared.no_error { push("no_error".into()); }
        match &self.declared.throws {
            Some(ThrowsDecl::OpenEmpty) => push("throws:open".into()),
            Some(ThrowsDecl::Unknown) => push("throws:any".into()),
            Some(ThrowsDecl::Types(ts)) => {
                let list: Vec<String> = ts.iter().map(|t| t.as_str()).collect();
                push(format!("throws:{}", list.join("|")));
            }
            None => {}
        }
        for e in &self.inferred.effects {
            push(format!("i:{}", e));
        }
        // 无体目标（extern/未承诺 pure）导致的保守标记必须随包导出，
        // 否则导入方会把「未知」误判为「无副作用」（LTO 下会误优化）。
        if self.inferred.may_unknown_effects { push("i:unknown".into()); }
        if self.inferred.may_unknown_errors { push("i:unknown_errors".into()); }
        if !self.inferred.throws.is_empty() {
            push("i:throws".into());
        }
        out
    }

    /// 从 `.lcl` flags tokens 还原（兼容旧 `t`/`e` 单字符）。
    pub fn from_tokens(tokens: &[String]) -> Self {
        let mut declared = EffectDecl::default();
        let mut inferred = crate::hir::effects::infer::EffectSet::default();
        for t in tokens {
            if t == "t" { declared.no_error = true; continue; }
            if t == "e" { declared.pure = true; continue; }
            if let Some(rest) = t.strip_prefix("d:") {
                declared.effects.push(Symbol::intern(rest));
            } else if let Some(rest) = t.strip_prefix("i:") {
                if rest == "throws" {
                    inferred.throws.insert("?".into());
                } else if rest == "unknown" {
                    inferred.may_unknown_effects = true;
                } else if rest == "unknown_errors" {
                    inferred.may_unknown_errors = true;
                } else {
                    inferred.effects.insert(rest.to_string());
                }
            } else if t == "pure" {
                declared.pure = true;
            } else if t == "no_error" {
                declared.no_error = true;
            } else if let Some(rest) = t.strip_prefix("throws:") {
                declared.throws = Some(match rest {
                    "open" => ThrowsDecl::OpenEmpty,
                    "any" => ThrowsDecl::Unknown,
                    list => ThrowsDecl::Types(list.split('|').map(Symbol::intern).collect()),
                });
            }
        }
        Self { declared, inferred }
    }
}

/// 解析函数级效应注解。
pub fn parse(attrs: &[Attr]) -> Result<EffectDecl> {
    let mut decl = EffectDecl::default();
    for a in attrs.iter().filter(|a| a.is_builtin()) {
        let name = a.name.as_str();
        if is_effect(&name) {
            let sym = a.name;
            if !decl.effects.contains(&sym) {
                decl.effects.push(sym);
            }
            continue;
        }
        match name.as_str() {
            "pure" => decl.pure = true,
            "no_error" => decl.no_error = true,
            "throws" => merge_throws(&mut decl, parse_throws(a)?),
            _ => {}
        }
    }
    Ok(decl)
}

fn parse_throws(a: &Attr) -> Result<ThrowsDecl> {
    if a.args.is_empty() {
        return Ok(ThrowsDecl::OpenEmpty);
    }
    let mut names = Vec::new();
    for arg in &a.args {
        match arg {
            AttrArg::Expr(e) => match e.as_ref() {
                Expr::Ident(s, _) => names.push(*s),
                _ => return Err(bad_throws(a)),
            },
            AttrArg::KeyValue(..) => return Err(bad_throws(a)),
        }
    }
    if names.len() == 1 && names[0].as_str() == "_" {
        return Ok(ThrowsDecl::Unknown);
    }
    Ok(ThrowsDecl::Types(names))
}

/// 合并多个 throws 声明：信息量优先（Types > Unknown > OpenEmpty）。
fn merge_throws(decl: &mut EffectDecl, new: ThrowsDecl) {
    decl.throws = Some(match (decl.throws.take(), new) {
        (Some(ThrowsDecl::Types(mut a)), ThrowsDecl::Types(b)) => {
            for t in b {
                if !a.contains(&t) {
                    a.push(t);
                }
            }
            ThrowsDecl::Types(a)
        }
        (Some(ThrowsDecl::Types(a)), _) => ThrowsDecl::Types(a),
        (_, ThrowsDecl::Types(b)) => ThrowsDecl::Types(b),
        (Some(ThrowsDecl::Unknown), _) | (_, ThrowsDecl::Unknown) => ThrowsDecl::Unknown,
        _ => ThrowsDecl::OpenEmpty,
    });
}

fn bad_throws(a: &Attr) -> Error {
    Error::Hir(format!(
        "#[throws] expects error type names, `_`, or empty (at {}:{})",
        a.span.start_line, a.span.start_col
    ))
}
