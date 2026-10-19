//! const-eval：二元运算求值（从 const_eval.rs 拆出，保持文件 ≤300 行）。
use super::*;

impl crate::hir::lower::Ctx {
    /// 常量二元运算（算术/比较/位运算/逻辑；类型不匹配/溢出 → 位置化报错）。
    pub(super) fn eval_const_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: &Span,
        env: &HashMap<Symbol, (HirType, HirLiteral)>,
        depth: usize,
    ) -> Result<(HirType, HirLiteral)> {
                let (lt, ll) = self.eval_const_expr(lhs, env, depth)?;
                let (rt, rl) = self.eval_const_expr(rhs, env, depth)?;
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
}
