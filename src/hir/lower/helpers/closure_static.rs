//! M2 静态闭包：命名函数 / 非捕获 lambda → Copy 闭包（零分配）。
//!
//! 表示：闭包值 `{ env, vtable }`，其中
//! - env = 裸函数代码指针（raw-convention，直接调用 `code(args)`）；
//! - vtable = `[noop_drop, trampoline]`；trampoline 签名 `ret (env, args...)`，
//!   转发为 `env(args...)`（env 即裸代码指针）。
//! 静态闭包按值复制安全（类型 `Closure(.., false)` 为 Copy）；被强制为拥有型后
//! 若发生 drop，vtable[0] 的 no-op 保证安全。

use super::*;
use crate::error::Result;
use crate::hir::lower::FnSig;
use crate::hir::ty::{FnId, VarId};

impl crate::hir::lower::Ctx {
    /// 静态闭包值：把裸函数值（FnPtr 类型）包成 Copy 闭包。
    pub(crate) fn make_static_closure(
        &mut self,
        raw_fn: HirNodeBox,
        ps: &[HirType],
        ret: &HirType,
    ) -> HirNodeBox {
        let (concrete, iface) = self.ensure_closure_tramp(ps, ret);
        SMFP {
            value: raw_fn,
            concrete_type: concrete,
            interface_name: iface,
            ty: HirType::Closure(ps.to_vec(), Box::new(ret.clone()), false, false),
        }.into()
    }

    /// 生成/缓存某签名的 trampoline + vtable，返回 (concrete_type, iface)。
    fn ensure_closure_tramp(&mut self, ps: &[HirType], ret: &HirType) -> (Symbol, Symbol) {
        let iface = closure_iface_name(ps, ret);
        if let Some(v) = self.closure_tramps.get(&iface) {
            return v.clone();
        }
        let noop = self.ensure_noop_drop();
        let sig_ty = HirType::Closure(ps.to_vec(), Box::new(ret.clone()), false, false);
        let concrete = Symbol::intern(&format!("__fnptr_tramp_{}", mangle_for_symbol(&sig_ty)));
        let name = concrete;
        let fn_id = FnId(self.fns.len());

        // 参数：env（FnPtr 签名）+ 原参数
        let env_sym = Symbol::intern("__env");
        let mut params: Vec<(Symbol, HirType)> =
            vec![(env_sym, HirType::FnPtr(ps.to_vec(), Box::new(ret.clone())))];
        for (i, p) in ps.iter().enumerate() {
            params.push((Symbol::intern(&format!("__p{}", i)), p.clone()));
        }
        self.fns.push(FnSig {
            name,
            params: params.clone(),
            return_type: ret.clone(),
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: Default::default(),
            span: Span::default(),
            hidden: 0,
            is_noreturn: matches!(ret, HirType::Never),
        });
        self.fn_map.entry(name).or_default().push(fn_id);

        // body：return env(__p0, __p1, ...)
        // 转发实参按移动处理：所有权交给被调函数，trampoline 作用域结束不再 drop
        let args: Vec<HirNodeBox> = (1..params.len())
            .map(|i| implicit_move(SVar { var: VarId(i), ty: params[i].1.clone() }.into()))
            .collect();
        let call: HirNodeBox = SCallP {
            fn_ptr: SVar { var: VarId(0), ty: params[0].1.clone() }.into(),
            args,
            ty: ret.clone(),
        }.into();
        let body = if matches!(ret, HirType::Void) {
            HirBlock::new(vec![
                HirStmt::Expr { expr: call, span: Span::default() },
                HirStmt::Return { value: None, span: Span::default() },
            ])
        } else {
            HirBlock::new(vec![HirStmt::Return { value: Some(call), span: Span::default() }])
        };
        let locals: Vec<HirLocal> = params.iter()
            .map(|(n, t)| HirLocal::new(*n, t.clone(), false))
            .collect();
        self.lambda_fns.push(HirFn {
            span: Span::default(),
            attrs: Vec::new(),
            effects: crate::hir::effects::EffectDecl::default(),
            fn_id,
            name,
            is_inline: false,
            extern_c: false,
            is_specialized: false,
            is_pub: false,
            is_macro: false,
            follow_sources: Vec::new(),
            inferred: Default::default(),
            params: params.clone(),
            param_attrs: Vec::new(),
            return_type: ret.clone(),
            locals,
            body,
        });

        // vtable：[noop_drop, trampoline]
        self.vtables.push(VtableEntry {
            concrete_type: concrete,
            interface: iface,
            method_fn_ids: vec![noop, fn_id],
        });
        self.closure_tramps.insert(iface, (concrete, iface));
        (concrete, iface)
    }

    /// 共享 no-op drop（静态闭包被强制为拥有型后 drop 时调用）。
    fn ensure_noop_drop(&mut self) -> FnId {
        let name = Symbol::intern("__closure_noop_drop");
        if let Some(ids) = self.fn_map.get(&name) {
            return ids[0];
        }
        let env = Symbol::intern("__env");
        let env_ty = HirType::Ref(Box::new(HirType::Void), false);
        let fn_id = FnId(self.fns.len());
        self.fns.push(FnSig {
            name,
            params: vec![(env, env_ty.clone())],
            return_type: HirType::Void,
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: Default::default(),
            span: Span::default(),
            hidden: 0,
            is_noreturn: false,
        });
        self.fn_map.entry(name).or_default().push(fn_id);
        self.lambda_fns.push(HirFn {
            span: Span::default(),
            attrs: Vec::new(),
            effects: crate::hir::effects::EffectDecl::default(),
            fn_id,
            name,
            is_inline: false,
            extern_c: false,
            is_specialized: false,
            is_pub: false,
            is_macro: false,
            follow_sources: Vec::new(),
            inferred: Default::default(),
            params: vec![(env, env_ty.clone())],
            param_attrs: Vec::new(),
            return_type: HirType::Void,
            locals: vec![HirLocal::new(env, env_ty, false)],
            body: HirBlock::new(vec![HirStmt::Return { value: None, span: Span::default() }]),
        });
        fn_id
    }
}

/// 命名函数作值：先取函数表首候选的 FnPtr 类型（与 expr_lower 原逻辑一致）。
fn fn_value_type(sig: &FnSig) -> HirType {
    let params: Vec<HirType> = sig.params.iter().map(|(_, t)| t.clone()).collect();
    HirType::FnPtr(params, Box::new(sig.return_type.clone()))
}

impl crate::hir::lower::Ctx {
    /// 外部 C 调用实参：直接函数名 → 裸 FnPtr；其余报错（unsafe 前不支持）。
    pub(crate) fn lower_extern_fn_arg(&mut self, ast: &Expr, span: &Span) -> Result<HirNodeBox> {
        if let Expr::Ident(name, _) = ast {
            if let Some(candidates) = self.fn_map.get(name) {
                if let Some(&first) = candidates.first() {
                    let ty = fn_value_type(&self.fns[first.0]);
                    if matches!(ty, HirType::FnPtr(..)) {
                        return Ok(SFnPtr { fn_id: first, ty }.into());
                    }
                }
            }
        }
        Err(crate::error::Error::Hir(format!(
            "extern C 函数指针参数目前只支持直接函数名（裸指针留待 unsafe）(at {}:{})",
            span.start_line, span.start_col
        )))
    }
}
