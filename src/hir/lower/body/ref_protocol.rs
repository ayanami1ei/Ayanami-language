//! R1：引用协议（自含型）——任意含 `get(ref self) -> ref T` / `get_mut(ref mut self) -> ref mut T`
//! 的类型自动解引用（字段访问 / 方法调用 / 字段赋值）。
//!
//! 识别按**方法形态**（与迭代器协议同风格，无名字表、无标注）；设计见
//! `docs/reference-model.md` §2.1。`Ptr[T]` / `ref T` / `*T` 是既有特例，协议使其可扩展。
use super::*;
use crate::hir::lower::strip_generic_name;

impl crate::hir::lower::Ctx {
    /// 协议识别：返回 `(目标类型, 是否有可变 getter)`；非协议类型返回 `None`。
    pub(crate) fn ref_protocol_target(&self, ty: &HirType) -> Option<(HirType, bool)> {
        let base = match strip_ownership_ref(ty) {
            HirType::Named(n) => strip_generic_name(n),
            _ => return None,
        };
        let (target, _) = self.find_ref_protocol_method(ty, &base, "get", false)?;
        let has_mut = self.find_ref_protocol_method(ty, &base, "get_mut", true).is_some();
        Some((target, has_mut))
    }

    /// 查找协议方法（`get`/`get_mut`）：单参数 `ref self`（接收者基名匹配），返回 `ref T`。
    /// 覆盖非泛型 impl 方法（`method_fn_ids`）与泛型 impl 方法（`generic_fns`）。
    fn find_ref_protocol_method(
        &self,
        ty: &HirType,
        base: &Symbol,
        name: &str,
        want_mut: bool,
    ) -> Option<(HirType, bool)> {
        let shape_ok = |recv: &HirType, recv_mut: bool, ret: &HirType| -> Option<HirType> {
            if recv_mut != want_mut {
                return None;
            }
            let HirType::Named(recv_name) = strip_ownership_ref(recv) else { return None };
            if strip_generic_name(recv_name) != *base {
                return None;
            }
            let HirType::Ref(ret_inner, _) = ret else { return None };
            // 目标类型：返回类型中的泛型参数 → 用接收者实例的实参替换
            ref_target_from_return(ty, ret_inner, &self.generic_struct_params)
        };
        // 非泛型 impl 方法
        for fid in &self.method_fn_ids {
            let sig = &self.fns[fid.0];
            if sig.name.as_str() != name || sig.params.len() != 1 {
                continue;
            }
            if let Some(target) = shape_ok(&sig.params[0].1, matches!(&sig.params[0].1, HirType::Ref(_, true)), &sig.return_type) {
                return Some((target, true));
            }
        }
        // 泛型 impl 方法（AST 签名 → HirType）
        for (n, _, stmt, is_method) in &self.generic_fns {
            if !*is_method || n.as_str() != name {
                continue;
            }
            let Stmt::FnDecl { params, return_type, .. } = stmt else { continue };
            if params.len() != 1 {
                continue;
            }
            let recv = ast_type_to_hir(&params[0].1, &self.interfaces);
            let ret = ast_type_to_hir(return_type, &self.interfaces);
            if let Some(target) = shape_ok(&recv, matches!(&recv, HirType::Ref(_, true)), &ret) {
                return Some((target, true));
            }
        }
        None
    }

