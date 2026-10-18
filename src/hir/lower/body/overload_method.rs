//! 方法调用解析（bd 8yn 方法候选优先 + 接收者匹配）；从 overload_resolve.rs 拆出。
use super::*;
use super::overload_resolve::same_base_name;

impl crate::hir::lower::Ctx {
    /// 方法调用分派（bd 8yn）：1) 仅 impl 方法候选精确匹配；2) 同名**泛型方法**特化
    /// （精确门控 + 仅方法候选，避免被同名自由函数抢占）；3) 常规链（含自由函数以
    /// 接收者为首参的兼容用法）。
    pub(crate) fn resolve_method_dispatch(&mut self, receiver_ty: &HirType, method: &Symbol,
        arg_types: &[HirType], deref_arg_types: &[HirType], lit_mask: &[bool], span: &Span) -> Option<FnId> {
        self.resolve_method_only(receiver_ty, method, arg_types)
            .or_else(|| {
                let mut all_param_types = vec![receiver_ty.clone()];
                all_param_types.extend(deref_arg_types.iter().cloned());
                if self.has_generic_method_candidate(method, receiver_ty, all_param_types.len()) {
                    self.specialize_generic_method_call(method, &all_param_types, span).ok()
                } else {
                    None
                }
            })
            .or_else(|| self.resolve_method(receiver_ty, method, arg_types))
            .or_else(|| self.resolve_method_literals(receiver_ty, method, arg_types, lit_mask))
            .or_else(|| self.resolve_method(receiver_ty, method, deref_arg_types))
    }

    /// bd 8yn：方法调用候选顺序——impl 方法在前（自由函数同名时不抢占），
    /// 其余按注册顺序（自由函数以接收者为首参的兼容用法保留）。
    fn ordered_method_candidates(&self, name: &Symbol) -> Option<Vec<FnId>> {
        let all = self.fn_map.get(name)?;
        let mut candidates: Vec<FnId> = all.iter().copied()
            .filter(|id| self.method_fn_ids.contains(id)).collect();
        candidates.extend(all.iter().copied().filter(|id| !self.method_fn_ids.contains(id)));
        Some(candidates)
    }

    pub(crate) fn resolve_fn_call(&self, name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        if name.as_str() == "is_none" {
            eprintln!("DBG resolve_fn_call is_none args={:?} cands={:?}",
                arg_types.iter().map(hir_type_display).collect::<Vec<_>>(),
                self.fn_map.get(name).map(|v| v.iter().map(|id| {
                    let sig = &self.fns[id.0];
                    let vis = sig.params.len().saturating_sub(sig.hidden);
                    format!("{}:{:?}", id.0, sig.params.iter().take(vis).map(|(_, t)| hir_type_display(t)).collect::<Vec<_>>())
                }).collect::<Vec<_>>()));
        }
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
            HirType::Ref(inner, param_mut) => {
                if receiver == inner.as_ref() { return true; }
                let recv_inner2 = match receiver {
                    HirType::Unique(i) => i.as_ref(),
                    other => other,
                };
                if recv_inner2 == inner.as_ref() { return true; }
                // e88：`ref mut T` 接收者可调用 `ref self` 方法（可变引用退化为不可变引用；
                // 反向不允许，调用组装处另有 immut→mut 报错）
                if !*param_mut {
                    if let HirType::Ref(recv_inner, true) = receiver {
                        if recv_inner.as_ref() == inner.as_ref() { return true; }
                    }
                }
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
                self.param_compatible(pt, at)
                    || (*is_lit && ((is_int_type(pt) && is_int_type(at)) || (is_float_type(pt) && is_float_type(at))))
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
        let candidates = self.ordered_method_candidates(method_name)?;
        let mut found: Option<FnId> = None;
        for &fn_id in &candidates {
            let sig = &self.fns[fn_id.0];
            if sig.params.is_empty() { continue; }
            if !Ctx::receiver_matches_param(receiver_type, &sig.params[0].1) { continue; }
            let visible = sig.params.len().saturating_sub(sig.hidden);
            if visible == 0 { continue; }
            let remaining = &sig.params[1..visible];
            if remaining.len() != arg_types.len() { continue; }
            let ok = remaining.iter().zip(arg_types).zip(lit_mask).all(|(((_, pt), at), is_lit)| {
                pt == at
                    || (*is_lit && ((is_int_type(pt) && is_int_type(at)) || (is_float_type(pt) && is_float_type(at))))
            });
            if ok {
                if found.is_some() { return None; }
                found = Some(fn_id);
            }
        }
        found
    }

    pub(crate) fn resolve_method(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        self.resolve_method_impl(receiver_type, method_name, arg_types, false)
    }

    /// 仅 impl 方法候选的解析（bd 8yn：方法优先于同名自由函数）
    pub(crate) fn resolve_method_only(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType]) -> Option<FnId> {
        self.resolve_method_impl(receiver_type, method_name, arg_types, true)
    }

    fn resolve_method_impl(&self, receiver_type: &HirType, method_name: &Symbol, arg_types: &[HirType], methods_only: bool) -> Option<FnId> {
        let candidates = self.ordered_method_candidates(method_name)?;
        let candidates: Vec<FnId> = if methods_only {
            candidates.into_iter().filter(|id| self.method_fn_ids.contains(id)).collect()
        } else { candidates };
        if candidates.is_empty() { return None; }
        for &fn_id in &candidates {
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
        for &fn_id in &candidates {
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
        for &fn_id in &candidates {
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
