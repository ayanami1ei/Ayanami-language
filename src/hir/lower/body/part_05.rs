use super::*;

/// 泛型单态化：两类型是否为同一基名（Named 去泛型参数后相等）。
fn same_base_name(a: &HirType, b: &HirType) -> bool {
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
            if arg_ty == inner.as_ref() || matches!(arg_ty, HirType::Ref(..)) {
                return true;
            }
            if let HirType::Unique(a) = arg_ty {
                if a.as_ref() == inner.as_ref() {
                    return true;
                }
            }
        }
        // FatPtr compatibility
        self.is_fatptr_compatible(param_ty, arg_ty)
    }

    /// Resolve a function call by name and argument types (overload-aware)
    pub(crate) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(name);
        let candidates = match candidates {
            Some(c) => c,
            None => {
                return None;
            }
        };
        let matches: Vec<FnId> = candidates.iter().copied()
            .filter(|&fn_id| {
                let sig = &self.fns[fn_id.0];
                let visible = sig.params.len().saturating_sub(sig.hidden);
                visible == arg_types.len()
                    && sig.params.iter().take(visible).zip(arg_types).all(|((_, pt), at)| {
                        self.param_compatible(pt, at)
                    })
            })
            .collect();
        if matches.len() == 1 {
            return Some(matches[0]);
        }
        if matches.len() > 1 {
            return None;
        }
        // 隐式数值转换回退（char→int / int→float / char→float）：
        // 仅在无精确匹配时参与，避免与精确重载竞争
        let casts: Vec<FnId> = candidates.iter().copied()
            .filter(|&fn_id| {
                // 泛型特化实例不参与隐式转换回退：
                // 否则 `add[char]` 会被 `add[int]`（char→int 转换）抢走
                if self.specialized_ids.contains(&fn_id) { return false; }
                let sig = &self.fns[fn_id.0];
                let visible = sig.params.len().saturating_sub(sig.hidden);
                visible == arg_types.len()
                    && sig.params.iter().take(visible).zip(arg_types).all(|((_, pt), at)| {
                        self.param_compatible(pt, at) || implicit_cast_ok(at, pt)
                    })
            })
            .collect();
        if casts.len() == 1 {
            return Some(casts[0]);
        }
        if casts.len() > 1 {
            return None;
        }
        // 泛型单态化回退：实参仍是基名（Result）而形参已实例化（Result<int,int>）时按基名匹配
        let lenient: Vec<FnId> = candidates.iter().copied()
            .filter(|&fn_id| {
                let sig = &self.fns[fn_id.0];
                sig.params.len() == arg_types.len()
                    && sig.params.iter().zip(arg_types).all(|((_, pt), at)| {
                        self.param_compatible(pt, at) || same_base_name(pt, at)
                    })
            })
            .collect();
        if lenient.len() == 1 {
            Some(lenient[0])
        } else {
            None
        }
    }

    /// 检查接收者类型是否匹配方法的 self 参数类型
    ///
    /// 支持多层所有权包装的自动剥离，例如：
    /// - `Unique(Shared(LinkedList))` 匹配 `Unique(LinkedList)`
    /// - `Shared(LinkedList)` 匹配 `LinkedList`
    /// - `Unique(Shared(LinkedList))` 匹配 `LinkedList`
    pub(crate) fn receiver_matches_param(receiver: &HirType, param: &HirType) -> bool {
        if receiver == param { return true; }
        // 逐层剥离接收者的所有权包装（Unique/Shared/Weak）
        // 处理多层包装如 Unique(Shared(T)) 的情况
        let recv_inner = match receiver {
            HirType::Unique(i) => i.as_ref(),
            other => other,
        };
        // 如果剥离后与参数完全相等，则匹配
        if recv_inner == param { return true; }
        // 再剥离参数的所有权包装
        let param_inner = match param {
            HirType::Unique(i) => i.as_ref(),
            other => other,
        };
        if recv_inner == param_inner { return true; }
        // 接收者仍有包装层未剥离？递归处理（如 Unique(Shared(T)) → Shared(T)）
        if recv_inner != receiver {
            return Self::receiver_matches_param(recv_inner, param);
        }
        // 允许向 shared/unique self 传入裸类型（自动包装）
        match param {
            HirType::Unique(inner) => {
                if receiver == inner.as_ref() { return true; }
            }
            // 允许向 ref/ref mut self 传入裸类型（自动借用）
            HirType::Ref(inner, _) => {
                if receiver == inner.as_ref() { return true; }
                let recv_inner2 = match receiver {
                    HirType::Unique(i) => i.as_ref(),
                    other => other,
                };
                if recv_inner2 == inner.as_ref() { return true; }
            }
            _ => {}
        }
        false
    }

    /// 重载解析（允许整数字面量适配任意整数形参；歧义返回 None）
    pub(crate) fn resolve_fn_call_literals(&self, name: &Symbol, arg_types: &[HirType], lit_mask: &[bool]) -> Option<FnId> {
        let candidates = self.fn_map.get(name)?;
        let mut found: Option<FnId> = None;
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible != arg_types.len() { continue; }
            let ok = sig.params.iter().take(visible).zip(arg_types).zip(lit_mask).all(|(((_, pt), at), is_lit)| {
                self.param_compatible(pt, at) || (*is_lit && is_int_type(pt) && is_int_type(at))
            });
            if ok {
                if found.is_some() { return None; }
                found = Some(fn_id);
            }
        }
        found
    }

    /// 方法重载解析（允许整数字面量适配任意整数形参；歧义返回 None）
    pub(crate) fn resolve_method_literals(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType], lit_mask: &[bool]) -> Option<FnId> {
        let candidates = self.fn_map.get(method_name)?;
        let mut found: Option<FnId> = None;
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible == 0 { continue; }
            let remaining = &sig.params[1..visible];
            if remaining.len() != arg_types.len() { continue; }
            let ok = remaining.iter().zip(arg_types).zip(lit_mask).all(|(((_, pt), at), is_lit)| {
                pt == at || (*is_lit && is_int_type(pt) && is_int_type(at))
            });
            if ok {
                if found.is_some() { return None; }
                found = Some(fn_id);
            }
        }
        found
    }

    pub(crate) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(method_name)?;
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible == 0 { continue; }
            let remaining = &sig.params[1..visible];
            if remaining.len() != arg_types.len() { continue; }
            if remaining.iter().zip(arg_types).all(|((_, pt), at)| pt == at) {
                return Some(fn_id);
            }
        }
        // 泛型单态化回退：实参是基名而形参已实例化（精确/基名匹配，含特化实例）
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible == 0 { continue; }
            let remaining = &sig.params[1..visible];
            if remaining.len() != arg_types.len() { continue; }
            if remaining.iter().zip(arg_types).all(|((_, pt), at)| {
                pt == at
                    || same_base_name(pt, at)
                    || strip_ownership_ref(pt) == strip_ownership_ref(at)
                    || same_base_name(strip_ownership_ref(pt), strip_ownership_ref(at))
            }) {
                return Some(fn_id);
            }
        }
        // 隐式转换回退（跳过泛型特化实例，避免串型）
        for &fn_id in candidates {
            if self.specialized_ids.contains(&fn_id) { continue; }
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible == 0 { continue; }
            let remaining = &sig.params[1..visible];
            if remaining.len() != arg_types.len() { continue; }
            if remaining.iter().zip(arg_types).all(|((_, pt), at)| {
                pt == at
                    || same_base_name(pt, at)
                    || strip_ownership_ref(pt) == strip_ownership_ref(at)
                    || same_base_name(strip_ownership_ref(pt), strip_ownership_ref(at))
                    || implicit_cast_ok(at, pt)
                    || implicit_cast_ok(strip_ownership_ref(at), strip_ownership_ref(pt))
            }) {
                return Some(fn_id);
            }
        }
        None
    }

    // ----------------------------------------------------------------
    //  阶段：泛型特化（单态化）
    //  当函数调用匹配到泛型函数时，根据具体参数类型生成特化版本
    // ----------------------------------------------------------------
}
