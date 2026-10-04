use super::*;
use crate::error::{Error, Result};

/// 允许的隐式数值转换（仅无损拓宽）：
/// char -> int、char -> float、int -> float。
/// 比较时忽略 ref/unique 等所有权包装。
pub(crate) fn implicit_cast_ok(from: &HirType, to: &HirType) -> bool {
    matches!(
        (strip_ownership_ref(from), strip_ownership_ref(to)),
        (HirType::Char, HirType::Int)
            | (HirType::Char, HirType::Float)
            | (HirType::Int, HirType::Float)
    )
}

/// `ref T` 在值上下文自动解引用为 `T`（其他表达式原样返回）
pub(crate) fn auto_deref(expr: HirNodeBox) -> HirNodeBox {
    if let HirType::Ref(inner, _) = expr.expr_type() {
        SDeref { expr, ty: *inner }.into()
    } else {
        expr
    }
}

/// 重载解析用的解引用类型（`Ref(T)` → `T`）
pub(crate) fn deref_type(ty: &HirType) -> HirType {
    match ty {
        HirType::Ref(inner, _) => (**inner).clone(),
        _ => ty.clone(),
    }
}

fn is_primitive(ty: &HirType) -> bool {
    matches!(ty, HirType::Int | HirType::Float | HirType::Char | HirType::Bool)
}

/// 按目标类型对表达式做隐式转换：
/// - 类型相同或属于允许的拓宽转换 → 插入 SCast（基元目标类型，不含所有权包装）
/// - 两个基元之间无转换规则 → 明确报错（避免生成非法 LLVM IR）
/// - 其余情况原样返回，交给既有逻辑处理
pub(crate) fn coerce_expr(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox> {
    // ref T → T：值上下文自动解引用（目标本身是引用时保持原样，支持别名）
    let expr = if matches!(target, HirType::Ref(..)) { expr } else { auto_deref(expr) };
    let src = expr.expr_type();
    let src_inner = strip_ownership_ref(&src).clone();
    let tgt_inner = strip_ownership_ref(target).clone();
    if src_inner == tgt_inner {
        return Ok(expr);
    }
    // 比较表达式在 HIR 中保留操作数类型，MIR→LIR 才产出 bool
    if expr.is_comparison() && tgt_inner == HirType::Bool {
        return Ok(expr);
    }
    if implicit_cast_ok(&src_inner, &tgt_inner) {
        return Ok(SCast { expr, ty: tgt_inner }.into());
    }
    if is_primitive(&src_inner) && is_primitive(&tgt_inner) {
        return Err(Error::Hir(format!(
            "cannot implicitly convert `{}` to `{}` (at {}:{})",
            hir_type_display(&src), hir_type_display(target),
            span.start_line, span.start_col
        )));
    }
    Ok(expr)
}
