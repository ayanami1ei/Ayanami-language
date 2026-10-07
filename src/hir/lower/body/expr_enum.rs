use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_enum_construct(&mut self, enum_name: &Symbol, variant_name: &Symbol, tuple_args: &Vec<Expr>, named_args: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox> {
        if !named_args.is_empty() {
            return Err(Error::Hir(format!("named fields in enum construct not yet supported at {}:{}", span.start_line, span.start_col)));
        }
        // #150：未知变体必须报错（此前静默回退 tag 0）
        let tag = self.struct_defs.get(enum_name).and_then(|fields| {
            let target = format!("_data_{}", variant_name);
            fields.iter().position(|f| f.name.as_str() == target).map(|i| (i - 1) as i64)
        });
        let Some(tag) = tag else {
            return Err(Error::Hir(format!(
                "enum `{}` has no variant `{}` (at {}:{})",
                enum_name.as_str(), variant_name.as_str(), span.start_line, span.start_col
            )));
        };
        // 载荷个数检查（变体结构体字段数）
        let base_variant = Symbol::intern(&format!("{}_{}", enum_name.as_str(), variant_name.as_str()));
        if let Some(vf) = self.struct_defs.get(&base_variant) {
            if vf.len() != tuple_args.len() {
                return Err(Error::Hir(format!(
                    "variant `{}::{}` expects {} field(s), found {} (at {}:{})",
                    enum_name.as_str(), variant_name.as_str(), vf.len(), tuple_args.len(),
                    span.start_line, span.start_col
                )));
            }
        }
        let hir_args: Vec<HirNodeBox> = tuple_args
            .iter()
            .map(|e| self.lower_expr(e).map(implicit_move))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
        let data_field = Symbol::intern(&format!("_data_{}", variant_name));
        // #120：泛型枚举构造单态化（从载荷推断类型实参；E::A(1) → E<int>）
        let (enum_ty_name, var_struct_name) = self.monomorphize_enum_construct(enum_name, variant_name, &arg_types);
        let data_ty = HirType::Named(var_struct_name);
        let var_fields: Vec<(Symbol, HirNodeBox)> = hir_args.into_iter().enumerate()
            .map(|(i, e)| (Symbol::intern(&format!("_{}", i)), e))
            .collect();
        let var_literal: HirNodeBox = SStruct {
            type_name: var_struct_name,
            fields: var_fields,
            ty: data_ty,
        }.into();
        let fields = if let Some(enum_fields) = self.struct_defs.get(&enum_ty_name) {
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
        let ty = HirType::Named(enum_ty_name);
        Ok(SStruct { type_name: enum_ty_name, fields, ty }.into())
    }

    /// #120：从构造载荷推断枚举类型实参并单态化（枚举布局 + 全部变体 struct）。
    /// 无法推断全部泛型参数时回退基名（如 `Option::None()`，需期望类型上下文）。
    fn monomorphize_enum_construct(
        &mut self,
        enum_name: &Symbol,
        variant_name: &Symbol,
        arg_types: &[HirType],
    ) -> (Symbol, Symbol) {
        let base_variant = Symbol::intern(&format!("{}_{}", enum_name, variant_name));
        let gp = match self.generic_struct_params.get(enum_name) {
            Some(g) if !g.is_empty() => g.clone(),
            _ => return (*enum_name, base_variant),
        };
        // 从变体字段类型与实参推断（仅直接泛型参数形参，如 A(T)）
        let mut subst: HashMap<Symbol, HirType> = HashMap::new();
        if let Some(vf) = self.struct_defs.get(&base_variant) {
            for (i, at) in arg_types.iter().enumerate() {
                if let Some(f) = vf.get(i) {
                    if let HirType::Named(p) = strip_ownership_ref(&f.ty) {
                        if gp.iter().any(|(n, _)| n == p) {
                            subst.insert(*p, at.clone());
                        }
                    }
                }
            }
        }
        if !gp.iter().all(|(n, _)| subst.contains_key(n)) {
            return (*enum_name, base_variant);
        }
        let args_str = gp.iter()
            .map(|(n, _)| hir_type_display(subst.get(n).unwrap()))
            .collect::<Vec<_>>()
            .join(",");
        let concrete_enum = Symbol::intern(&format!("{}<{}>", enum_name, args_str));
        if !self.struct_defs.contains_key(&concrete_enum) {
            // 枚举布局引用的全部变体 struct
            let variant_bases: Vec<Symbol> = self.struct_defs.get(enum_name)
                .map(|fields| fields.iter()
                    .filter(|f| f.name.as_str().starts_with("_data_"))
                    .filter_map(|f| match &f.ty { HirType::Named(n) => Some(*n), _ => None })
                    .collect())
                .unwrap_or_default();
            for vb in &variant_bases {
                let cname = Symbol::intern(&format!("{}<{}>", vb, args_str));
                if !self.struct_defs.contains_key(&cname) {
                    if let Some(fields) = self.struct_defs.get(vb) {
                        let cfields: Vec<HirStructField> = fields.iter()
                            .map(|f| HirStructField { name: f.name, ty: substitute_hir_type(&f.ty, &subst) })
                            .collect();
                        self.struct_defs.insert(cname, cfields);
                    }
                }
            }
            if let Some(fields) = self.struct_defs.get(enum_name) {
                let cfields: Vec<HirStructField> = fields.iter().map(|f| {
                    let ty = match &f.ty {
                        HirType::Named(n) if variant_bases.contains(n) =>
                            HirType::Named(Symbol::intern(&format!("{}<{}>", n, args_str))),
                        other => substitute_hir_type(other, &subst),
                    };
                    HirStructField { name: f.name, ty }
                }).collect();
                self.struct_defs.insert(concrete_enum, cfields);
            }
        }
        let concrete_variant = Symbol::intern(&format!("{}<{}>", base_variant, args_str));
        (concrete_enum, concrete_variant)
    }
}
