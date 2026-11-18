use super::*;

/// 泛型实参推断冲突：两侧都具体且类别不同。
/// 数值互转与所有权包装差异放行（交由后续 coerce），单字母大写未解参数放行。
pub(super) fn types_infer_conflict(a: &HirType, b: &HirType) -> bool {
    let a = strip_ownership_ref(a);
    let b = strip_ownership_ref(b);
    if a == b {
        return false;
    }
    let numeric = |t: &HirType| matches!(
        t,
        HirType::Int | HirType::IntN { .. } | HirType::Float | HirType::F32
    );
    if numeric(a) && numeric(b) {
        return false;
    }
    let bare_param = |t: &HirType| matches!(t, HirType::Named(n)
        if n.as_str().len() == 1 && n.as_str().chars().all(|c| c.is_uppercase()));
    if bare_param(a) || bare_param(b) {
        return false;
    }
    matches!((a, b), (HirType::Named(_), HirType::Named(_)))
        || matches!(a, HirType::Named(_)) != matches!(b, HirType::Named(_))
}
