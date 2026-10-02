use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn param_compatible(&self, param_ty: &HirType, arg_ty: &HirType) -> bool {
        if param_ty == arg_ty { return true; }
        // Shared/Unique value types: allow passing plain T to shared T
        if let HirType::Shared(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        if let HirType::Unique(inner) = param_ty {
            if arg_ty == inner.as_ref() { return true; }
        }
        // Allow passing Shared(T)/Unique(T)/Weak(T) to plain T
        // (ownership wrapper is transparent for primitives)
        if let HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) = arg_ty {
            if param_ty == inner.as_ref() { return true; }
        }
        // ref/ref mut 形参：允许传裸值（自动借用）或已借用值
        if let HirType::Ref(inner, _) = param_ty {
            if arg_ty == inner.as_ref() || matches!(arg_ty, HirType::Ref(..)) {
                return true;
            }
            if let HirType::Shared(a) | HirType::Unique(a) | HirType::Weak(a) = arg_ty {
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
                sig.params.len() == arg_types.len()
                    && sig.params.iter().zip(arg_types).all(|((_, pt), at)| {
                        self.param_compatible(pt, at)
                    })
            })
            .collect();
        if matches.len() == 1 {
            Some(matches[0])
        } else {
            if matches.len() > 1 {
            }
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
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        // 如果剥离后与参数完全相等，则匹配
        if recv_inner == param { return true; }
        // 再剥离参数的所有权包装
        let param_inner = match param {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if recv_inner == param_inner { return true; }
        // 接收者仍有包装层未剥离？递归处理（如 Unique(Shared(T)) → Shared(T)）
        if recv_inner != receiver {
            return Self::receiver_matches_param(recv_inner, param);
        }
        // 允许向 shared/unique self 传入裸类型（自动包装）
        match param {
            HirType::Shared(inner) | HirType::Unique(inner) => {
                if receiver == inner.as_ref() { return true; }
            }
            // 允许向 ref/ref mut self 传入裸类型（自动借用）
            HirType::Ref(inner, _) => {
                if receiver == inner.as_ref() { return true; }
                let recv_inner2 = match receiver {
                    HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
                    other => other,
                };
                if recv_inner2 == inner.as_ref() { return true; }
            }
            _ => {}
        }
        false
    }

    pub(crate) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        let candidates = self.fn_map.get(method_name)?;
        for &fn_id in candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let remaining = &sig.params[1..];
            if remaining.len() != arg_types.len() { continue; }
            if remaining.iter().zip(arg_types).all(|((_, pt), at)| pt == at) {
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
