//! 返回类型一致性检查（P4b 附带）：非数值/非装箱的类别不匹配报错
//! （此前会静默生成坏 IR，如 `fn f() -> int { return Box[int]{...} }`）。
//!
//! 未标注 lambda 的返回类型此时尚未推断（`fn_ret` 暂为 void），延后校验（#169）。

use super::*;

impl crate::hir::lower::Ctx {
    /// 返回类型一致性检查（`lambda_ret_pending` 时延后）。
    pub(crate) fn check_return_type(&self, wrapped: &HirNodeBox, fn_ret: &HirType, span: &Span) -> Result<()> {
        if self.lambda_ret_pending {
            return Ok(());
        }
        let got = strip_ownership_ref(&expr_type(wrapped)).clone();
        let want = strip_ownership_ref(fn_ret).clone();
        let numeric = |t: &HirType| matches!(t,
            HirType::Int | HirType::Float | HirType::F32 | HirType::Char
            | HirType::Bool | HirType::IntN { .. });
        let iface_ok = matches!(want, HirType::FatPtr { .. });
        // `return null`：整型字面量 0 作空指针
        let null_ptr_ok = as_int_literal(wrapped).is_some()
            && matches!(want, HirType::RawPtr(_) | HirType::Ref(..) | HirType::Unique(_) | HirType::FnPtr(..));
        if !return_types_compatible(&got, &want) && !(numeric(&got) && numeric(&want)) && !iface_ok && !null_ptr_ok {
            return Err(Error::Hir(format!(
                "return type mismatch: expected `{}`, found `{}` (at {}:{})",
                hir_type_display(fn_ret), hir_type_display(&expr_type(wrapped)),
                span.start_line, span.start_col
            )));
        }
        Ok(())
    }
}

/// 返回类型兼容：完全相等；闭包参数/返回一致且 `Fn` 可作 `FnOnce`（子类型），
/// owns_env 忽略（静态 lambda 可作 `Fn(...)`）；拥有数组 `[T; n]` 与 `[T]` 尺寸擦除等价；
/// `!` 为任意类型子类型。
fn return_types_compatible(got: &HirType, want: &HirType) -> bool {
    if got == want { return true; }
    let array_like = |t: &HirType| matches!(t, HirType::Array(_) | HirType::ArraySized(_, _));
    match (got, want) {
        (HirType::Never, _) => true,
        (HirType::Closure(gp, gr, _, g_once), HirType::Closure(wp, wr, _, w_once)) =>
            (!*g_once || *w_once) && gp == wp && gr == wr,
        (HirType::Unique(a), HirType::Unique(b)) if array_like(a) && array_like(b) => true,
        (a, b) if array_like(a) && array_like(b) => true,
        _ => false,
    }
}
