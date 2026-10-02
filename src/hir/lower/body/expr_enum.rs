use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_enum_construct(&mut self, enum_name: &Symbol, variant_name: &Symbol, tuple_args: &Vec<Expr>, named_args: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox> {
        if !named_args.is_empty() {
            return Err(Error::Hir(format!("named fields in enum construct not yet supported at {}:{}", span.start_line, span.start_col)));
        }
        let tag = self.struct_defs.get(enum_name).map_or(0i64, |fields| {
            let target = format!("_data_{}", variant_name);
            for (i, f) in fields.iter().enumerate() {
                if f.name.as_str() == target { return (i - 1) as i64; }
            }
            0i64
        });
        let hir_args: Vec<HirNodeBox> = tuple_args
            .iter()
            .map(|e| self.lower_expr(e).map(implicit_move))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let data_field = Symbol::intern(&format!("_data_{}", variant_name));
        let var_struct_name = Symbol::intern(&format!("{}_{}", enum_name, variant_name));
        let data_ty = HirType::Named(var_struct_name);
        let var_fields: Vec<(Symbol, HirNodeBox)> = hir_args.into_iter().enumerate()
            .map(|(i, e)| (Symbol::intern(&format!("_{}", i)), e))
            .collect();
        let var_literal: HirNodeBox = SStruct {
            type_name: var_struct_name,
            fields: var_fields,
            ty: data_ty,
        }.into();
        let fields = if let Some(enum_fields) = self.struct_defs.get(enum_name) {
            let mut all_fields: Vec<(Symbol, HirNodeBox)> = enum_fields.iter()
                .map(|f| (f.name, SConst { val: HirLiteral::Int(0), ty: f.ty.clone() }.into()))
                .collect();
            all_fields[0] = (Symbol::intern("_tag"), SConst { val: HirLiteral::Int(tag), ty: HirType::Int }.into());
            if let Some(pos) = enum_fields.iter().position(|f| f.name == data_field) {
                all_fields[pos] = (data_field, var_literal);
            }
            all_fields
        } else {
            vec![
                (Symbol::intern("_tag"), SConst { val: HirLiteral::Int(tag), ty: HirType::Int }.into()),
                (data_field, var_literal),
            ]
        };
        let ty = HirType::Named(*enum_name);
        Ok(SStruct { type_name: *enum_name, fields, ty }.into())
    }
}
