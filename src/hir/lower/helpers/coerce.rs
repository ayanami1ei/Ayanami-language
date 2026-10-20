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
    // 比较表达式在 HIR 中保留操作数类型，MIR→LIR 才产出 bool（i1）
    if expr.is_comparison() {
        if tgt_inner == HirType::Bool {
            // #182：结果类型修正为 Bool（值即 i1），否则下游按操作数类型发射
            // （结构体 bool 字段 store i64、Option::Some(a==b) 实例化成 Option[int] 等）
            return Ok(retype_comparison_bool(expr));
        }
        if matches!(tgt_inner, HirType::Int | HirType::IntN { .. } | HirType::Float | HirType::F32) {
            return Ok(SBoolToNum { expr, ty: tgt_inner }.into());
        }
    }
    if src_inner == tgt_inner {
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

/// #182：比较表达式结果类型修正为 Bool（发射结果本就是 i1，bitcast i1→i1 合法且会被折叠）。
/// 比较在 HIR 中保留操作数类型（A2c），此处在目标类型为 Bool 的上下文中显式标注，
/// 避免下游按操作数类型发射（结构体 bool 字段 store i64、Option::Some(a==b) 成 Option[int] 等）。
pub(crate) fn retype_comparison_bool(expr: HirNodeBox) -> HirNodeBox {
    if expr.expr_type() == HirType::Bool || !expr.is_comparison() {
        return expr;
    }
    SBoolToNum { expr, ty: HirType::Bool }.into()
}

/// #201：赋值类型兼容判定（在 `coerce_expr` 等隐式转换之后仍不一致时调用）。
/// 不兼容 → 上游报明确类型错误，避免结构体/聚合值进入基元槽位产生非法 IR。
pub(crate) fn assign_compatible(src: &HirType, dst: &HirType) -> bool {
    if src == dst {
        return true;
    }
    match (src, dst) {
        // 引用：自动解引用/重新借用由更早的转换处理，此处放行
        (HirType::Ref(..), _) | (_, HirType::Ref(..)) => true,
        // 基元：隐式数值转换已在 coerce_expr 处理
        (a, b) if is_primitive(a) && is_primitive(b) => true,
        // null 字面量 `0` 赋给指针/拥有指针（既有惯用法）
        (HirType::Int, HirType::RawPtr(_)) | (HirType::Int, HirType::Unique(_)) => true,
        (HirType::Unique(a), HirType::Unique(b)) => assign_compatible(a, b),
        // 数组：定长/动态（`[T; n]` ↔ `[T]`）表示一致（同为堆指针），元素类型需兼容
        (HirType::Array(a), HirType::Array(b))
        | (HirType::ArraySized(a, _), HirType::ArraySized(b, _))
        | (HirType::ArraySized(a, _), HirType::Array(b))
        | (HirType::Array(a), HirType::ArraySized(b, _)) => assign_compatible(a, b),
        // 基名相同的泛型实例化（基/具体名混用为既有行为）
        (HirType::Named(a), HirType::Named(b)) => {
            crate::hir::lower::strip_generic_name(a) == crate::hir::lower::strip_generic_name(b)
        }
        (HirType::FatPtr { name: a, .. }, HirType::FatPtr { name: b, .. }) => a == b,
        (HirType::Closure(..), HirType::Closure(..))
        | (HirType::FnPtr(..), HirType::FnPtr(..))
        | (HirType::Closure(..), HirType::FatPtr { .. })
        | (HirType::FnPtr(..), HirType::FatPtr { .. }) => true,
        _ => false,
    }
}

/// #201：赋值目标类型检查（赋值 / 字段写入 / 解引用写入共用）。
/// 不兼容 → 明确类型错误（此前进入 llc 产生非法 IR）。
pub(crate) fn ensure_assignable(
    value: &HirNodeBox,
    dst: &HirType,
    span: &Span,
    what: &str,
) -> Result<()> {
    let vty = if value.is_comparison() { HirType::Bool } else { value.expr_type() };
    if assign_compatible(&vty, dst) {
        return Ok(());
    }
    Err(Error::Hir(format!(
        "type mismatch: cannot assign `{}` to {} (at {}:{})",
        hir_type_display(&vty), what, span.start_line, span.start_col
    )))
}
