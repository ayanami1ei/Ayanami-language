use super::*;
use crate::parser::ast::vis::Visibility;

impl crate::hir::lower::Ctx {
    pub(super) fn collect_struct_def(
        &mut self,
        vis: &Visibility,
        name: &Symbol,
        fields: &[(Symbol, Type)],
        generic_params: &[(Symbol, Vec<Symbol>)],
        is_union: bool,
        span: &Span,
        ns_prefix: &str,
    ) -> Result<()> {
        super::collect_ns::check_unique_generic_params(generic_params, span)?;
        let source_scope = ns_prefix.starts_with(crate::package::SCOPE_PREFIX);
        if source_scope && vis.is_public() && self.source_declared_types.contains(name) {
            return Ok(());
        }

        let type_name = self.scoped_type_symbol(ns_prefix, *name);
        if ns_prefix.is_empty() {
            self.source_declared_types.insert(type_name);
        }
        let module = (!ns_prefix.is_empty()).then(|| Symbol::intern(ns_prefix));
        let hir_fields: Vec<HirStructField> = fields
            .iter()
            .map(|(n, t)| HirStructField {
                name: *n,
                ty: self.ast_type_to_hir_scoped(t, module),
            })
            .collect();

        if is_union {
            self.collect_union_def(name, &hir_fields, generic_params, span, type_name)?;
        } else {
            self.struct_defs.insert(type_name, hir_fields);
            if !generic_params.is_empty() {
                self.generic_struct_params
                    .insert(type_name, generic_params.to_vec());
            }
        }

        if source_scope
            && !vis.is_public()
            && !self.source_declared_types.contains(name)
            && !Self::is_scope_shared_type(name)
        {
            self.struct_defs.remove(name);
            self.generic_struct_params.remove(name);
            self.type_ifaces.remove(name);
        }
        Ok(())
    }

    fn collect_union_def(
        &mut self,
        name: &Symbol,
        fields: &[HirStructField],
        generic_params: &[(Symbol, Vec<Symbol>)],
        span: &Span,
        type_name: Symbol,
    ) -> Result<()> {
        if !generic_params.is_empty() {
            return Err(Error::Hir(format!(
                "generic unions are not supported yet (`{}`) (at {}:{})",
                name.as_str(),
                span.start_line,
                span.start_col
            )));
        }
        for field in fields {
            if !self.union_field_ok(&field.ty) {
                return Err(Error::Hir(format!(
                    "union `{}` field `{}` has type `{}` which may own heap data; union fields must be drop-free (at {}:{})",
                    name.as_str(),
                    field.name.as_str(),
                    hir_type_display(&field.ty),
                    span.start_line,
                    span.start_col
                )));
            }
        }
        let mut def = vec![HirStructField {
            name: Symbol::intern("__union"),
            ty: fields.first().map(|f| f.ty.clone()).unwrap_or(HirType::Int),
        }];
        def.extend_from_slice(fields);
        self.struct_defs.insert(type_name, def);
        Ok(())
    }
}
