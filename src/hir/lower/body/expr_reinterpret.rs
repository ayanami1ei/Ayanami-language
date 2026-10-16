//! `bits::reinterpret[T](x)` 内置：位级重解释（同尺寸 bitcast/ptrtoint/inttoptr）。
//!
//! 从 expr_call.rs 拆出（保持文件 ≤300 行）。

use super::*;

impl crate::hir::lower::Ctx {
    /// P3：`bits::reinterpret[T](x)` —— 需 unsafe；两侧限同尺寸 Copy 类型。
    pub(crate) fn lower_reinterpret_builtin(
        &mut self,
        hir_args: &mut Vec<HirNodeBox>,
        explicit: Option<&Vec<Type>>,
        span: &Span,
    ) -> Result<HirNodeBox> {
        self.require_unsafe("`bits::reinterpret`", span)?;
        let Some(tys) = explicit else {
            return Err(Error::Hir(format!(
                "bits::reinterpret requires an explicit target type: `bits::reinterpret[T](x)` (at {}:{})",
                span.start_line, span.start_col
            )));
        };
        if tys.len() != 1 || hir_args.len() != 1 {
            return Err(Error::Hir(format!(
                "bits::reinterpret expects exactly 1 type argument and 1 value argument (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        let target = ast_type_to_hir(&tys[0], &self.interfaces);
        let arg = hir_args.pop().unwrap();
        let src = expr_type(&arg);
        let (Some(ss), Some(ds)) = (reinterpret_size(&src), reinterpret_size(&target)) else {
            return Err(Error::Hir(format!(
                "bits::reinterpret supports primitive/pointer Copy types only (got `{}` -> `{}`) (at {}:{})",
                hir_type_display(&src), hir_type_display(&target), span.start_line, span.start_col
            )));
        };
        if ss != ds {
            return Err(Error::Hir(format!(
                "bits::reinterpret size mismatch: `{}` ({} bytes) -> `{}` ({} bytes) (at {}:{})",
                hir_type_display(&src), ss, hir_type_display(&target), ds, span.start_line, span.start_col
            )));
        }
        return Ok(SReinterpret { expr: arg, ty: target }.into());
    }
}

/// P3：`bits::reinterpret` 可接受的类型尺寸（基元/指针等 Copy 类型；None=不支持）。
fn reinterpret_size(ty: &HirType) -> Option<u64> {
    match ty {
        HirType::Int | HirType::Float | HirType::Ref(..) | HirType::FnPtr(..)
        | HirType::RawPtr(_) | HirType::Unique(_) => Some(8),
        HirType::F32 => Some(4),
        HirType::Char | HirType::Bool => Some(1),
        HirType::IntN { bits, .. } => Some((*bits / 8) as u64),
        _ => None,
    }
}
