//! `size_of[T]()` / `align_of[T]()` 内置：编译期常量（布局见 hir/layout.rs）。
//!
//! 仅当带显式类型实参时拦截（无类型实参的同名用户函数不受影响）。

use super::*;

impl crate::hir::lower::Ctx {
    /// Phase 7：`size_of[T]()` / `align_of[T]()` —— 目标类型字节大小 / 对齐（i64 常量）。
    pub(crate) fn lower_layout_builtin(
        &mut self,
        name: &Symbol,
        explicit: Option<&Vec<Type>>,
        hir_args: &mut Vec<HirNodeBox>,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let Some(tys) = explicit else {
            return Err(Error::Hir(format!(
                "`{}` requires an explicit type argument: `{}[T]()` (at {}:{})",
                name.as_str(), name.as_str(), span.start_line, span.start_col
            )));
        };
        if tys.len() != 1 || !hir_args.is_empty() {
            return Err(Error::Hir(format!(
                "`{}` expects exactly 1 type argument and no value arguments (at {}:{})",
                name.as_str(), span.start_line, span.start_col
            )));
        }
        let ty = ast_type_to_hir(&tys[0], &self.interfaces);
        let defs: crate::hir::layout::StructDefs = self.struct_defs.iter()
            .map(|(k, fs)| (*k, fs.iter().map(|f| (f.name, f.ty.clone())).collect()))
            .collect();
        let (size, align) = crate::hir::layout::type_layout(&ty, &defs);
        let v = if name.as_str() == "align_of" { align } else { size };
        Ok(SConst { val: HirLiteral::Int(v as i64), ty: HirType::Int }.into())
    }
}
