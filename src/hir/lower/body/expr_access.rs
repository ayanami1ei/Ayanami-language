use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_field_access(&mut self, object: &Box<Expr>, field: &Symbol, expr_span: &Span) -> Result<HirNodeBox> {
        let hir_object = self.lower_expr(object)?;
        let object_ty = hir_object.expr_type();
        // M4：先确保泛型实例（如 Option<Ptr[Tree]>）已单态化，字段查找才能取到特化类型
        self.instantiate_type(&object_ty)?;
        let field_index = self.find_field_index(&object_ty, field, expr_span)?;
        let field_ty = self.find_field_type(&object_ty, field, expr_span)?;
        // P3b：union 字段读取需 unsafe（初始化/写入按 Rust 语义无需）
        if let HirType::Named(n) = strip_ownership_ref(&object_ty) {
            if self.is_union_type(n) {
                self.require_unsafe(&format!("read of union field `{}`", field.as_str()), expr_span)?;
            }
        }
        Ok(SField {
            object: hir_object,
            field: *field,
            field_index,
            ty: field_ty,
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
        // 内置数组（含 `ref [T]` / `ref mut [T]` 自动解引用）：优先于方法解析，
        // 否则导入 std 后可能误配 `String.index` 等同名方法（读索引此前直接报
        // "index on non-array type"）。
        let deref_ty = strip_ownership(match &inner_ty {
            HirType::Ref(inner, _) => inner.as_ref().clone(),
            other => other.clone(),
        });
        if let HirType::Array(elem) | HirType::ArraySized(elem, _) = &deref_ty {
            let elem_ty = (**elem).clone();
            let hir_index = coerce_index(hir_index, &HirType::Int, span)?;
            // P3b（Rust 方式）：`[T; n]` 索引默认越界检查（panic 101）；
            // 动态 `[T]` 无长度信息，保持未检查原语
            // P3b（Rust 方式）：越界检查（`[T; n]` 静态 / 动态字段兄弟边界；
            // 裸 `[T]` 无边界信息保持未检查原语）
            let hir_index = self.check_array_index(&hir_object, hir_index, span);
            return Ok(SIdx {
                object: hir_object,
                index: hir_index,
                ty: elem_ty,
            }.into());
        }
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
