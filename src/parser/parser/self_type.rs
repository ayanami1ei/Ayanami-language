use super::*;

/// impl 方法签名中的 `Self` → impl 目标类型（含泛型实参）替换。
pub(super) fn subst_self_in_type(ty: &Type, self_ty: &Type) -> Type {
    match ty {
        Type::Self_(_) => self_ty.clone(),
        Type::Named(n, _) if n.as_str() == "Self" => self_ty.clone(),
        Type::Ref(inner, m, sp) => Type::Ref(Box::new(subst_self_in_type(inner, self_ty)), *m, *sp),
        Type::Unique(inner, sp) => Type::Unique(Box::new(subst_self_in_type(inner, self_ty)), *sp),
        Type::Array(inner, sp) => Type::Array(Box::new(subst_self_in_type(inner, self_ty)), *sp),
        Type::Generic(n, args, sp) => Type::Generic(
            *n,
            args.iter().map(|a| subst_self_in_type(a, self_ty)).collect(),
            *sp,
        ),
        Type::FnPtr(params, ret, sp) => Type::FnPtr(
            params.iter().map(|p| subst_self_in_type(p, self_ty)).collect(),
            Box::new(subst_self_in_type(ret, self_ty)),
            *sp,
        ),
        other => other.clone(),
    }
}
