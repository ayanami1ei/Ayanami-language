//! P4c：统一结构匹配（设计 docs/generic-inference.md §3.1）。
//! 「期望类型（含泛型参数名）」× 「实际类型」递归匹配；泛型参数名绑定写入 `subst`。
//! 调用点推断等入口（`infer_generic_from_param`）共用这一条规则，嵌套泛型匹配不再各自实现。
use super::*;

/// 递归结构匹配，泛型参数名绑定写入 `subst`；返回形状是否匹配。
/// 绑定为尽力收集：调用方按函数泛型参数过滤并做冲突诊断（P4d）。
pub(crate) fn unify(expected: &HirType, actual: &HirType, subst: &mut HashMap<Symbol, HirType>) -> bool {
    match expected {
        // 泛型参数名 / 泛型实例编码 `Base<A,B>` / 接口名（Named / FatPtr 承载）
        HirType::Named(name) | HirType::FatPtr { name, .. } => unify_named(name, actual, subst),
        // 拥有包装：穿透（形参期望包装、实参未包装时按内层继续；自动装箱由后续处理）
        HirType::Unique(inner) => match actual {
            HirType::Unique(a) => unify(inner, a, subst),
            other => unify(inner, other, subst),
        },
        // 借用形参：忽略 `ref` 包装继续匹配内层
        HirType::Ref(inner, _) => unify(inner, actual, subst),
        // 裸指针 `*T`：两侧都按 `*T` 匹配内层
        HirType::RawPtr(inner) => match strip_ownership_ref(actual) {
            HirType::RawPtr(a) => unify(inner, a, subst),
            _ => false,
        },
        // 数组（拥有/定长）：从元素类型推导
        HirType::Array(inner) | HirType::ArraySized(inner, _) => match strip_ownership_ref(actual) {
            HirType::Array(a) | HirType::ArraySized(a, _) => unify(inner, a, subst),
            _ => false,
        },
        // 闭包 / 函数指针签名：逐参数 + 返回匹配
        HirType::Closure(ps, ret, _, _) | HirType::FnPtr(ps, ret) => match strip_ownership_ref(actual) {
            HirType::FnPtr(aps, aret) | HirType::Closure(aps, aret, _, _) => {
                ps.len() == aps.len()
                    && ps.iter().zip(aps.iter()).all(|(p, a)| unify(p, a, subst))
                    && unify(ret, aret, subst)
            }
            _ => false,
        },
        // 基元等：形状相等即匹配（数值互转 / 字面量适配由 coercion 处理，此处不绑定）
        other => strip_ownership_ref(actual) == other,
    }
}

/// `Named` / `FatPtr` 名字匹配：`Base<A,B>` 拆分后逐参数 `unify`；普通名字直接绑定。
fn unify_named(name: &Symbol, actual: &HirType, subst: &mut HashMap<Symbol, HirType>) -> bool {
    let s = name.as_str();
    let actual_name = match strip_ownership_ref(actual) {
        HirType::Named(n) | HirType::FatPtr { name: n, .. } => n.as_str(),
        _ => String::new(),
    };
    // 泛型实例编码 `Base<...>`：按配对括号拆分，逐参数递归
    if let Some(inner) = generic_inner(&s) {
        let base = &s[..s.find(['<', '[']).unwrap_or(s.len())];
        let parts = split_generic_args(inner);
        // 实参本身是泛型参数名（与形参名同名）→ 直接映射（兼容旧行为）
        if parts.iter().any(|p| p.trim() == actual_name) {
            subst.insert(Symbol::intern(&actual_name), actual.clone());
            return true;
        }
        // 基础名一致且实参也是泛型实例 → 逐参数匹配（zip 截断，兼容旧行为）
        let actual_base = actual_name.split(['<', '[']).next().unwrap_or(&actual_name);
        if actual_base != base {
            return false;
        }
        let Some(actual_inner) = generic_inner(&actual_name) else { return false; };
        let actual_parts = split_generic_args(actual_inner);
        for (ep, ap) in parts.iter().zip(actual_parts.iter()) {
            unify(&sig_str_to_hir(ep.trim()), &sig_str_to_hir(ap.trim()), subst);
        }
        return true;
    }
    // 普通名字（含具体类型名——调用方按函数泛型参数过滤）→ 绑定
    subst.insert(*name, actual.clone());
    true
}
