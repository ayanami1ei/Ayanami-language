//! A2c：函数契约标注的校验与提取（`#[assume]`；requires/ensures 后续接入）。

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, AttrArg, Expr};

/// 校验函数级契约标注的参数形态（假定已通过白名单校验）。
pub fn validate_fn_attrs(attrs: &[Attr]) -> Result<()> {
    for a in attrs {
        let needs_expr = matches!(a.name.as_str().as_str(), "assume" | "requires" | "ensures");
        if needs_expr && (a.args.len() != 1 || !matches!(&a.args[0], AttrArg::Expr(_))) {
            return Err(Error::Hir(format!(
                "#[{}] requires exactly one condition expression (at {}:{})",
                a.name, a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

/// 契约条件必须是 bool 值或比较运算（HIR 中比较保持操作数类型）。
pub fn ensure_bool_condition(cond: &crate::hir::HirNodeBox, kind: &str, line: usize, col: usize) -> Result<()> {
    if cond.expr_type() != crate::hir::HirType::Bool && !cond.is_comparison() {
        return Err(Error::Hir(format!(
            "#[{}] condition must be bool (at {}:{})", kind, line, col
        )));
    }
    Ok(())
}

/// `AYANAMI_CHECKS=0` 关闭运行检查（requires/ensures 退化为 assume），默认开启。
pub fn checks_enabled() -> bool {
    std::env::var("AYANAMI_CHECKS").map(|v| v != "0").unwrap_or(true)
}

/// 提取 `#[requires(cond)]` 条件（表达式 + 行列），按声明顺序。
pub fn requires_conditions(attrs: &[Attr]) -> Vec<(&Expr, usize, usize)> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.is_builtin() && a.name.as_str() == "requires") {
        if let Some(AttrArg::Expr(e)) = a.args.first() {
            out.push((e.as_ref(), a.span.start_line, a.span.start_col));
        }
    }
    out
}

/// 提取 `#[ensures(cond)]` 条件（表达式 + 行列），按声明顺序。
pub fn ensure_conditions(attrs: &[Attr]) -> Vec<(&Expr, usize, usize)> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.is_builtin() && a.name.as_str() == "ensures") {
        if let Some(AttrArg::Expr(e)) = a.args.first() {
            out.push((e.as_ref(), a.span.start_line, a.span.start_col));
        }
    }
    out
}

/// 提取 `#[assume(cond)]` 条件表达式（按声明顺序）。
pub fn assume_conditions(attrs: &[Attr]) -> Vec<&Expr> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.is_builtin() && a.name.as_str() == "assume") {
        if let Some(AttrArg::Expr(e)) = a.args.first() {
            out.push(e.as_ref());
        }
    }
    out
}
