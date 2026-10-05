use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_field_access(&mut self, object: &Box<Expr>, field: &Symbol, expr_span: &Span) -> Result<HirNodeBox> {
        let hir_object = self.lower_expr(object)?;
        let object_ty = hir_object.expr_type();
        let field_index = self.find_field_index(&object_ty, field, expr_span)?;
        let field_ty = self.find_field_type(&object_ty, field, expr_span)?;
        Ok(SField {
            object: hir_object,
            field: *field,
            field_index,
            ty: field_ty,
        }.into())
    }

    pub(crate) fn lower_struct_literal(&mut self, type_name: &Symbol, generic_args: &Vec<Type>, fields: &Vec<(Symbol, Expr)>) -> Result<HirNodeBox> {
        // Handle generic struct instantiation
        let concrete_name = if !generic_args.is_empty() {
            let args_str: Vec<String> = generic_args.iter()
                .map(|a| type_to_string_generic(a, &self.interfaces))
                .collect();
            let encoded = format!("{}<{}>", type_name, args_str.join(","));
            let name_sym = Symbol::intern(&encoded);
            // Monomorphize: create concrete struct def if not exists
            if !self.struct_defs.contains_key(&name_sym) {
                if let Some(generic_fields) = self.struct_defs.get(type_name) {
                    // Build substitution map: T → concrete type
                    let mut generic_params = self.collected_generic_params(type_name);
                    // 若 generic_struct_params 未从 .lcl 合并，则从字段类型推断 GP 名称
                    if generic_params.is_empty() && !generic_args.is_empty() {
                        generic_params = generic_args.iter().enumerate()
                            .map(|(i, _)| (Symbol::intern(&format!("_G{}", i)), None))
                            .collect();
                        // 尝试从字段类型中提取实际 GP 名称（单字母大写名）
                        if let Some(fields) = self.struct_defs.get(type_name) {
                            for field in fields {
                                if let HirType::Named(n) = strip_ownership_ref(&field.ty) {
                                    let s = n.as_str();
                                    if s.len() == 1 && s.chars().all(|c| c.is_uppercase()) {
                                        generic_params = vec![(*n, None)];
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    let mut subst: HashMap<Symbol, HirType> = HashMap::new();
                    for ((gp_name, _), concrete_ty) in generic_params.iter().zip(generic_args.iter()) {
                        let hir_ty = ast_type_to_hir(concrete_ty, &self.interfaces);
                        subst.insert(*gp_name, hir_ty);
                    }
                    // Substitute field types
                    let concrete_fields: Vec<HirStructField> = generic_fields.iter()
                        .map(|f| {
                            let new_ty = substitute_hir_type(&f.ty, &subst);
                            if &f.ty != &new_ty {
                            }
                            HirStructField { name: f.name, ty: new_ty }
                        })
                        .collect();
                    self.struct_defs.insert(name_sym, concrete_fields);
                }
            }
            name_sym
        } else {
            *type_name
        };
        let struct_ty = HirType::Named(concrete_name);
        let mut hir_fields = Vec::new();
        // Pre-compute field types from struct def for null type coercion
        let field_tys: HashMap<Symbol, HirType> = self.struct_defs.get(&concrete_name)
            .map(|fields| fields.iter().map(|f| (f.name, f.ty.clone())).collect())
            .unwrap_or_default();
        for (name, expr) in fields {
            let mut hir_val = self.lower_expr(expr)?;
            // Coerce null literal to the correct pointer type
            if let Some(HirLiteral::Int(0)) = hir_val.as_const() {
                if let Some(field_ty) = field_tys.get(name) {
                    if matches!(field_ty, HirType::Unique(_)) {
                        hir_val = SConst { val: HirLiteral::Int(0), ty: field_ty.clone() }.into();
                    }
                }
            }
            // 字段类型隐式数值转换
            if let Some(field_ty) = field_tys.get(name) {
                hir_val = coerce_expr(hir_val, field_ty, &expr.span())?;
            }
            hir_fields.push((*name, implicit_move(hir_val)));
        }
        Ok(SStruct {
            type_name: concrete_name,
            fields: hir_fields,
            ty: struct_ty,
        }.into())
    }

    pub(crate) fn lower_array_literal(&mut self, elems: &Vec<Expr>, span: &Span) -> Result<HirNodeBox> {
        let _ = span;
        let mut hir_elems = Vec::new();
        for e in elems {
            hir_elems.push(implicit_move(self.lower_expr(e)?));
        }
        let elem_ty = if !hir_elems.is_empty() {
            strip_ownership(expr_type(&hir_elems[0]))
        } else {
            HirType::Int
        };
        // `[a, b, ...]` 拥有堆数组
        let ty = if !hir_elems.is_empty() {
            HirType::Unique(Box::new(HirType::ArraySized(Box::new(elem_ty.clone()), hir_elems.len())))
        } else {
            HirType::Unique(Box::new(HirType::Array(Box::new(elem_ty))))
        };
        Ok(SArrLit { elems: hir_elems, ty }.into())
    }

    pub(crate) fn lower_index(&mut self, object: &Box<Expr>, index: &Box<Expr>, span: &Span) -> Result<HirNodeBox> {
        let hir_object = self.lower_expr(object)?;
        let hir_index = auto_deref(self.lower_expr(index)?);
        let object_ty = expr_type(&hir_object);
        let inner_ty = strip_ownership(object_ty.clone());
        // Virtual dispatch through interface
        if let HirType::FatPtr { name: iface, .. } = &inner_ty {
            let iface_name = *iface;
            if !self.interfaces.contains_key(&iface_name) {
                self.ensure_specialized_interface(&iface_name)?;
            }
            let iface_reg = self.interfaces.get(&iface_name)
                .ok_or_else(|| Error::Hir(format!("unknown interface `{}` used as type (at {}:{})", iface_name, span.start_line, span.start_col)))?;
            let method_idx = iface_reg.methods.iter()
                .position(|m| m.name == Symbol::intern("index"))
                .ok_or_else(|| Error::Hir(format!("interface `{}` has no method `index` (at {}:{})", iface_name, span.start_line, span.start_col)))?;
            let ret_ty = iface_reg.methods[method_idx].return_type.clone();
            if Self::contains_self_type(&ret_ty)
                || iface_reg.methods[method_idx].params.iter().any(|(_, t)| Self::contains_self_type(t))
            {
                return Err(Error::Hir(format!(
                    "interface `{}` method `{}` uses Self; dynamic dispatch (`ref {}`) is not supported — use a generic bound like `[T: {}]` (at {}:{})",
                    iface_name.as_str(), Symbol::intern("index").as_str(), iface_name.as_str(), iface_name.as_str(),
                    span.start_line, span.start_col
                )));
            }
            return Ok(SVCall {
                receiver: hir_object,
                interface: iface_name,
                method_index: method_idx,
                args: vec![hir_index],
                concrete_type: iface_name,
                ty: ret_ty,
            }.into());
        }
        let index_ty = hir_index.expr_type();
        let index_fn = Symbol::intern("index");
        let index_lit = is_numeric_literal(&hir_index);
        // 解析顺序：精确 → 字面量适配 → 常见整数索引形参（int/usize）→ 泛型特化
        let mut fn_id = self.resolve_fn_call(&index_fn, &[object_ty.clone(), index_ty.clone()])
            .or_else(|| self.resolve_fn_call_literals(&index_fn, &[object_ty.clone(), index_ty.clone()], &[false, index_lit]));
        if fn_id.is_none() {
            for t in [HirType::Int, HirType::IntN { bits: 64, signed: false }] {
                if let Some(id) = self.resolve_fn_call(&index_fn, &[object_ty.clone(), t]) {
                    fn_id = Some(id);
                    break;
                }
            }
        }
        let fn_id = fn_id.or_else(|| self.specialize_generic_call(&index_fn, &[object_ty.clone(), index_ty.clone()], span).ok());
        if let Some(fn_id) = fn_id {
            let ret_ty = self.fns[fn_id.0].return_type.clone();
            let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter().map(|(_, t)| t.clone()).collect();
            let hir_index = if param_tys.len() > 1 { coerce_index(hir_index, &param_tys[1], span)? } else { hir_index };
            let mut args: Vec<HirNodeBox> = vec![hir_object, hir_index].into_iter().enumerate().map(|(i, arg)| {
                if i >= param_tys.len() { return arg; }
                wrap_arg_for_param(arg, &param_tys[i])
            }).collect();
            self.append_caller_args(fn_id, &mut args, span);
            return Ok(SCall { fn_id, args, ty: ret_ty }.into());
        }
        // 内置数组索引：统一转 i64 寻址
        let elem_ty = match &inner_ty {
            HirType::Array(inner) | HirType::ArraySized(inner, _) => *inner.clone(),
            _ => return Err(Error::Hir(format!("index on non-array type at {}:{}", span.start_line, span.start_col))),
        };
        let hir_index = coerce_index(hir_index, &HirType::Int, span)?;
        Ok(SIdx {
            object: hir_object,
            index: hir_index,
            ty: elem_ty,
        }.into())
    }
}
