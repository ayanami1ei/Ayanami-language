//! `size_of[T]()` / `align_of[T]()` / `offset_of[T](field)` 内置：编译期常量
//! （布局见 hir/layout.rs）。运行期 lowering 与 const 上下文（const_eval）共用求值。
//!
//! 仅当带显式类型实参时拦截（无类型实参的同名用户函数不受影响）。

use super::*;

impl crate::hir::lower::Ctx {
    /// 布局内置统一求值：返回 `Some((Int, 值))`；非布局内置返回 `None`。
    pub(crate) fn eval_layout_builtin_call(
        &self,
        name: &Symbol,
        generic_args: &Vec<Type>,
        args: &Vec<Expr>,
        span: &Span,
    ) -> Result<Option<(HirType, HirLiteral)>> {
        let is_layout = matches!(name.as_str().as_str(), "size_of" | "align_of" | "offset_of");
        if !is_layout || generic_args.is_empty() {
            return Ok(None);
        }
        if generic_args.len() != 1 {
            return Err(Error::Hir(format!(
                "`{}` expects exactly 1 type argument (at {}:{})",
                name.as_str(), span.start_line, span.start_col
            )));
        }
        let ty = ast_type_to_hir(&generic_args[0], &self.interfaces);
        let defs: crate::hir::layout::StructDefs = self.struct_defs.iter()
            .map(|(k, fs)| (*k, fs.iter().map(|f| (f.name, f.ty.clone())).collect()))
            .collect();
        let value: u64 = if name.as_str() == "offset_of" {
            let field = match args.first() {
                Some(Expr::Ident(f, _)) if args.len() == 1 => *f,
                _ => {
                    return Err(Error::Hir(format!(
                        "`offset_of` expects a field identifier: `offset_of[T](field)` (at {}:{})",
                        span.start_line, span.start_col
                    )));
                }
            };
            crate::hir::layout::field_offset(&ty, field, &defs).ok_or_else(|| {
                Error::Hir(format!(
                    "`offset_of`: `{}` has no field `{}` (or layout unknown) (at {}:{})",
                    hir_type_display(&ty), field.as_str(), span.start_line, span.start_col
                ))
            })?
        } else {
            if !args.is_empty() {
                return Err(Error::Hir(format!(
                    "`{}` expects no value arguments (at {}:{})",
                    name.as_str(), span.start_line, span.start_col
                )));
            }
            let (size, align) = crate::hir::layout::type_layout(&ty, &defs);
            if name.as_str() == "align_of" { align } else { size }
        };
        Ok(Some((HirType::Int, HirLiteral::Int(value as i64))))
    }

    /// 运行期 lowering：`size_of[T]()` / `align_of[T]()` → i64 常量。
    pub(crate) fn lower_layout_builtin(
        &mut self,
        name: &Symbol,
        explicit: Option<&Vec<Type>>,
        args: &Vec<Expr>,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let Some(tys) = explicit else {
            return Err(Error::Hir(format!(
                "`{}` requires an explicit type argument: `{}[T]()` (at {}:{})",
                name.as_str(), name.as_str(), span.start_line, span.start_col
            )));
        };
        let Some((ty, val)) = self.eval_layout_builtin_call(name, tys, args, span)? else {
            return Err(Error::Hir(format!("internal error: not a layout builtin (at {}:{})", span.start_line, span.start_col)));
        };
        Ok(SConst { val, ty }.into())
    }

    /// 运行期 lowering：`offset_of[T](field)` → i64 常量。
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
        let Some((ty, val)) = self.eval_layout_builtin_call(&Symbol::intern("offset_of"), tys, args, span)? else {
            return Err(Error::Hir(format!("internal error: not a layout builtin (at {}:{})", span.start_line, span.start_col)));
        };
        Ok(SConst { val, ty }.into())
    }
}
