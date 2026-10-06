use super::*;

impl crate::hir::lower::Ctx {
    /// M6.2c：结构体常量求值（非泛型结构体；字段按声明顺序规范化，标量/嵌套数组/嵌套结构体）
    pub(crate) fn eval_const_struct(
        &self,
        type_name: &Symbol,
        generic_args: &[Type],
        fields: &[(Symbol, Expr)],
        span: &Span,
        env: &HashMap<Symbol, (HirType, HirLiteral)>,
        depth: usize,
    ) -> Result<(HirType, HirLiteral)> {
        if !generic_args.is_empty() {
            return Err(Error::Hir(format!(
                "generic struct constants are not supported yet (at {}:{})",
                span.start_line, span.start_col
            )));
        }
        let def = self.struct_defs.get(type_name).cloned().ok_or_else(|| Error::Hir(format!(
            "unknown struct `{}` in constant initializer (at {}:{})",
            type_name.as_str(), span.start_line, span.start_col
        )))?;
        if fields.len() != def.len() {
            return Err(Error::Hir(format!(
                "struct constant `{}` expects {} field(s), found {} (at {}:{})",
                type_name.as_str(), def.len(), fields.len(),
                span.start_line, span.start_col
            )));
        }
        let mut vals = Vec::with_capacity(def.len());
        for f in &def {
            let (fname, fty) = (f.name, &f.ty);
            let fexpr = fields.iter().find(|(n, _)| *n == fname).ok_or_else(|| Error::Hir(format!(
                "missing field `{}` in struct constant `{}` (at {}:{})",
                fname.as_str(), type_name.as_str(), span.start_line, span.start_col
            )))?;
            let (mut ft, mut fl) = self.eval_const_expr(&fexpr.1, env, depth)?;
            super::const_coerce::coerce_literal(&mut ft, &mut fl, fty, span)?;
            vals.push(fl);
        }
        Ok((HirType::Named(*type_name), HirLiteral::Struct(vals)))
    }
}
