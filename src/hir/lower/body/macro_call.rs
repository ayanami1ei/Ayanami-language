//! A5c-2：函数宏调用 `#name(args)` 的降级（编译期展开为表达式）。

use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_macro_call(&mut self, name: &Symbol, args: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        // 宏所在包（裸名可能被多个包导出 → 要求全限定）
        let lcls = self.imported_macro_lcls.get(name).cloned().unwrap_or_default();
        if lcls.is_empty() {
            return Err(Error::Hir(format!(
                "unknown function macro `#{}` (at {}:{})",
                name, span.start_line, span.start_col
            )));
        }
        if lcls.len() > 1 {
            return Err(Error::Hir(format!(
                "ambiguous function macro `#{}`: exported by multiple packages; use `#pkg::{}` (at {}:{})",
                name, name, span.start_line, span.start_col
            )));
        }
        let lcl = &lcls[0];
        let full = name.as_str();
        let macro_name = full.rsplit('.').next().unwrap_or(&full).to_string();

        // 实参以源码文本传给宏（保留原始形态）
        let arg_src: Vec<String> = args.iter().map(crate::formatter::format_expr).collect();

        if self.macro_depth >= 32 {
            return Err(Error::Hir(format!(
                "function macro recursion limit reached at `#{}` (at {}:{})",
                name, span.start_line, span.start_col
            )));
        }
        self.macro_depth += 1;
        let expanded = crate::compiler::macro_expand::invoke_macro_expr(
            lcl,
            &macro_name,
            &arg_src,
            span.start_line,
            span.start_col,
            &crate::hir::lower::source_path(),
        );
        self.macro_depth -= 1;

        let text = expanded?;
        let expr = crate::parser::parse_expression(&text).map_err(|e| {
            Error::Hir(format!(
                "macro `#{}` produced an invalid expression: {} (at {}:{})",
                name, e, span.start_line, span.start_col
            ))
        })?;
        self.lower_expr(&expr)
    }
}
