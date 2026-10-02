//! A3a：效应注解解析与存储（注解权威；推断辅助在后续阶段）。
//!
//! - `#[throws(E1, E2)]`：异常效应；`#[throws()]`/`#[throws]` 显式空集。
//! - `#[eff(io, state, alloc)]`：其他效应；`#[eff()]` 显式空集。
//! - `#[pure]` 视为 `#[eff()]` 的别名。
//! - 允许多声明与少声明；不引入 `unknown` 效应（无注解即无保证）。

use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Attr, AttrArg, Expr};

/// 已知效应名（`#[eff(...)]` 集合）。
pub const KNOWN_EFFECTS: &[&str] = &["io", "state", "alloc"];

/// 函数声明的效应集合。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectDecl {
    /// None = 未声明；Some([]) = 显式空集（可作为优化保证）
    pub throws: Option<Vec<Symbol>>,
    /// None = 未声明；Some([]) = 显式空集
    pub effs: Option<Vec<Symbol>>,
}

impl EffectDecl {
    /// 显式声明了空 eff 集合（无 io/state/alloc）。
    pub fn no_effects(&self) -> bool {
        matches!(&self.effs, Some(v) if v.is_empty())
    }

    /// 显式声明了空 throws 集合。
    pub fn no_throws(&self) -> bool {
        matches!(&self.throws, Some(v) if v.is_empty())
    }
}

/// 解析函数级效应注解。
pub fn parse(attrs: &[Attr]) -> Result<EffectDecl> {
    let mut decl = EffectDecl::default();
    for a in attrs.iter().filter(|a| a.is_builtin()) {
        match a.name.as_str().as_str() {
            "throws" => {
                let names = parse_names(a, "throws")?;
                let e = decl.throws.get_or_insert_with(Vec::new);
                for n in names {
                    if !e.contains(&n) { e.push(n); }
                }
            }
            "eff" => {
                let names = parse_names(a, "eff")?;
                for n in &names {
                    if !KNOWN_EFFECTS.contains(&n.as_str().as_str()) {
                        return Err(Error::Hir(format!(
                            "unknown effect `{}` (supported: {}) (at {}:{})",
                            n, KNOWN_EFFECTS.join(", "),
                            a.span.start_line, a.span.start_col
                        )));
                    }
                }
                let e = decl.effs.get_or_insert_with(Vec::new);
                for n in names {
                    if !e.contains(&n) { e.push(n); }
                }
            }
            // `#[pure]` = 显式空 eff 集合
            "pure" if decl.effs.is_none() => decl.effs = Some(Vec::new()),
            _ => {}
        }
    }
    Ok(decl)
}

fn parse_names(a: &Attr, kind: &str) -> Result<Vec<Symbol>> {
    let mut out = Vec::new();
    for arg in &a.args {
        match arg {
            AttrArg::Expr(e) => match e.as_ref() {
                Expr::Ident(s, _) => out.push(*s),
                _ => return Err(bad_arg(kind, a)),
            },
            AttrArg::KeyValue(..) => return Err(bad_arg(kind, a)),
        }
    }
    Ok(out)
}

fn bad_arg(kind: &str, a: &Attr) -> Error {
    Error::Hir(format!(
        "#[{}] expects effect/type names (at {}:{})",
        kind, a.span.start_line, a.span.start_col
    ))
}
