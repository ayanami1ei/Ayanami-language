//! #160：从具体类型对接口的实现方法推断接口泛型实参。
//!
//! 例：约束 `I: Iterator[T]` 中 I 已由实参推断为 `ArrayListIter<int>`，
//! 则从 `next(ref mut self) -> Option[T]` 的实现（含泛型 impl）反推 T=int，
//! 供调用点补齐剩余泛型参数。
use super::*;

impl crate::hir::lower::Ctx {
    /// 返回接口泛型参数名 → 推断类型；全部推断成功才返回 Some。
    pub(crate) fn infer_iface_args_for_concrete(
        &self, iface_sym: &Symbol, concrete: &HirType,
    ) -> Option<HashMap<Symbol, HirType>> {
        let iface_base = crate::hir::lower::strip_generic_name(iface_sym);
        let iface_reg = self.interfaces.get(&iface_base)?.clone();
        let iface_gp: Vec<Symbol> = iface_reg.generic_params.iter().map(|(n, _)| *n).collect();
        if iface_gp.is_empty() { return None; }
        let concrete_name = match concrete {
            HirType::Named(n) => *n,
            HirType::FatPtr { name, .. } => *name,
            _ => return None,
        };
        let concrete_base = crate::hir::lower::strip_generic_name(&concrete_name);
        let mut subst: HashMap<Symbol, HirType> = HashMap::new();
        let mut matched = false;
        for im in &iface_reg.methods {
            // 候选 1：具体方法（self 基类型与具体类型一致）
            let mut candidates: Vec<(Vec<HirType>, HirType)> = Vec::new();
            for sig in &self.fns {
                if sig.name != im.name || sig.params.is_empty() { continue; }
                let self_ty = strip_ownership_ref(&sig.params[0].1).clone();
                if let HirType::Named(n) = &self_ty {
                    if crate::hir::lower::strip_generic_name(n) == concrete_base {
                        candidates.push((
                            sig.params[1..].iter().map(|(_, t)| t.clone()).collect(),
                            sig.return_type.clone(),
                        ));
                    }
                }
            }
            // 候选 2：泛型 impl 方法（用具体类型统一 self 类型得到 impl 泛型实参）
            for (gf_name, gp, gf_stmt) in &self.generic_fns {
                if *gf_name != im.name { continue; }
                let Stmt::FnDecl { params, return_type, .. } = gf_stmt else { continue };
                if params.is_empty() { continue; }
                let self_hir = strip_ownership_ref(&ast_type_to_hir(&params[0].1, &self.interfaces)).clone();
                let impl_gp: Vec<Symbol> = gp.iter().map(|(n, _)| *n).collect();
                let mut impl_subst = HashMap::new();
                if !Self::infer_iface_generic(&self_hir, concrete, &impl_gp, &mut impl_subst) { continue; }
                if !impl_subst.keys().all(|k| impl_gp.contains(k)) { continue; }
                let hir_params: Vec<HirType> = params[1..].iter()
                    .map(|(_, t)| substitute_hir_type(&ast_type_to_hir(t, &self.interfaces), &impl_subst))
                    .collect();
                let ret = substitute_hir_type(&ast_type_to_hir(return_type, &self.interfaces), &impl_subst);
                candidates.push((hir_params, ret));
            }
            // 用接口签名统一候选，得到接口泛型实参
            for (iparams, irets) in candidates {
                if iparams.len() != im.params.len() { continue; }
                let mut local = subst.clone();
                let mut ok = true;
                for ((_, ift), imt) in im.params.iter().zip(iparams.iter()) {
                    if !Self::infer_iface_generic(ift, imt, &iface_gp, &mut local) { ok = false; break; }
                }
                if ok && Self::infer_iface_generic(&im.return_type, &irets, &iface_gp, &mut local) {
                    subst = local;
                    matched = true;
                }
            }
        }
        if matched && iface_gp.iter().all(|n| subst.contains_key(n)) { Some(subst) } else { None }
    }