    /// 合成协议 getter 调用（`receiver.get()` / `receiver.get_mut()`，返回引用）。
    pub(crate) fn build_ref_protocol_call(
        &mut self,
        receiver: HirNodeBox,
        receiver_ty: &HirType,
        mutable: bool,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let name = Symbol::intern(if mutable { "get_mut" } else { "get" });
        // 泛型方法 → 调用点特化；非泛型 → 方法表形状匹配取 FnId
        let fid = if self.generic_fns.iter().any(|(n, _, _, is_method)| *is_method && n.as_str() == name.as_str()) {
            self.specialize_generic_method_call(&name, &[receiver_ty.clone()], span)?
        } else {
            let base = match strip_ownership_ref(receiver_ty) {
                HirType::Named(n) => strip_generic_name(n),
                _ => return Err(Error::Hir(format!(
                    "internal: ref protocol receiver is not a named type (at {}:{})",
                    span.start_line, span.start_col
                ))),
            };
            self.method_fn_ids.iter().copied().find(|fid| {
                let sig = &self.fns[fid.0];
                sig.name.as_str() == name.as_str() && sig.params.len() == 1 && matches!(
                    &sig.params[0].1,
                    HirType::Ref(recv, m) if *m == mutable && matches!(
                        strip_ownership_ref(recv), HirType::Named(n) if strip_generic_name(n) == base
                    )
                )
            }).ok_or_else(|| Error::Hir(format!(
                "internal: ref protocol getter `{}` not found (at {}:{})",
                name.as_str(), span.start_line, span.start_col
            )))?
        };
        let param_ty = HirType::Ref(Box::new(receiver_ty.clone()), mutable);
        let arg = wrap_arg_for_param(receiver, &param_ty);
        // 与显式调用路径同规则：不可变引用不能作为 `ref mut` 参数（e88）
        if mutable && matches!(arg.expr_type(), HirType::Ref(_, false)) {
            return Err(Error::Hir(format!(
                "cannot borrow `{}` as mutable through an immutable reference (at {}:{})",
                hir_type_display(receiver_ty), span.start_line, span.start_col
            )));
        }
        let ty = self.fns[fid.0].return_type.clone();
        Ok(SCall { fn_id: fid, args: vec![arg], ty }.into())
    }

    /// R1 方法回退：接收者实现协议且方法不在接收者本身上时，`get()`/`get_mut()` 解引用后重试。
    /// 成功则替换 `receiver`/`receiver_ty` 并返回方法 FnId。
    pub(crate) fn try_ref_protocol_deref(
        &mut self,
        receiver: &mut HirNodeBox,
        receiver_ty: &mut HirType,
        method: &Symbol,
        arg_types: &[HirType],
        deref_arg_types: &[HirType],
        lit_mask: &[bool],
        span: &Span,
    ) -> Result<Option<FnId>> {
        let Some((target_ty, has_mut)) = self.ref_protocol_target(receiver_ty) else {
            return Ok(None);
        };
        for want_mut in [false, true] {
            if want_mut && !has_mut {
                continue;
            }
            if let Some(id) = self.resolve_method_dispatch(
                &target_ty, method, arg_types, deref_arg_types, lit_mask, span,
            ) {
                let call = self.build_ref_protocol_call(receiver.clone(), receiver_ty, want_mut, span)?;
                *receiver = call;
                *receiver_ty = target_ty;
                return Ok(Some(id));
            }
        }
        Ok(None)
    }
}

/// 返回类型中的目标类型：泛型参数名 → 接收者实例的对应实参；否则原样返回。
fn ref_target_from_return(
    recv_ty: &HirType,
    ret: &HirType,
    generic_struct_params: &HashMap<Symbol, Vec<(Symbol, Vec<Symbol>)>>,
) -> Option<HirType> {
    let HirType::Named(ret_name) = strip_ownership_ref(ret) else {
        return Some(strip_ownership_ref(ret).clone());
    };
    let base = match strip_ownership_ref(recv_ty) {
        HirType::Named(n) => strip_generic_name(n),
        _ => return None,
    };
    let Some(params) = generic_struct_params.get(&base) else {
        // 非泛型接收者：返回类型按原样（若与接收者同名则是自引用协议，不支持）
        return Some(strip_ownership_ref(ret).clone());
    };
    let Some(pos) = params.iter().position(|(n, _)| *n == *ret_name) else {
        return Some(strip_ownership_ref(ret).clone());
    };
    // 接收者实例 `Base<A,B>` 的实参
    let HirType::Named(recv_full) = strip_ownership_ref(recv_ty) else { return None };
    let name = recv_full.as_str();
    let inner = generic_inner(&name)?;
    let args = split_generic_args(inner);
    let arg = args.get(pos)?;
    Some(sig_str_to_hir(arg.trim()))
}
