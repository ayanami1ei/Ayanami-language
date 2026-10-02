//! A2c：函数契约标注的校验与提取（`#[assume]`；requires/ensures 后续接入）。

use crate::error::{Error, Result};
use crate::parser::ast::{Attr, AttrArg, Expr};

/// 校验函数级契约标注的参数形态（假定已通过白名单校验）。
pub fn validate_fn_attrs(attrs: &[Attr]) -> Result<()> {
    for a in attrs {
        if a.name.as_str() == "assume"
            && (a.args.len() != 1 || !matches!(&a.args[0], AttrArg::Expr(_)))
        {
            return Err(Error::Hir(format!(
                "#[assume] requires exactly one condition expression (at {}:{})",
                a.span.start_line, a.span.start_col
            )));
        }
    }
    Ok(())
}

/// 提取 `#[assume(cond)]` 条件表达式（按声明顺序）。
pub fn assume_conditions(attrs: &[Attr]) -> Vec<&Expr> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.name.as_str() == "assume") {
        if let Some(AttrArg::Expr(e)) = a.args.first() {
            out.push(e.as_ref());
        }
    }
    out
}
