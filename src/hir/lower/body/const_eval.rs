use super::*;

impl crate::hir::lower::Ctx {
    /// M6.1：收集并求值顶层 `const`（按文件顺序，可引用此前 const）
    pub(crate) fn collect_const_decl(
        &mut self,
        name: Symbol,
        ty: Option<&Type>,
        value: &Expr,
        span: &Span,
    ) -> Result<()> {
        let (mut hir_ty, mut lit) = self.eval_const_expr(value, &HashMap::new(), 0)?;
        if matches!(lit, HirLiteral::Array(_)) {
            return Err(Error::Hir(format!("array constants are not supported yet; use `static` (at {}:{})", span.start_line, span.start_col)));
        }
        if let Some(ann) = ty {
            let want = ast_type_to_hir(ann, &self.interfaces);
            super::const_coerce::coerce_literal(&mut hir_ty, &mut lit, &want, span)?;
        }
        self.consts.insert(name, (hir_ty, lit));
        Ok(())
    }

    /// M6.2：收集 `static` 定义（常量初始化；类型推断/标注同 const）
    pub(crate) fn collect_static_decl(
        &mut self,
        name: Symbol,
        is_mut: bool,
        is_pub: bool,
        ty: Option<&Type>,
        value: &Expr,
        span: &Span,
    ) -> Result<()> {
        let (mut hir_ty, mut lit) = self.eval_const_expr(value, &HashMap::new(), 0)?;
        if let Some(ann) = ty {
            let want = ast_type_to_hir(ann, &self.interfaces);
            super::const_coerce::coerce_literal(&mut hir_ty, &mut lit, &want, span)?;
        }
        self.statics.insert(name, crate::hir::HirStatic { name, ty: hir_ty, value: lit, is_mut, is_pub, is_external: false });
        Ok(())
    }

