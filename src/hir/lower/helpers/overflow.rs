//! M1.7：整数溢出检查运行时助手（debug 构建）
//!
//! `a + b` / `a - b` / `a * b`（整数）在非 release 下降级为
//! `__ayanami_ovf_{add,sub,mul}_{iN,uN}(a, b, line, col, file)` 调用；
//! 运行时溢出则 panic（退出码 101），release 直接回绕。

use super::*;

/// 溢出检查适用的运算符名（仅整数 + / - *）
pub(crate) fn ovf_op_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("add"),
        BinaryOp::Sub => Some("sub"),
        BinaryOp::Mul => Some("mul"),
        _ => None,
    }
}

impl crate::hir::lower::Ctx {
    /// 降级为溢出检查调用（debug 构建）
    pub(crate) fn lower_ovf_call(&mut self, op_name: &str, ty: &HirType, lhs: HirNodeBox, rhs: HirNodeBox, span: &Span) -> HirNodeBox {
        let fid = self.ensure_ovf_helper(op_name, ty);
        let mut args = vec![lhs, rhs];
        self.append_caller_args(fid, &mut args, span);
        SCall { fn_id: fid, args, ty: ty.clone() }.into()
    }

    /// 注册（或复用）溢出检查助手，返回其 FnId
    pub(crate) fn ensure_ovf_helper(&mut self, op: &str, ty: &HirType) -> FnId {
        let suffix = match ty {
            HirType::IntN { bits, signed } => intn_name(*bits, *signed),
            _ => "i64".to_string(),
        };
        let name = Symbol::intern(&format!("__ayanami_ovf_{}_{}", op, suffix));
        if let Some(id) = self.ovf_helpers.get(&name) {
            return *id;
        }
        let fn_id = FnId(self.fns.len());
        let params: Vec<(Symbol, HirType)> = vec![
            (Symbol::intern("a"), ty.clone()),
            (Symbol::intern("b"), ty.clone()),
            (Symbol::intern("__line"), HirType::Int),
            (Symbol::intern("__col"), HirType::Int),
            (Symbol::intern("__file"), HirType::Named(Symbol::intern("String"))),
        ];
        self.fns.push(crate::hir::lower::FnSig {
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: crate::hir::effects::EffectSet::default(),
            name,
            params: params.clone(),
            return_type: ty.clone(),
            span: Span::default(),
            hidden: 3,
        });
        self.synth_externs.push(crate::hir::HirFn {
            span: Span::default(),
            attrs: Vec::new(),
            effects: crate::hir::effects::EffectDecl::default(),
            fn_id,
            name,
            is_inline: false,
            extern_c: true,
            is_pub: false,
            is_macro: false,
            follow_sources: Vec::new(),
            inferred: crate::hir::effects::EffectSet::default(),
            params,
            param_attrs: Vec::new(),
            return_type: ty.clone(),
            locals: Vec::new(),
            body: crate::hir::HirBlock::new(Vec::new()),
        });
        self.ovf_helpers.insert(name, fn_id);
        fn_id
    }
}
