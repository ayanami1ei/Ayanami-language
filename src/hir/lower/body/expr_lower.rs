use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> Result<HirNodeBox> {
        match expr {
            Expr::Literal(lit) => self.lower_literal(lit),
            Expr::Suffixed { lit, suffix, span } => self.lower_suffixed(lit, suffix, span),
            Expr::Ident(name, span) => {
                // `Enum::Variant`（无参数）被解析器合并为 `Enum.Variant`
                if let Some((enum_name, variant_name)) = name.as_str().split_once('.') {
                    let enum_sym = Symbol::intern(enum_name);
                    if self.is_enum_type(&enum_sym) {
                        return self.lower_enum_construct(
                            &enum_sym,
                            &Symbol::intern(variant_name),
                            &vec![],
                            &vec![],
                            span,
                        );
                    }
                }
                // M2：闭包捕获变量 → env 字段访问
                if let Some(env) = &self.lambda_env {
                    if let Some((idx, cty)) = env.lookup(name) {
                        let base: HirNodeBox = SVar { var: env.var, ty: env.ty.clone() }.into();
                        return Ok(SField { object: base, field: *name, field_index: idx, ty: cty }.into());
                    }
                }
                if let Some((var_id, ty, _)) = self.lookup_var(name) {
                    return Ok(SVar { var: var_id, ty }.into());
                }
                // M6.1：编译期常量（局部变量优先，const 次之）
                if let Some((ty, val)) = self.consts.get(name) {
                    return Ok(SConst { val: val.clone(), ty: ty.clone() }.into());
                }
                // M6.2：全局变量读取（SGlobal 取址 + SDeref 载入）
                if let Some(st) = self.statics.get(name).cloned() {
                    let addr: HirNodeBox = SGlobal { name: st.name, ty: st.ty.clone(), mutable: st.is_mut }.into();
                    return Ok(SDeref { expr: addr, ty: st.ty.clone() }.into());
                }
                if let Some(candidates) = self.fn_map.get(name) {
                    if let Some(&first) = candidates.first() {
                        let (params, ret) = {
                            let sig = &self.fns[first.0];
                            if sig.params.is_empty() {
                                return Err(Error::Hir(format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col)));
                            }
                            (sig.params.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>(), sig.return_type.clone())
                        };
                        // M2 统一：函数名作值 → 静态闭包（Copy、零分配）
                        let fnptr_ty = HirType::FnPtr(params.clone(), Box::new(ret.clone()));
                        let raw: HirNodeBox = SFnPtr { fn_id: first, ty: fnptr_ty }.into();
                        return Ok(self.make_static_closure(raw, &params, &ret));
                    }
                }
                Err(Error::Hir(format!("undefined variable `{}` at {}:{}", name, span.start_line, span.start_col)))
            }
            Expr::Binary { op, lhs, rhs, span } => self.lower_binary(op, lhs, rhs, span),
            Expr::Cast { expr, ty, span } => self.lower_cast(expr, ty, span),
            Expr::Unary { op, arg, .. } => self.lower_unary(op, arg),
            Expr::FnCall { name, args, generic_args, span } => self.lower_fn_call(name, args, if generic_args.is_empty() { None } else { Some(generic_args) }, span),
            Expr::CallExpr { target, args, span } => self.lower_call_expr(target, args, span),
            Expr::TryOp(inner, span) => self.lower_try_op(inner, span),
            Expr::Match { value, arms, span } => self.lower_match_expr(value, arms, span),
            Expr::If { cond, then_block, elifs, else_block, span } => self.lower_if_expr(cond, then_block, elifs, else_block, span),
            Expr::MacroCall { name, args, span } => self.lower_macro_call(name, args, span),
            Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, span } => self.lower_enum_construct(enum_name, variant_name, tuple_args, named_args, span),
            Expr::MethodCall { object, method, args, span } => self.lower_method_call(object, method, args, span),
            Expr::Move(inner, _) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                Ok(SMove { expr: hir_inner, ty }.into())
            }
            Expr::Clone(inner, span) => {
                let hir_inner = self.lower_expr(inner)?;
                let ty = hir_inner.expr_type();
                // Copy 类型直接复制；拥有堆数据的类型没有通用深拷贝（数组运行时不带长度），
                // 引导使用各类型的 `.copy()` 方法。
                if ty.is_copy() {
                    Ok(hir_inner)
                } else {
                    Err(Error::Hir(format!(
                        "`clone` is not implemented for owned value of type {}; use `.copy()` instead (at {}:{})",
                        crate::hir::display::display_type(&ty),
                        span.start_line,
                        span.start_col
                    )))
                }
            }
            Expr::ToUnique(inner, _) => {
                self.allow_bare_array = true;
                let hir_inner = self.lower_expr(inner)?;
                self.allow_bare_array = false;
                let inner_ty = hir_inner.expr_type();
                // Copy 类型的 unique 无意义（装箱会被复制语义抵消）：零开销传递
                if inner_ty.is_copy() {
                    return Ok(hir_inner);
                }
                let ty = HirType::Unique(Box::new(strip_ownership(inner_ty)));
                Ok(SToUnique { expr: hir_inner, ty }.into())
            }
            Expr::FieldAccess { object, field, span: expr_span } => self.lower_field_access(object, field, expr_span),
            Expr::StructLiteral { type_name, generic_args, fields, .. } => self.lower_struct_literal(type_name, generic_args, fields),
            Expr::ArrayLiteral(elems, span) => self.lower_array_literal(elems, span),
            // M6.2c：`[v; n]` 运行期按字面量计数展开（常量上下文由 const_eval 处理）
            Expr::ArrayRepeat { value, count, span } => {
                let n = match count.as_ref() {
                    Expr::Literal(crate::parser::ast::Literal::Int(v, _)) => *v as usize,
                    _ => return Err(Error::Hir(format!(
                        "repeat literal count must be an integer literal at {}:{}",
                        span.start_line, span.start_col
                    ))),
                };
                if n > 1024 {
                    return Err(Error::Hir(format!(
                        "repeat literal too large ({}); use a fill loop instead (at {}:{})",
                        n, span.start_line, span.start_col
                    )));
                }
                let elems: Vec<Expr> = (0..n).map(|_| (**value).clone()).collect();
                self.lower_array_literal(&elems, span)
            }
            Expr::Index { object, index, span } => self.lower_index(object, index, span),
            Expr::Null(_) => {
                Ok(SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into())
            }
            Expr::Ref(inner, mutable, span) => {
                // M6.2：`ref NAME`（static）→ 全局地址本身
                if let Expr::Ident(name, _) = inner.as_ref() {
                    if let Some(st) = self.statics.get(name).cloned() {
                        if *mutable && !st.is_mut {
                            return Err(Error::Hir(format!(
                                "cannot take a mutable reference to immutable static `{}` (at {}:{})",
                                name.as_str(), span.start_line, span.start_col
                            )));
                        }
                        let addr: HirNodeBox = SGlobal { name: st.name, ty: st.ty.clone(), mutable: *mutable }.into();
                        let ty = HirType::Ref(Box::new(st.ty.clone()), *mutable);
                        return Ok(SRef { expr: addr, mutable: *mutable, ty }.into());
                    }
                }
                let hir_inner = self.lower_expr(inner)?;
                let inner_ty = hir_inner.expr_type();
                // 已是引用：`ref`/`ref mut` 为再借用，直接传递（避免双重间接；
                // 递归 `f(ref mut a)` 传 `ref mut [T]` 参数即此路径）
                if let HirType::Ref(_, inner_mut) = &inner_ty {
                    if *mutable && !*inner_mut {
                        return Err(Error::Hir(format!(
                            "cannot take a mutable reference through an immutable reference (at {}:{})",
                            span.start_line, span.start_col
                        )));
                    }
                    return Ok(hir_inner);
                }
                let ty = HirType::Ref(Box::new(inner_ty), *mutable);
                Ok(SRef { expr: hir_inner, mutable: *mutable, ty }.into())
            }
            Expr::ArraySized { elem_type, count, .. } => {
                let hir_count = self.lower_expr(count)?;
                let elem_ty = ast_type_to_hir(elem_type, &self.interfaces);
                // If count is a compile-time constant, use ArraySized type
                let ty = if let Some(HirLiteral::Int(n)) = hir_count.as_const() {
                    HirType::Unique(Box::new(HirType::ArraySized(Box::new(elem_ty.clone()), *n as usize)))
                } else if let Some(other) = hir_count.as_const() {
                    // M6.1：非整数常量不能作数组大小（避免把浮点/字符常量发射成分配大小）
                    let s = count.span();
                    return Err(Error::Hir(format!(
                        "array size must be an integer constant, found {:?} (at {}:{})",
                        other, s.start_line, s.start_col
                    )));
                } else {
                    HirType::Unique(Box::new(HirType::Array(Box::new(elem_ty.clone()))))
                };
                Ok(SArrSz { count: hir_count, elem_ty, ty }.into())
            }
            Expr::Asm { template, outputs, inputs, .. } => self.lower_asm(template, outputs, inputs),
            Expr::Lambda { params, return_type, body, .. } => self.lower_lambda(params, return_type, body),
        }
    }
}
