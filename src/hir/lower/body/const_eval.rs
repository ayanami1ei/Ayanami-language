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
        let (mut hir_ty, mut lit) = self.eval_const_expr(value)?;
        if let Some(ann) = ty {
            let want = ast_type_to_hir(ann, &self.interfaces);
            coerce_literal(&mut hir_ty, &mut lit, &want, span)?;
        }
        self.consts.insert(name, (hir_ty, lit));
        Ok(())
    }

    /// 常量表达式求值（M6.1：字面量/常量引用/一元/二元；不含函数调用与 cast）
    fn eval_const_expr(&self, expr: &Expr) -> Result<(HirType, HirLiteral)> {
        match expr {
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
                        coerce_literal(&mut HirType::Int, &mut out, &target_ty, span)?;
                        out
                    }
                    Literal::Float(f, _) => {
                        let mut out = HirLiteral::Float(*f);
                        coerce_literal(&mut HirType::Float, &mut out, &target_ty, span)?;
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
            Expr::Ident(name, span) => self.consts.get(name).cloned().ok_or_else(|| {
                Error::Hir(format!(
                    "const initializer must be a compile-time constant: `{}` is not a const (at {}:{})",
                    name, span.start_line, span.start_col
                ))
            }),
            Expr::Unary { op, arg, span } => {
                let (arg_ty, arg_lit) = self.eval_const_expr(arg)?;
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
            Expr::Binary { op, lhs, rhs, span } => {
                let (lt, ll) = self.eval_const_expr(lhs)?;
                let (rt, rl) = self.eval_const_expr(rhs)?;
                if lt != rt {
                    return Err(Error::Hir(format!(
                        "const operator type mismatch: {} vs {} (at {}:{})",
                        hir_type_display(&lt), hir_type_display(&rt), span.start_line, span.start_col
                    )));
                }
                let err_ovf = || {
                    Error::Hir(format!(
                        "const evaluation overflow (at {}:{})",
                        span.start_line, span.start_col
                    ))
                };
                let err_div0 = || {
                    Error::Hir(format!(
                        "const evaluation division by zero (at {}:{})",
                        span.start_line, span.start_col
                    ))
                };
                match (op, lt, ll, rl) {
                    (BinaryOp::Add, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l.checked_add(r).ok_or_else(err_ovf)?)))
                    }
                    (BinaryOp::Sub, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l.checked_sub(r).ok_or_else(err_ovf)?)))
                    }
                    (BinaryOp::Mul, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l.checked_mul(r).ok_or_else(err_ovf)?)))
                    }
                    (BinaryOp::Div, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l.checked_div(r).ok_or_else(err_div0)?)))
                    }
                    (BinaryOp::Mod, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l.checked_rem(r).ok_or_else(err_div0)?)))
                    }
                    (BinaryOp::BitAnd, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l & r)))
                    }
                    (BinaryOp::BitOr, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l | r)))
                    }
                    (BinaryOp::BitXor, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l ^ r)))
                    }
                    (BinaryOp::Shl, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l << (r & 63))))
                    }
                    (BinaryOp::Shr, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Int, HirLiteral::Int(l >> (r & 63))))
                    }
                    (BinaryOp::Eq, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l == r)))
                    }
                    (BinaryOp::Neq, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l != r)))
                    }
                    (BinaryOp::Lt, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l < r)))
                    }
                    (BinaryOp::Gt, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l > r)))
                    }
                    (BinaryOp::Le, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l <= r)))
                    }
                    (BinaryOp::Ge, HirType::Int, HirLiteral::Int(l), HirLiteral::Int(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l >= r)))
                    }
                    (op, ty @ (HirType::Float | HirType::F32), HirLiteral::Float(l), HirLiteral::Float(r)) => {
                        match op {
                            BinaryOp::Add => Ok((ty, HirLiteral::Float(l + r))),
                            BinaryOp::Sub => Ok((ty, HirLiteral::Float(l - r))),
                            BinaryOp::Mul => Ok((ty, HirLiteral::Float(l * r))),
                            BinaryOp::Div => {
                                if r == 0.0 {
                                    Err(err_div0())
                                } else {
                                    Ok((ty, HirLiteral::Float(l / r)))
                                }
                            }
                            BinaryOp::Eq => Ok((HirType::Bool, HirLiteral::Bool(l == r))),
                            BinaryOp::Neq => Ok((HirType::Bool, HirLiteral::Bool(l != r))),
                            BinaryOp::Lt => Ok((HirType::Bool, HirLiteral::Bool(l < r))),
                            BinaryOp::Gt => Ok((HirType::Bool, HirLiteral::Bool(l > r))),
                            BinaryOp::Le => Ok((HirType::Bool, HirLiteral::Bool(l <= r))),
                            BinaryOp::Ge => Ok((HirType::Bool, HirLiteral::Bool(l >= r))),
                            _ => Err(Error::Hir(format!(
                                "invalid const binary operator (at {}:{})",
                                span.start_line, span.start_col
                            ))),
                        }
                    }
                    (BinaryOp::And, HirType::Bool, HirLiteral::Bool(l), HirLiteral::Bool(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l && r)))
                    }
                    (BinaryOp::Or, HirType::Bool, HirLiteral::Bool(l), HirLiteral::Bool(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l || r)))
                    }
                    (BinaryOp::Eq, HirType::Bool, HirLiteral::Bool(l), HirLiteral::Bool(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l == r)))
                    }
                    (BinaryOp::Neq, HirType::Bool, HirLiteral::Bool(l), HirLiteral::Bool(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l != r)))
                    }
                    (BinaryOp::Eq, HirType::Char, HirLiteral::Char(l), HirLiteral::Char(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l == r)))
                    }
                    (BinaryOp::Neq, HirType::Char, HirLiteral::Char(l), HirLiteral::Char(r)) => {
                        Ok((HirType::Bool, HirLiteral::Bool(l != r)))
                    }
                    _ => Err(Error::Hir(format!(
                        "invalid const binary operator (at {}:{})",
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

/// 常量类型适配：按标注类型转换/检查字面量（含 IntN 范围检查）
fn coerce_literal(lit_ty: &mut HirType, lit: &mut HirLiteral, want: &HirType, span: &Span) -> Result<()> {
    match (want, &*lit_ty, &*lit) {
        (HirType::Int, HirType::Int, HirLiteral::Int(_)) => {}
        (HirType::Float, HirType::Int, HirLiteral::Int(v)) => {
            *lit = HirLiteral::Float(*v as f64);
            *lit_ty = HirType::Float;
        }
        (HirType::Float, HirType::Float, HirLiteral::Float(_)) => {}
        (HirType::F32, HirType::Int, HirLiteral::Int(v)) => {
            *lit = HirLiteral::Float(*v as f64);
            *lit_ty = HirType::F32;
        }
        (HirType::F32, HirType::Float, HirLiteral::Float(_)) => {
            *lit_ty = HirType::F32;
        }
        (HirType::IntN { bits, signed }, HirType::Int, HirLiteral::Int(v)) => {
            check_intn_range(*v, *bits, *signed, want, span)?;
            *lit_ty = want.clone();
        }
        (HirType::Char, HirType::Char, HirLiteral::Char(_)) => {}
        (HirType::Bool, HirType::Bool, HirLiteral::Bool(_)) => {}
        _ => {
            return Err(Error::Hir(format!(
                "const type mismatch: expected {}, found {} (at {}:{})",
                hir_type_display(want), hir_type_display(lit_ty), span.start_line, span.start_col
            )))
        }
    }
    Ok(())
}

/// 定宽整数范围检查（字面量以 i64 存储）
fn check_intn_range(v: i64, bits: u8, signed: bool, want: &HirType, span: &Span) -> Result<()> {
    let ok = if signed {
        let min = -(1i128 << (bits - 1));
        let max = (1i128 << (bits - 1)) - 1;
        (v as i128) >= min && (v as i128) <= max
    } else {
        let max = if bits >= 64 { u64::MAX as u128 } else { (1u128 << bits) - 1 };
        v >= 0 && (v as u128) <= max
    };
    if !ok {
        return Err(Error::Hir(format!(
            "literal out of range for `{}` (at {}:{})",
            hir_type_display(want), span.start_line, span.start_col
        )));
    }
    Ok(())
}
