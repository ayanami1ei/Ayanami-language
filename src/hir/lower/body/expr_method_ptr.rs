//! M5 P2：裸指针方法（`offset`/`read_volatile`/`write_volatile`）——从 expr_method.rs 拆出。
use super::*;

impl crate::hir::lower::Ctx {
    /// 裸指针接收者的内置方法；非裸指针或未命中返回 `None`。
    pub(crate) fn lower_rawptr_method(
        &mut self,
        receiver: &HirNodeBox,
        receiver_ty: &HirType,
        method: &Symbol,
        hir_args: &[HirNodeBox],
        span: &Span,
    ) -> Result<Option<HirNodeBox>> {
        let HirType::RawPtr(inner) = receiver_ty else { return Ok(None) };
        let inner_ty = inner.as_ref().clone();
        match method.as_str().as_str() {
            "offset" if hir_args.len() == 1 => {
                self.require_unsafe("`offset` on a raw pointer", span)?;
                let index = coerce_index(hir_args[0].clone(), &HirType::Int, span)?;
                return Ok(Some(SPtrOffset { ptr: receiver.clone(), index, ty: receiver_ty.clone() }.into()));
            }
            "read_volatile" if hir_args.is_empty() => {
                self.require_unsafe("`read_volatile` on a raw pointer", span)?;
                if !inner_ty.is_copy() {
                    return Err(Error::Hir(format!(
                        "`read_volatile` requires a Copy pointee (got `{}`) (at {}:{})",
                        hir_type_display(&inner_ty), span.start_line, span.start_col
                    )));
                }
                return Ok(Some(SVolatileLoad { ptr: receiver.clone(), ty: inner_ty }.into()));
            }
            "write_volatile" if hir_args.len() == 1 => {
                self.require_unsafe("`write_volatile` on a raw pointer", span)?;
                if !inner_ty.is_copy() {
                    return Err(Error::Hir(format!(
                        "`write_volatile` requires a Copy pointee (got `{}`) (at {}:{})",
                        hir_type_display(&inner_ty), span.start_line, span.start_col
                    )));
                }
                let value = coerce_expr(hir_args[0].clone(), &inner_ty, span)?;
                return Ok(Some(SVolatileStore { ptr: receiver.clone(), value, ty: HirType::Void }.into()));
            }
            _ => {}
        }
        Ok(None)
    }
}
