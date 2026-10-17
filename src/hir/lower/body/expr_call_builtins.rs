//! 调用位置的编译器内置构造（`Ptr::new` 等）；从 expr_call.rs 拆出。
use super::*;

impl crate::hir::lower::Ctx {
    /// `Ptr::new(v)`：堆分配拥有指针（稳定地址）
    pub(super) fn lower_ptr_new(&mut self, hir_args: &mut Vec<HirNodeBox>, span: &Span) -> Result<HirNodeBox> {
        if hir_args.len() != 1 {
            return Err(Error::Hir(format!(
                "Ptr::new expects exactly 1 argument (at {}:{})", span.start_line, span.start_col
            )));
        }
        let arg = hir_args.pop().unwrap();
        let inner = strip_ownership(expr_type(&arg));
        // 阶段 1：仅指针表示内层类型可装箱（标量 unique 当前非指针表示）
        if !matches!(inner,
            HirType::Named(_) | HirType::FatPtr { .. } | HirType::Unique(_)
            | HirType::Array(_) | HirType::ArraySized(_, _) | HirType::Closure(..))
        {
            return Err(Error::Hir(format!(
                "Ptr[T] currently supports struct/enum/interface/array elements only (got {}) (at {}:{})",
                hir_type_display(&inner), span.start_line, span.start_col
            )));
        }
        Ok(SToUnique { expr: arg, ty: HirType::Unique(Box::new(inner)) }.into())
    }
}
