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
    // M2：闭包实参按拥有值移动（非 Copy，调用方不再释放）
    if matches!(param_ty, HirType::Closure(..)) && matches!(arg.expr_type(), HirType::Closure(..)) {
        return implicit_move(arg);
    }
    // 整数字面量 → 定宽整数形参（值形参）
    if let HirType::IntN { .. } = param_ty {
        if arg.expr_type() == HirType::Int && as_int_literal(&arg).is_some() {
            return retype_int_literal(arg, param_ty);
        }
    }
    // 浮点字面量 → f32/f64 形参（值形参；整数/浮点字面量均可）
    if matches!(param_ty, HirType::Float | HirType::F32) {
        let at = arg.expr_type();
        let lit_ok = (at == HirType::Float && as_float_literal(&arg).is_some())
            || (at == HirType::Int && as_int_literal(&arg).is_some());
        if lit_ok {
            return retype_float_literal(arg, param_ty);
        }
    }
    // 值形参位置：`ref T` 自动解引用（引用形参保持原样）
    let arg = if !matches!(param_ty, HirType::Ref(..)) {
        let at = arg.expr_type();
        match &at {
            HirType::Ref(inner, _) if strip_ownership_ref(inner) == strip_ownership_ref(param_ty) => auto_deref(arg),
            _ => arg,
        }
    } else {
        arg
    };
    let arg_ty = arg.expr_type();
    // 隐式数值转换：char→int / int→float / char→float
    let arg = if implicit_cast_ok(&arg_ty, param_ty) {
        SCast { expr: arg, ty: strip_ownership_ref(param_ty).clone() }.into()
    } else {
        arg
    };
    let arg_ty = arg.expr_type();
    let converted = match param_ty {
        HirType::Unique(pt) => {
            // Copy 类型（int/float/char/bool）无需装箱
            if pt.as_ref().is_copy() {
                arg
            } else if arg_ty == *pt.as_ref() {
                match param_ty {
                    HirType::Unique(_) => SToUnique { expr: arg, ty: param_ty.clone() }.into(),
                    _ => arg,
                }
            } else if let HirType::Unique(inner) = &arg_ty {
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Unique(_) => wrap_for_unique_param(arg, param_ty),
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
    // 按值参数（含 unique）消费实参：插入移动；ref/shared/weak 是借用/共享语义
    match param_ty {
        HirType::Ref(..) => converted,
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
        BinaryOp::BitAnd => Some("bitand"),
        BinaryOp::BitOr => Some("bitor"),
        BinaryOp::BitXor => Some("bitxor"),
        BinaryOp::Shl => Some("shl"),
        BinaryOp::Shr => Some("shr"),
        BinaryOp::And | BinaryOp::Or => None, // logical ops not overloadable
    }
}

    /// 例如：`-`（负号）→ "neg"
    /// 将一元运算符映射到对应的方法名（用于运算符重载查找）
pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str> {
    match op {
        UnaryOp::Neg => Some("neg"),
        UnaryOp::Not => Some("not"),
        UnaryOp::BitNot => Some("bitnot"),
    }
}
