//! 可写位置地址计算（局部 / 引用值 / 字段 / 下标 / 解引用，含嵌套）。
use super::*;
use super::util::*;
use super::fn_lower::strip_ownership;
use super::mir_ref::as_ptr_tmp;

/// #91：递归计算「可写位置」的地址（局部 / 引用值 / 字段，含嵌套字段）
pub(super) fn place_ptr(expr: &MirNodeBox, ctx: &mut dyn LirLowerCtx) -> Option<LirValue> {
    // 引用值本身就是指针（局部需从栈槽加载指针）
    if matches!(expr.expr_type(), HirType::Ref(..)) {
        let val = expr.lower_to_lir(ctx);
        return Some(match val {
            LirValue::Tmp(_) => val,
            other => {
                let t = ctx.next_tmp();
                ctx.emit(SLirLoad { dest: t, src: extract_var(&other), ty: expr.expr_type() }.into());
                LirValue::Tmp(t)
            }
        });
    }
    if let Some(v) = expr.as_local() {
        return Some(LirValue::Var(v));
    }
    // M6.2：全局地址（直接）与「解引用全局」（自动借用 `f(STATIC)`）都取全局指针
    if let Some(name) = expr.as_global() {
        let dest = ctx.next_tmp();
        ctx.emit(SLirGlobalAddr { dest, name }.into());
        return Some(LirValue::Tmp(dest));
    }
    if let Some(inner) = expr.as_deref() {
        if let Some(name) = inner.as_global() {
            let dest = ctx.next_tmp();
            ctx.emit(SLirGlobalAddr { dest, name }.into());
            return Some(LirValue::Tmp(dest));
        }
        // M4：解引用位置 —— 内层为指针（ref/unique/Ptr）时，指针值即地址
        if matches!(inner.expr_type(), HirType::Ref(..) | HirType::Unique(_)) {
            let val = inner.lower_to_lir(ctx);
            return Some(match val {
                LirValue::Tmp(_) => val,
                other => {
                    let t = ctx.next_tmp();
                    ctx.emit(SLirLoad { dest: t, src: extract_var(&other), ty: inner.expr_type() }.into());
                    LirValue::Tmp(t)
                }
            });
        }
    }
    // #130：数组元素地址（`o.xs[0].a = v` 的对象是索引表达式）
    if let Some((obj, index)) = expr.as_index() {
        let obj_val = obj.lower_to_lir(ctx);
        let arr_tmp = as_ptr_tmp(ctx, obj_val, &obj.expr_type());
        let idx_val = index.lower_to_lir(ctx);
        let arr_tmp = array_base_through_ref(ctx, arr_tmp, &obj.expr_type());
        let obj_ty = strip_ownership(obj.expr_type());
        let elem_ty = array_elem_ty(&obj_ty).unwrap_or(HirType::Int);
        let dest = ctx.next_tmp();
        ctx.emit(SLirIndexAddr { dest, arr_tmp, index: idx_val, elem_ty }.into());
        return Some(LirValue::Tmp(dest));
    }
    if let Some((obj, field_index)) = expr.as_field_access() {
        let obj_ptr = place_ptr(obj, ctx)?;
        let dest = ctx.next_tmp();
        ctx.emit(SLirFieldAddr { dest, obj: obj_ptr, field_index, struct_ty: obj.expr_type() }.into());
        return Some(LirValue::Tmp(dest));
    }
    None
}

/// 从（可能带 `ref` / 拥有包装的）数组类型取元素类型。
pub(super) fn array_elem_ty(ty: &HirType) -> Option<HirType> {
    let base = match ty {
        HirType::Ref(inner, _) => strip_ownership((**inner).clone()),
        other => strip_ownership(other.clone()),
    };
    match base {
        HirType::Array(inner) | HirType::ArraySized(inner, _) => Some((*inner).clone()),
        _ => None,
    }
}

/// `ref [T]` / `ref mut [T]` 索引：ref 值是调用方数组变量槽的地址（ptr 的地址），
/// 索引前需先取出数组指针；普通拥有数组/数组值原样返回。
pub(super) fn array_base_through_ref(ctx: &mut dyn LirLowerCtx, base_tmp: u64, ty: &HirType) -> u64 {
    if let HirType::Ref(inner, _) = ty {
        if matches!(strip_ownership((**inner).clone()), HirType::Array(_) | HirType::ArraySized(_, _)) {
            let t = ctx.next_tmp();
            ctx.emit(SLirLoadPtr { dest: t, src: LirValue::Tmp(base_tmp), ty: (**inner).clone() }.into());
            return t;
        }
    }
    base_tmp
}
