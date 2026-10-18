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

    /// Phase 7 续：`offset_of[T](field)` —— 字段偏移（AST 层读字段标识符，不求值）。
    pub(crate) fn lower_offset_of(
        &mut self,
        args: &Vec<Expr>,
        explicit: Option<&Vec<Type>>,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let Some(tys) = explicit else {
            return Err(Error::Hir(format!(
                "`offset_of` requires an explicit type argument: `offset_of[T](field)` (at {}:{})",
                span.start_line, span.start_col
            )));
        };
        if tys.len() != 1 || args.len() != 1 {
            return Err(Error::Hir(format!(
                "`offset_of` expects exactly 1 type argument and 1 field identifier (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        let field = match &args[0] {
            Expr::Ident(f, _) => *f,
            _ => {
                return Err(Error::Hir(format!(
                    "`offset_of` expects a field identifier: `offset_of[T](field)` (at {}:{})",
                    span.start_line, span.start_col
                )));
            }
        };
        let ty = ast_type_to_hir(&tys[0], &self.interfaces);
        let defs: crate::hir::layout::StructDefs = self.struct_defs.iter()
            .map(|(k, fs)| (*k, fs.iter().map(|f| (f.name, f.ty.clone())).collect()))
            .collect();
        let offset = crate::hir::layout::field_offset(&ty, field, &defs).ok_or_else(|| {
            Error::Hir(format!(
                "`offset_of`: `{}` has no field `{}` (or layout unknown) (at {}:{})",
                hir_type_display(&ty), field.as_str(), span.start_line, span.start_col
            ))
        })?;
        Ok(SConst { val: HirLiteral::Int(offset as i64), ty: HirType::Int }.into())
    }
}
