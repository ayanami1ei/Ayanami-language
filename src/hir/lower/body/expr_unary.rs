//! 一元运算符降级（`lower_unary`；从 expr_ops1.rs 拆出，保持文件 ≤300 行）。

use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_unary(&mut self, op: &UnaryOp, arg: &Box<Expr>) -> Result<HirNodeBox> {
        // M5 P2：裸指针解引用 `*p`（读需 unsafe；仅接受 `*T`）
        if matches!(op, UnaryOp::Deref) {
            let span = arg.span();
            self.require_unsafe("dereference of a raw pointer", &span)?;
            let hir_arg = self.lower_expr(arg)?;
            let arg_ty = expr_type(&hir_arg);
            let pointee = match strip_ownership_ref(&arg_ty) {
                HirType::RawPtr(inner) => inner.as_ref().clone(),
                other => return Err(Error::Hir(format!(
                    "cannot dereference `{}`: expected a raw pointer `*T` (at {}:{})",
                    hir_type_display(other), span.start_line, span.start_col
                ))),
            };
            return Ok(SDeref { expr: hir_arg, ty: pointee }.into());
        }
        let hir_arg = auto_deref(self.lower_expr(arg)?);
        let arg_ty = expr_type(&hir_arg);
        let inner_ty = strip_ownership(arg_ty.clone());
        // M1.2：`~` 仅适用于整数/char
        if matches!(op, UnaryOp::BitNot) && matches!(inner_ty, HirType::Float | HirType::Bool) {
            return Err(Error::Hir(format!(
                "cannot apply `~` to `{}` (at {}:{})",
                hir_type_display(&inner_ty),
                arg.span().start_line, arg.span().start_col
            )));
        }
        // Try operator overloading (skip for primitive types)
        let is_primitive = matches!(&inner_ty, HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. });
        if !is_primitive {
            if let Some(op_fn_name) = unary_op_to_fn_name(op) {
                let param_types = [arg_ty.clone()];
                if let Some(fn_id) = self.resolve_fn_call(&Symbol::intern(op_fn_name), &param_types) {
                    let ret_ty = self.fns[fn_id.0].return_type.clone();
                    let mut args = vec![implicit_move(hir_arg)];
                    self.append_caller_args(fn_id, &mut args, &arg.span());
                    return Ok(SCall { fn_id, args, ty: ret_ty }.into());
                }
            }
        }
        let ty = if matches!(op, UnaryOp::Not) {
            HirType::Bool
        } else {
            strip_ownership(arg_ty)
        };
        Ok(SUn {
            op: *op,
            arg: hir_arg,
            ty,
        }.into())
    }
}
