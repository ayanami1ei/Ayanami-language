//! `static_assert(cond)` 内置：编译期断言（条件须为常量；false 位置化报错）。
//!
//! Phase 12（P4e-2）：复用 `const_eval` 求值（含 `#[compile_time]` 函数调用）。
use super::*;

impl crate::hir::lower::Ctx {
    /// `static_assert(cond)` —— 条件求值为 `true` 则空操作（Void 常量）；否则编译错。
    pub(crate) fn lower_static_assert(&mut self, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        if args.len() != 1 {
            return Err(Error::Hir(format!(
                "static_assert expects exactly 1 condition argument (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        let (ty, lit) = self.eval_const_expr(&args[0], &HashMap::new(), 0).map_err(|e| {
            Error::Hir(format!(
                "static_assert condition must be a compile-time constant: {} (at {}:{})",
                e, span.start_line, span.start_col
            ))
        })?;
        match lit {
            HirLiteral::Bool(true) => {}
            HirLiteral::Bool(false) => {
                return Err(Error::Hir(format!(
                    "static assertion failed (at {}:{})",
                    span.start_line, span.start_col
                )));
            }
            _ => {
                return Err(Error::Hir(format!(
                    "static_assert condition must be a bool constant (got `{}`) (at {}:{})",
                    hir_type_display(&ty), span.start_line, span.start_col
                )));
            }
        }
        Ok(SConst { val: HirLiteral::Int(0), ty: HirType::Void }.into())
    }
}
