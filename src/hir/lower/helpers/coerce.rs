use super::*;
use crate::error::{Error, Result};

/// M1.3b（tbo）：不再有隐式数值提升 —— char→int / char→float / int→float 均需显式 `as`。
/// 保留函数以维持调用点结构（如需恢复隐式规则在此处加回）。
pub(crate) fn implicit_cast_ok(from: &HirType, to: &HirType) -> bool {
    let _ = (from, to);
    false
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

/// 整数字面量节点？（SConst Int）
pub(crate) fn as_int_literal(e: &HirNodeBox) -> Option<i64> {
    match e.as_const() {
        Some(HirLiteral::Int(n)) => Some(*n),
        _ => e.as_neg_int_literal(),
    }
}

/// 浮点字面量节点？（SConst Float）
pub(crate) fn as_float_literal(e: &HirNodeBox) -> Option<f64> {
    match e.as_const() {
        Some(HirLiteral::Float(n)) => Some(*n),
        _ => e.as_neg_float_literal(),
    }
}

/// 数值字面量（整数或浮点）
pub(crate) fn is_numeric_literal(e: &HirNodeBox) -> bool {
    as_int_literal(e).is_some() || as_float_literal(e).is_some()
}

/// M1.8a：索引表达式统一为整数（与目标索引类型做显式整数转换）
pub(crate) fn coerce_index(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox> {
    let src = strip_ownership_ref(&expr.expr_type()).clone();
    let tgt = strip_ownership_ref(target).clone();
    if src == tgt {
        return Ok(expr);
    }
    if is_int_type(&src) && is_int_type(&tgt) {
        return Ok(SCast { expr, ty: tgt }.into());
    }
    Err(Error::Hir(format!(
        "index must be an integer, found `{}` (at {}:{})",
        hir_type_display(&src), span.start_line, span.start_col
    )))
}

/// 浮点类型（float / f32）
pub(crate) fn is_float_type(ty: &HirType) -> bool {
    matches!(strip_ownership_ref(ty), HirType::Float | HirType::F32)
}

/// 把浮点字面量重定型为目标类型（非字面量原样返回）
pub(crate) fn retype_float_literal(expr: HirNodeBox, target: &HirType) -> HirNodeBox {
    if let Some(n) = as_float_literal(&expr) {
        return SConst { val: HirLiteral::Float(n), ty: strip_ownership_ref(target).clone() }.into();
    }
    // 整数字面量 → 浮点（Rust 风格字面量推断，如 f(2) / if 分支 1.5 与 2）
    if let Some(n) = as_int_literal(&expr) {
        return SConst { val: HirLiteral::Float(n as f64), ty: strip_ownership_ref(target).clone() }.into();
    }
    expr
}

/// 整数类型（int 或定宽整数）
pub(crate) fn is_int_type(ty: &HirType) -> bool {
    matches!(strip_ownership_ref(ty), HirType::Int | HirType::IntN { .. })
}

/// 把整数字面量重定型为目标整数类型（非字面量原样返回）
pub(crate) fn retype_int_literal(expr: HirNodeBox, target: &HirType) -> HirNodeBox {
    match as_int_literal(&expr) {
        Some(n) => SConst { val: HirLiteral::Int(n), ty: strip_ownership_ref(target).clone() }.into(),
        None => expr,
    }
}

fn is_primitive(ty: &HirType) -> bool {
    matches!(ty, HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. })
}

/// 按目标类型对表达式做隐式转换：
/// - 类型相同或属于允许的拓宽转换 → 插入 SCast（基元目标类型，不含所有权包装）
/// - 两个基元之间无转换规则 → 明确报错（避免生成非法 LLVM IR）
/// - 其余情况原样返回，交给既有逻辑处理
pub(crate) fn coerce_expr(expr: HirNodeBox, target: &HirType, span: &Span) -> Result<HirNodeBox> {
    // ref T → T：值上下文自动解引用（目标本身是引用时保持原样，支持别名）
    let expr = if matches!(target, HirType::Ref(..)) { expr } else { auto_deref(expr) };
    let src = expr.expr_type();
    // M1.9：`!` 是任意类型的子类型（发散表达式；无运行期转换）
    if matches!(strip_ownership_ref(&src), HirType::Never) {
        return Ok(expr);
    }
    // 目标为 `!` 但源不是发散表达式 → 明确报错
    if matches!(strip_ownership_ref(target), HirType::Never) {
        return Err(Error::Hir(format!(
            "cannot implicitly convert `{}` to `!` (at {}:{})",
            hir_type_display(&src), span.start_line, span.start_col
        )));
    }
    let src_inner = strip_ownership_ref(&src).clone();
    let tgt_inner = strip_ownership_ref(target).clone();
    // 整数字面量适配定宽整数（如 f(1) → i32 形参；Rust 风格字面量推断）
    if let HirType::IntN { .. } = &tgt_inner {
        if src_inner == HirType::Int && as_int_literal(&expr).is_some() {
            return Ok(retype_int_literal(expr, &tgt_inner));
        }
    }
    // 浮点字面量适配（f32/f64；整数/浮点字面量 → 浮点期望类型）
    if matches!(tgt_inner, HirType::Float | HirType::F32) {
        let lit_ok = (src_inner == HirType::Float && as_float_literal(&expr).is_some())
            || (src_inner == HirType::Int && as_int_literal(&expr).is_some());
        if lit_ok {
            return Ok(retype_float_literal(expr, &tgt_inner));
        }
    }
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
