//! 数组接收者的内置方法（P3b：`get_unchecked`；从 expr_method.rs 拆出）。

use super::*;

/// 数组接收者（含 `ref`/`Unique` 包装）的元素类型；非数组返回 None。
pub(crate) fn array_elem_type(receiver_ty: &HirType) -> Option<HirType> {
    let deref_ty = strip_ownership(match receiver_ty {
        HirType::Ref(inner, _) => inner.as_ref().clone(),
        other => other.clone(),
    });
    match &deref_ty {
        HirType::Array(elem) | HirType::ArraySized(elem, _) => Some(elem.as_ref().clone()),
        _ => None,
    }
}

impl crate::hir::lower::Ctx {
    /// `arr.get_unchecked(i)`：跳过越界检查（需 unsafe）。
    pub(crate) fn lower_get_unchecked(
        &mut self,
        receiver: HirNodeBox,
        hir_args: &Vec<HirNodeBox>,
        elem_ty: HirType,
        span: &Span,
    ) -> Result<HirNodeBox> {
        self.require_unsafe("`get_unchecked`", span)?;
        let index = coerce_index(hir_args[0].clone(), &HirType::Int, span)?;
        Ok(SIdx { object: receiver, index, ty: elem_ty }.into())
    }
}