    /// #160：检查泛型约束；参数化约束（`I: It[T]`）先把泛型实参替换为已推断类型再比对。
    pub(crate) fn check_generic_constraint(
        &self, gp_name: &Symbol, iface_name: &Symbol,
        generic_mappings: &HashMap<Symbol, HirType>, span: &Span,
    ) -> Result<()> {

                let concrete_ty = generic_mappings.get(gp_name)
                    .ok_or_else(|| Error::Hir(format!("internal error: generic param `{}` not resolved at {}:{}", gp_name, span.start_line, span.start_col)))?;
                // 递归剥离所有权包装（Unique/Shared/Weak），
                // 处理多层包装如 Unique(Shared(LinkedList)) → LinkedList
                let mut concrete_inner = concrete_ty;
                while matches!(concrete_inner, HirType::Unique(_)) {
                    concrete_inner = strip_ownership_ref(concrete_inner);
                }
                let concrete_type_name = match concrete_inner {
                    HirType::Named(n) => {
                        let base = crate::hir::lower::strip_generic_name(n);
                        if self.type_ifaces.contains_key(n) { *n }
                        else { base }
                    }
                    HirType::FatPtr { name, .. } => {
                        // FatPtr 是接口类型（如 shared List）。
                        // 检查该接口是否包含了约束接口的所有方法。
                        let iface_methods = self.interfaces.get(iface_name)
                            .map(|reg| reg.methods.iter().map(|m| m.name).collect::<Vec<_>>())
                            .unwrap_or_default();
                        let all_ok = iface_methods.iter().all(|method_name| {
                            self.interfaces.get(name)
                                .map(|reg| reg.methods.iter().any(|m| m.name == *method_name))
                                .unwrap_or(false)
                        });
                        if !all_ok {
                            return Err(Error::Hir(format!(
                                "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                                hir_type_display(concrete_ty), iface_name, gp_name,
                                span.start_line, span.start_col
                            )));
                        }
                        // 直接标记为已实现，跳过后续 type_ifaces 检查
                        return Ok(());
                    }
                    HirType::Int => Symbol::intern("int"),
                    HirType::Float => Symbol::intern("float"),
                    HirType::Char => Symbol::intern("char"),
                    HirType::Bool => Symbol::intern("bool"),
                    HirType::Array(_) => Symbol::intern("[int]"), // simplified
                    _ => return Err(Error::Hir(format!(
                        "type `{}` does not satisfy interface `{}` for generic parameter `{}` at {}:{}",
                        hir_type_display(concrete_ty), iface_name, gp_name,
                        span.start_line, span.start_col
                    ))),
                };
                // #160：参数化约束先替换泛型实参（`It<T>` + T=int → `It<int>`）再比对
                let iface_text = iface_name.as_str();
                let expected_iface: Symbol = match generic_inner(&iface_text) {
                    Some(inner) => {
                        let args: Vec<String> = split_generic_args(inner).iter().map(|text| {
                            let sym = Symbol::intern(text.trim());
                            match generic_mappings.get(&sym) {
                                Some(t) => hir_type_display(t),
                                None => hir_type_display(&sig_str_to_hir(text.trim())),
                            }
                        }).collect();
                        Symbol::intern(&format!("{}<{}>", crate::hir::lower::strip_generic_name(iface_name), args.join(",")))
                    }
                    None => *iface_name,
                };
                let implements = self.type_ifaces.get(&concrete_type_name)
                    .map(|ifaces| {
                        ifaces.contains(&expected_iface)
                            || ifaces.contains(iface_name)
                            || ifaces.iter().any(|name| {
                                let s = name.as_str();
                                s.starts_with(&*iface_name.as_str()) && s.contains('<')
                            })
                    })
                    .unwrap_or(false);
                if !implements {
                    // 兜底：泛型 impl（`impl[T] Foo[T]`）按结构匹配接口。
                    // 具体 impl 已由 build_vtables 注册进 type_ifaces，这里只补泛型 impl 场景；
                    // 不得退化为「全局存在同名方法」检查——否则约束违例会被接受并错误特化。
                    let base_type = crate::hir::lower::strip_generic_name(&concrete_type_name);
                    if !self.check_generic_fns_for_iface(&base_type, iface_name) {
                        return Err(Error::Hir(format!(
                            "type `{}` does not implement interface `{}` required by generic parameter `{}` at {}:{}",
                            hir_type_display(concrete_ty), iface_name, gp_name,
                            span.start_line, span.start_col
                        )));
                    }
                }
        Ok(())
    }

    /// #161：具体类型值 → 接口胖指针字段/形参（含按需注册泛型 impl vtable）；
    /// 不适用（已是接口值/非具体类型/未实现接口）返回 None。
    pub(crate) fn coerce_iface_value(
        &mut self, arg: HirNodeBox, param_ty: &HirType, _span: &Span,
    ) -> Result<Option<HirNodeBox>> {
        let HirType::FatPtr { name: iface_name, .. } = param_ty else { return Ok(None); };
        let iface_name = *iface_name;
        // 泛型接口特化名（Iterator<int>）需先注册，register_generic_vtable 才能取到方法表
        self.ensure_specialized_interface(&iface_name)?;
        let arg_ty = expr_type(&arg);
        if matches!(&arg_ty, HirType::FatPtr { .. }) { return Ok(None); }
        let Some(ct) = Self::extract_concrete_type_name(&arg_ty) else { return Ok(None); };
        let base_ct = crate::hir::lower::strip_generic_name(&ct);
        let has = |s: &Self, n: &Symbol| s.type_ifaces.get(n).map(|v| v.contains(&iface_name)).unwrap_or(false);
        if !has(self, &ct) && !has(self, &base_ct)
            && self.check_generic_fns_for_iface(&base_ct, &iface_name)
        {
            self.register_generic_vtable(&ct, &base_ct, &iface_name)?;
        }
        if has(self, &ct) || has(self, &base_ct) {
            Ok(Some(self.make_fatptr_arg(arg, param_ty, ct, iface_name)))
        } else {
            Ok(None)
        }
    }
}