    /// 常量表达式求值（M6.1 字面量/引用/一元/二元；M6.3 追加 const fn 调用与 if 表达式）
    pub(crate) fn eval_const_expr(
        &self,
        expr: &Expr,
        env: &HashMap<Symbol, (HirType, HirLiteral)>,
        depth: usize,
    ) -> Result<(HirType, HirLiteral)> {
        match expr {
            Expr::ArrayLiteral(..) | Expr::ArrayRepeat { .. } => self.eval_const_array(expr, env, depth),
            Expr::StructLiteral { type_name, generic_args, fields, span } => self.eval_const_struct(type_name, generic_args, fields, span, env, depth),
            Expr::Literal(lit) => match lit {
                Literal::Int(i, _) => Ok((HirType::Int, HirLiteral::Int(*i))),
                Literal::Float(f, _) => Ok((HirType::Float, HirLiteral::Float(*f))),
                Literal::Char(c, _) => Ok((HirType::Char, HirLiteral::Char(*c))),
                Literal::Bool(b, _) => Ok((HirType::Bool, HirLiteral::Bool(*b))),
                Literal::String(_, s) => Err(Error::Hir(format!(
                    "string constants are not supported yet (at {}:{})",
                    s.start_line, s.start_col
                ))),
            },
            Expr::Suffixed { lit, suffix, span } => {
                let suffix_str = suffix.as_str();
                let target_ty = match fixed_width_type(&suffix_str) {
                    Some(ty) => ty,
                    None => match suffix_str.as_str() {
                        "int" => HirType::Int,
                        "float" | "f64" => HirType::Float,
                        "f32" => HirType::F32,
                        "char" => HirType::Char,
                        "bool" => HirType::Bool,
                        _ => {
                            return Err(Error::Hir(format!(
                                "unknown literal suffix `{}` (at {}:{})",
                                suffix_str, span.start_line, span.start_col
                            )))
                        }
                    },
                };
                let lit = match lit {
                    Literal::Int(i, _) => {
                        let mut out = HirLiteral::Int(*i);
                        super::const_coerce::coerce_literal(&mut HirType::Int, &mut out, &target_ty, span)?;
                        out
                    }
                    Literal::Float(f, _) => {
                        let mut out = HirLiteral::Float(*f);
                        super::const_coerce::coerce_literal(&mut HirType::Float, &mut out, &target_ty, span)?;
                        out
                    }
                    Literal::Char(c, _) => HirLiteral::Char(*c),
                    Literal::Bool(b, _) => HirLiteral::Bool(*b),
                    Literal::String(_, s) => {
                        return Err(Error::Hir(format!(
                            "string constants are not supported yet (at {}:{})",
                            s.start_line, s.start_col
                        )))
                    }
                };
                Ok((target_ty, lit))
            }
            Expr::Ident(name, span) => env.get(name).cloned()
                .or_else(|| self.consts.get(name).cloned())
                .ok_or_else(|| {
                    Error::Hir(format!(
                        "const initializer must be a compile-time constant: `{}` is not a const (at {}:{})",
                        name, span.start_line, span.start_col
                    ))
                }),
            Expr::Unary { op, arg, span } => {
                let (arg_ty, arg_lit) = self.eval_const_expr(arg, env, depth)?;
                match (op, arg_ty, arg_lit) {
                    (UnaryOp::Neg, HirType::Int, HirLiteral::Int(v)) => {
                        let negated = v.checked_neg().ok_or_else(|| {
                            Error::Hir(format!(
                                "const evaluation overflow (at {}:{})",
                                span.start_line, span.start_col
                            ))
                        })?;
                        Ok((HirType::Int, HirLiteral::Int(negated)))
                    }
                    (UnaryOp::Neg, ty @ (HirType::Float | HirType::F32), HirLiteral::Float(f)) => {
                        Ok((ty, HirLiteral::Float(-f)))
                    }
                    (UnaryOp::Not, HirType::Bool, HirLiteral::Bool(b)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(!b)))
                    }
                    (UnaryOp::BitNot, HirType::Int, HirLiteral::Int(v)) => {
                        Ok((HirType::Int, HirLiteral::Int(!v)))
                    }
                    _ => Err(Error::Hir(format!(
                        "invalid unary operator for const (at {}:{})",
                        span.start_line, span.start_col
                    ))),
                }
            }
            Expr::Binary { op, lhs, rhs, span } => self.eval_const_binary(op, lhs, rhs, span, env, depth),
            Expr::FnCall { name, args, generic_args, span } => {
                // Phase 7：布局内置（size_of/align_of/offset_of）在 const 上下文同样可用
                if let Some((t, v)) = self.eval_layout_builtin_call(name, generic_args, args, span)? {
                    return Ok((t, v));
                }
                // M6.3：`const fn` 调用 → 编译期解释执行
                if self.const_fns.contains_key(name) {
                    let mut vals = Vec::with_capacity(args.len());
                    for a in args {
                        vals.push(self.eval_const_expr(a, env, depth)?);
                    }
                    return self.eval_const_fn(*name, vals, depth + 1);
                }
                Err(Error::Hir(format!(
                    "const initializer must be a compile-time constant: `{}` is not a #[compile_time] function (at {}:{})",
                    name.as_str(), span.start_line, span.start_col
                )))
            }
            Expr::If { cond, then_block, elifs, else_block, span } => {
                // M6.3：if 表达式（分支块值）
                let (ct, cv) = self.eval_const_expr(cond, env, depth)?;
                let cval = matches!((ct, cv), (HirType::Bool, HirLiteral::Bool(true)));
                if cval {
                    return self.eval_const_block_value(then_block, env, depth);
                }
                for (c, b) in elifs {
                    let (ct, cv) = self.eval_const_expr(c, env, depth)?;
                    if matches!((ct, cv), (HirType::Bool, HirLiteral::Bool(true))) {
                        return self.eval_const_block_value(b, env, depth);
                    }
                }
                match else_block {
                    Some(b) => self.eval_const_block_value(b, env, depth),
                    None => Err(Error::Hir(format!(
                        "const if expression requires an else branch (at {}:{})",
                        span.start_line, span.start_col
                    ))),
                }
            }
            _ => {
                let s = expr.span();
                Err(Error::Hir(format!(
                    "const initializer must be a compile-time constant (at {}:{})",
                    s.start_line, s.start_col
                )))
            }
        }
    }
}
