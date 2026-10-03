//! 泛型结构体/枚举的最小单态化（见 docs/generic-monomorphization.md）。

use super::*;

/// 泛型单态化：把变体结构体名 `Base_Variant` 改写为 `Base_Variant<args>`。
fn rewrite_variant_name(ty: &HirType, base: Symbol, suffix: &str) -> HirType {
    if let HirType::Named(n) = ty {
        let ns = n.as_str();
        let bs = base.as_str();
        if ns.starts_with(&format!("{}_", bs)) && !ns.contains('<') {
            return HirType::Named(Symbol::intern(&format!("{}{}", ns, suffix)));
        }
    }
    ty.clone()
}


impl crate::hir::lower::Ctx {
    /// 泛型单态化：按需实例化类型中出现的 `Named("X<args>")`。
    pub fn instantiate_type(&mut self, ty: &HirType) -> Result<()> {
        match ty {
            HirType::Named(n) => self.instantiate_named(*n),
            HirType::Unique(t) | HirType::Array(t) => self.instantiate_type(t),
            HirType::ArraySized(t, _) => self.instantiate_type(t),
            HirType::Ref(t, _) => self.instantiate_type(t),
            HirType::FnPtr(ps, r) => {
                for p in ps { self.instantiate_type(p)?; }
                self.instantiate_type(r)
            }
            _ => Ok(()),
        }
    }

    /// 实例化 `Named("Base<args>")`：替换字段类型、改写变体名、递归实例化嵌套。
    pub fn instantiate_named(&mut self, name: Symbol) -> Result<()> {
        if self.struct_defs.contains_key(&name) {
            return Ok(());
        }
        let s = name.as_str();
        let Some(pos) = s.find('<') else { return Ok(()); };
        let base = strip_generic_name(&name);
        if base == name {
            return Ok(());
        }
        let Some(base_fields) = self.struct_defs.get(&base).cloned() else {
            return Ok(());
        };
        let subst = self.build_generic_subst(&name, &base);
        let suffix = &s[pos..]; // "<int,int>"
        let mut new_fields: Vec<(Symbol, HirType)> = Vec::new();
        for f in &base_fields {
            let rewritten = rewrite_variant_name(&f.ty, base, suffix);
            new_fields.push((f.name, substitute_hir_type(&rewritten, &subst)));
        }
        self.struct_defs.insert(name, new_fields.iter()
            .map(|(n, t)| HirStructField { name: *n, ty: t.clone() })
            .collect());
        self.generic_struct_params.insert(name, Vec::new());

        // 实例化变体结构体（字段即枚举参数 → 同一 subst）
        for (_, fty) in &new_fields {
            if let HirType::Named(vn) = fty {
                let vn = *vn;
                if !self.struct_defs.contains_key(&vn) {
                    if let Some(vfields) = self.struct_defs.get(&vn).cloned() {
                        let substituted: Vec<HirStructField> = vfields.iter()
                            .map(|f| HirStructField { name: f.name, ty: substitute_hir_type(&f.ty, &subst) })
                            .collect();
                        self.struct_defs.insert(vn, substituted);
                        self.generic_struct_params.insert(vn, Vec::new());
                    }
                }
                self.instantiate_type(fty)?;
            }
        }
        Ok(())
    }

    /// 泛型单态化：按形参类型批量重写调用实参中的枚举构造。
    pub fn adapt_enum_args(
        &mut self,
        args: Vec<HirNodeBox>,
        param_tys: &[HirType],
    ) -> Result<Vec<HirNodeBox>> {
        let mut out = Vec::with_capacity(args.len());
        for (i, a) in args.into_iter().enumerate() {
            let a = if i < param_tys.len() {
                self.instantiate_enum_value(a, &param_tys[i])?
            } else {
                a
            };
            out.push(a);
        }
        Ok(out)
    }

    /// 泛型单态化：把枚举构造的基名 `SStruct` 重写为期望的实例化名。
    pub fn instantiate_enum_value(&mut self, node: HirNodeBox, expected: &HirType) -> Result<HirNodeBox> {
        let HirType::Named(ename) = expected else { return Ok(node); };
        let es = ename.as_str();
        let Some(pos) = es.find('<') else { return Ok(node); };
        let suffix = es[pos..].to_string();
        let Some(mut st) = node.as_struct_cloned() else { return Ok(node); };
        if st.type_name == *ename {
            return Ok(node);
        }
        st.type_name = *ename;
        st.ty = expected.clone();
        let mut new_fields = Vec::new();
        for (fname, fval) in st.fields {
            let fty = self.find_field_type(expected, &fname, &Span::default()).ok();
            let nv: HirNodeBox = if let Some(mut inner) = fval.as_struct_cloned() {
                let vn = inner.type_name.as_str();
                let inst = Symbol::intern(&format!("{}{}", vn, suffix));
                inner.type_name = inst;
                inner.ty = HirType::Named(inst);
                inner.into()
            } else if let Some(t) = &fty {
                fval.with_type(t.clone()).unwrap_or(fval)
            } else {
                fval
            };
            new_fields.push((fname, nv));
        }
        st.fields = new_fields;
        Ok(st.into())
    }

}
