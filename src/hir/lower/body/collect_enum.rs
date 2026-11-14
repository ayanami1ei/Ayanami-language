use crate::parser::ast::stmt::EnumVariant;

use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn collect_enum_def(
        &mut self,
        name: &Symbol,
        variants: &Vec<EnumVariant>,
        generic_params: &Vec<(Symbol, Vec<Symbol>)>,
        _ns_prefix: &str,
    ) -> Result<()> {
            for variant in variants {
                let var_struct_name = Symbol::intern(&format!("{}_{}", name, variant.name));
                let hir_fields: Vec<HirStructField> = match &variant.fields {
                    crate::parser::ast::stmt::EnumFields::Named(fields) => {
                        fields.iter()
                            .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                            .collect()
                    }
                    crate::parser::ast::stmt::EnumFields::Tuple(tys) => {
                        tys.iter().enumerate()
                            .map(|(i, t)| HirStructField { name: Symbol::intern(&format!("_{}", i)), ty: ast_type_to_hir(t, &self.interfaces) })
                            .collect()
                    }
                    crate::parser::ast::stmt::EnumFields::None => vec![],
                };
            self.struct_defs.insert(var_struct_name, hir_fields);
            if !generic_params.is_empty() {
                self.generic_struct_params.insert(var_struct_name, generic_params.clone());
            }
        }
        // Register enum struct: { tag: int, data_variant1, data_variant2, ... }
        let mut enum_fields = vec![HirStructField { name: Symbol::intern("_tag"), ty: HirType::Int }];
        for variant in variants {
            let vsn = Symbol::intern(&format!("{}_{}", name, variant.name));
            enum_fields.push(HirStructField { name: Symbol::intern(&format!("_data_{}", variant.name)), ty: HirType::Named(vsn) });
        }
        // N1/N2：声明期改写（泛型枚举不装箱——载荷类型尚未实例化）
        let enum_fields = self.apply_boxing_rewrite(enum_fields, !generic_params.is_empty());
        let enum_fields = self.apply_niche_rewrite(enum_fields);
        self.struct_defs.insert(*name, enum_fields);
        if !generic_params.is_empty() {
            self.generic_struct_params.insert(*name, generic_params.clone());
        }
        Ok(())
    }

    /// 枚举的程序项（变体结构体 + 枚举结构体；N1 niche 改写：docs/enum-layout.md §3.1）。
    pub(crate) fn build_enum_items(&mut self, name: &Symbol, variants: &Vec<EnumVariant>) -> Vec<HirItem> {
        let mut items = Vec::new();
        for variant in variants {
            let var_struct_name = Symbol::intern(&format!("{}_{}", name, variant.name));
            let hir_fields: Vec<HirStructField> = match &variant.fields {
                crate::parser::ast::stmt::EnumFields::Named(fields) => fields.iter()
                    .map(|(n, t)| HirStructField { name: *n, ty: ast_type_to_hir(t, &self.interfaces) })
                    .collect(),
                crate::parser::ast::stmt::EnumFields::Tuple(tys) => tys.iter().enumerate()
                    .map(|(i, t)| HirStructField { name: Symbol::intern(&format!("_{}", i)), ty: ast_type_to_hir(t, &self.interfaces) })
                    .collect(),
                crate::parser::ast::stmt::EnumFields::None => vec![],
            };
            items.push(HirItem::StructDef(HirStructDef { name: var_struct_name, fields: hir_fields }));
        }
        let mut enum_fields = vec![HirStructField { name: Symbol::intern("_tag"), ty: HirType::Int }];
        for variant in variants {
            let vsn = Symbol::intern(&format!("{}_{}", name, variant.name));
            enum_fields.push(HirStructField { name: Symbol::intern(&format!("_data_{}", variant.name)), ty: HirType::Named(vsn) });
        }
        let is_generic = self.generic_struct_params.get(name).map(|g| !g.is_empty()).unwrap_or(false);
        let enum_fields = self.apply_boxing_rewrite(enum_fields, is_generic);
        let enum_fields = self.apply_niche_rewrite(enum_fields);
        items.push(HirItem::StructDef(HirStructDef { name: *name, fields: enum_fields }));
        items
    }
}
