use super::*;

/// 泛型单态化：两类型是否为同一基名（Named 去泛型参数后相等）。
pub(super) fn same_base_name(a: &HirType, b: &HirType) -> bool {
    match (a, b) {
        (HirType::Named(x), HirType::Named(y)) => {
            let (xb, yb) = (crate::hir::lower::strip_generic_name(x), crate::hir::lower::strip_generic_name(y));
            xb == yb && (x != y || xb == *x)
        }
        _ => false,
    }
}

impl crate::hir::lower::Ctx {
    pub(crate) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        if param_ty == arg_ty { return true; }
        if let HirType::Unique(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        // Allow passing Shared(T)/Unique(T)/Weak(T) to plain T
        // (ownership wrapper is transparent for primitives)
        if let HirType::Unique(inner) = arg_ty {
            if param_ty == inner.as_ref() { return true; }
        }
        // 基名回退：裸泛型基名与同基名实例化（任一方向；`Box2` vs `Box2<String>` 等）。
        // 要求所有权包装一致：否则 ref 实参也会配到按值形参 → 重载歧义（#8yn 排查中暴露）
        if let (HirType::Named(pn), HirType::Named(an)) = (strip_ownership_ref(param_ty), strip_ownership_ref(arg_ty)) {
            let p_bare = !pn.as_str().contains('<');
            let a_bare = !an.as_str().contains('<');
            if p_bare != a_bare && crate::hir::lower::strip_generic_name(pn) == crate::hir::lower::strip_generic_name(an) {
                if p_bare {
                    // 正向（裸基名形参 ← 实例化实参）：允许包装差异（impl 自类型 + 自动借用）
                    return true;
                }
                // 反向（实例化形参 ← 裸基名实参）：要求包装一致，否则 ref 实参配到按值形参 → 歧义
                let same_wrap = matches!(param_ty, HirType::Ref(..)) == matches!(arg_ty, HirType::Ref(..))
                    && matches!(param_ty, HirType::Unique(_)) == matches!(arg_ty, HirType::Unique(_));
                if same_wrap {
                    return true;
                }
            }
        }
        // 数组：Array 与 ArraySized 在元素类型一致时兼容（定长缓冲区可传入不定长形参）
        {
            let p_inner = strip_ownership_ref(param_ty);
            let a_inner = strip_ownership_ref(arg_ty);
            let arrays_ok = match (p_inner, a_inner) {
                (HirType::Array(p), HirType::ArraySized(a, _))
                | (HirType::ArraySized(p, _), HirType::Array(a)) => p == a,
                _ => false,
            };
            if arrays_ok { return true; }
        }
        // ref/ref mut 形参：允许传裸值（自动借用）或已借用值
        if let HirType::Ref(inner, _) = param_ty {
            if arg_ty == inner.as_ref() {
                return true;
            }
            if let HirType::Ref(arg_inner, _) = arg_ty {
                // ref 形参 + ref 实参：内层类型须兼容。此前任意 ref 皆匹配 →
                // 重载误选（ArrayList[String] 索引解析到 String.index → 元素类型 char，#188）。
                // 内层不兼容时不提前返回，继续后续回退（基名匹配等）
                if self.param_compatible(inner.as_ref(), arg_inner.as_ref()) {
                    return true;
                }
            }
            if let HirType::Unique(a) = arg_ty {
                if a.as_ref() == inner.as_ref() {
                    return true;
                }
            }
        }
        // FatPtr compatibility
        self.is_fatptr_compatible(param_ty, arg_ty)
        // M2：静态闭包 → 拥有闭包（同布局，仅所有权标记不同；Copy 值可自由传入）
        || matches!((param_ty, arg_ty),
            (HirType::Closure(pps, pret, true, p_once), HirType::Closure(aps, aret, _, a_once))
                if pps == aps && pret == aret && (*p_once || !*a_once))
        // M2：静态闭包 → extern C 裸函数指针形参（实参侧替换为裸 FnPtr）
        || matches!((param_ty, arg_ty),
            (HirType::FnPtr(pps, pret), HirType::Closure(aps, aret, false, _))
                if pps == aps && pret == aret)
    }

}
