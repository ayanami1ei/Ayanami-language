use super::*;

pub(crate) fn implicit_move(expr: HirNodeBox) -> HirNodeBox {
    let ty = expr.expr_type();
    // 默认所有权：非 Copy 类型在赋值/传参时移动
    if !ty.is_copy() && !expr.is_move_or_clone() {
        SMove { expr, ty }.into()
    } else {
        expr
    }
}

/// 包装参数以匹配期望的参数类型（处理所有权转换）
pub(crate) fn wrap_arg_for_param(arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox {
    let arg_ty = arg.expr_type();
    let converted = match param_ty {
        HirType::Unique(pt) | HirType::Shared(pt) | HirType::Weak(pt) => {
            if arg_ty == *pt.as_ref() {
                match param_ty {
                    HirType::Unique(_) => SToUnique { expr: arg, ty: param_ty.clone() }.into(),
                    HirType::Shared(_) => SToShared { expr: arg, ty: param_ty.clone() }.into(),
                    HirType::Weak(_) => SToWeak { expr: arg, ty: param_ty.clone() }.into(),
                    _ => arg,
                }
            } else if let HirType::Unique(inner) = &arg_ty {
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Unique(_) => wrap_for_unique_param(arg, param_ty),
                        HirType::Shared(_) => SToShared { expr: arg, ty: param_ty.clone() }.into(),
                        HirType::Weak(_) => SToWeak { expr: arg, ty: param_ty.clone() }.into(),
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if let HirType::Shared(inner) = &arg_ty {
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Shared(_) => arg,
                        HirType::Unique(_) => SToUnique { expr: arg, ty: param_ty.clone() }.into(),
                        HirType::Weak(_) => SToWeak { expr: arg, ty: param_ty.clone() }.into(),
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if let HirType::Weak(inner) = &arg_ty {
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Weak(_) => arg,
                        HirType::Shared(_) => SToShared { expr: arg, ty: param_ty.clone() }.into(),
                        HirType::Unique(_) => SToUnique { expr: arg, ty: param_ty.clone() }.into(),
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if matches!(param_ty, HirType::Unique(_)) {
                wrap_for_unique_param(arg, param_ty)
            } else {
                arg
            }
        }
        HirType::Ref(pt, mutable) => {
            // ref 参数：已是借用直接传；否则对同类型左值自动取引用
            if matches!(&arg_ty, HirType::Ref(..)) {
                arg
            } else if arg_ty == **pt {
                SRef { expr: arg, mutable: *mutable, ty: param_ty.clone() }.into()
            } else {
                arg
            }
        }
        _ => arg,
    };
    // 按值参数（含 unique）消费实参：插入移动；shared/weak 仍是借用/共享语义
    match param_ty {
        HirType::Shared(_) | HirType::Weak(_) => converted,
        _ => implicit_move(converted),
    }
}

/// Like implicit_move, but also wraps plain values when the param expects Unique.
pub(crate) fn wrap_for_unique_param(expr: HirNodeBox, param_ty: &HirType) -> HirNodeBox {
    let ty = expr.expr_type();
    if matches!(param_ty, HirType::Unique(_))
        && !expr.is_move_or_clone()
    {
        SMove { expr, ty }.into()
    } else {
        expr
    }
}

pub(crate) fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("add"),
        BinaryOp::Sub => Some("sub"),
        BinaryOp::Mul => Some("mul"),
        BinaryOp::Div => Some("div"),
        BinaryOp::Mod => Some("rem"),
        BinaryOp::Eq => Some("eq"),
        BinaryOp::Neq => Some("ne"),
        BinaryOp::Lt => Some("lt"),
        BinaryOp::Gt => Some("gt"),
        BinaryOp::Le => Some("le"),
        BinaryOp::Ge => Some("ge"),
        BinaryOp::And | BinaryOp::Or => None, // logical ops not overloadable
    }
}

    /// 例如：`-`（负号）→ "neg"
    /// 将一元运算符映射到对应的方法名（用于运算符重载查找）
pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str> {
    match op {
        UnaryOp::Neg => Some("neg"),
        UnaryOp::Not => Some("not"),
    }
}
