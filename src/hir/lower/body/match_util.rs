//! Phase 1.3：模式匹配公共工具（标量模式校验 / 字面量 / 分支结果类型 / 发散判断）。
use super::*;

/// 字面量/区间模式对 scrutinee 类型的合法性检查（区间仅整数/char）
pub(super) fn check_scalar_pattern(base: &HirType, lit: &crate::parser::ast::Literal, range: bool, span: &Span) -> Result<()> {
    use crate::parser::ast::Literal;
    let ok = match base {
        HirType::Int | HirType::IntN { .. } => matches!(lit, Literal::Int(..)),
        HirType::Char => matches!(lit, Literal::Char(..)),
        HirType::Bool => matches!(lit, Literal::Bool(..)),
        HirType::Float | HirType::F32 => !range && matches!(lit, Literal::Float(..) | Literal::Int(..)),
        _ => false,
    };
    if !ok {
        return Err(Error::Hir(format!(
            "{} pattern cannot match `{}` (at {}:{})",
            if range { "range" } else { "literal" },
            hir_type_display(base), span.start_line, span.start_col
        )));
    }
    Ok(())
}

/// AST 字面量 → HIR 字面量
pub(super) fn hir_literal(l: &crate::parser::ast::Literal) -> HirLiteral {
    use crate::parser::ast::Literal;
    match l {
        Literal::Int(i, _) => HirLiteral::Int(*i),
        Literal::Float(f, _) => HirLiteral::Float(*f),
        Literal::Char(c, _) => HirLiteral::Char(*c),
        Literal::Bool(b, _) => HirLiteral::Bool(*b),
        Literal::String(s, _) => HirLiteral::String(s.clone()),
    }
}

/// match/if 分支公共结果类型（数值提升；其余取首个分支类型）
pub(super) fn match_result_type(a: &HirType, b: &HirType) -> HirType {
    let sa = strip_ownership(a.clone());
    let sb = strip_ownership(b.clone());
    // M1.9：`!` 是单位元（发散分支不影响公共类型）
    if sa == HirType::Never { return sb; }
    if sb == HirType::Never { return sa; }
    if sa == sb { return sa; }
    match (&sa, &sb) {
        (HirType::Float, HirType::Int) | (HirType::Int, HirType::Float)
        | (HirType::Float, HirType::Char) | (HirType::Char, HirType::Float) => HirType::Float,
        (HirType::Char, HirType::Int) | (HirType::Int, HirType::Char) => HirType::Int,
        _ => sa,
    }
}

/// M1.9：块是否发散（末尾为 `return` 或 `!` 类型表达式语句）
pub(super) fn block_diverges(b: &HirBlock) -> bool {
    match b.stmts.last() {
        Some(HirStmt::Return { .. }) => true,
        Some(HirStmt::Expr { expr, .. }) => matches!(strip_ownership(expr_type(expr)), HirType::Never),
        _ => false,
    }
}
