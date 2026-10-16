//! P3b（Rust 方式）：数组越界检查 —— `arr[i]` 默认检查，`arr.get_unchecked(i)` 跳过（需 unsafe）。
//!
//! 降级：索引提升到临时量，插入 `if idx >= len { __ayanami_panic_bounds_at(idx, len, ...) }`。
//! 比较用 `usize`（负数亦被捕获）；panic 退出码 101（与 std panic 一致）。
//! 仅对类型中保留长度的数组（`[T; n]`：字面量/参数/静态数组）生效；
//! 动态 `[T]` 无长度信息，保持未检查原语。

use super::*;

impl crate::hir::lower::Ctx {
    /// 越界检查运行时助手 `__ayanami_panic_bounds_at`（C 序：line, col, file, index, len；noreturn）
    pub(crate) fn ensure_bounds_helper(&mut self) -> FnId {
        let name = Symbol::intern("__ayanami_panic_bounds_at");
        if let Some(id) = self.ovf_helpers.get(&name) {
            return *id;
        }
        let fn_id = FnId(self.fns.len());
        let params: Vec<(Symbol, HirType)> = vec![
            (Symbol::intern("__line"), HirType::Int),
            (Symbol::intern("__col"), HirType::Int),
            (Symbol::intern("__file"), HirType::Named(Symbol::intern("String"))),
            (Symbol::intern("index"), HirType::Int),
            (Symbol::intern("len"), HirType::Int),
        ];
        self.fns.push(crate::hir::lower::FnSig {
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: crate::hir::effects::EffectSet::default(),
            name,
            params: params.clone(),
            return_type: HirType::Void,
            span: Span::default(),
            hidden: 0,
            is_noreturn: true,
            extern_c: false,
            is_unsafe: false,
        });
        self.synth_externs.push(crate::hir::HirFn {
            span: Span::default(),
            attrs: Vec::new(),
            effects: crate::hir::effects::EffectDecl::default(),
            fn_id,
            name,
            is_inline: false,
            extern_c: true,
            is_specialized: false,
            is_pub: false,
            is_macro: false,
            follow_sources: Vec::new(),
            inferred: crate::hir::effects::EffectSet::default(),
            params,
            param_attrs: Vec::new(),
            return_type: HirType::Void,
            locals: Vec::new(),
            body: crate::hir::HirBlock::new(Vec::new()),
        });
        self.ovf_helpers.insert(name, fn_id);
        fn_id
    }

    /// 对已知长度 `len` 的数组索引插入越界检查；返回访问用的索引节点（临时量，避免重复求值）。
    pub(crate) fn insert_bounds_check(&mut self, index: HirNodeBox, len: usize, span: &Span) -> HirNodeBox {
        // __idx = <index>
        let idx_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__idx"), HirType::Int, false));
        let idx_node: HirNodeBox = SVar { var: idx_var, ty: HirType::Int }.into();
        self.pending_stmts.push(HirStmt::Assign {
            target: idx_node.clone(),
            value: index,
            span: *span,
        });
        // if (idx as usize) >= len { panic_bounds_at(line, col, file, idx, len) }
        let usize_ty = HirType::IntN { bits: 64, signed: false };
        let idx_u: HirNodeBox = SCast { expr: idx_node.clone(), ty: usize_ty.clone() }.into();
        let len_u: HirNodeBox = SConst { val: HirLiteral::Int(len as i64), ty: usize_ty.clone() }.into();
        let cond: HirNodeBox = SBin { op: BinaryOp::Ge, lhs: idx_u, rhs: len_u, ty: usize_ty.clone() }.into();
        let fid = self.ensure_bounds_helper();
        let mut args: Vec<HirNodeBox> = self.caller_hidden_args(span, 3);
        args.push(idx_node.clone());
        args.push(SConst { val: HirLiteral::Int(len as i64), ty: HirType::Int }.into());
        let call: HirNodeBox = SCall { fn_id: fid, args, ty: HirType::Void }.into();
        self.pending_stmts.push(HirStmt::If {
            cond,
            then_block: HirBlock::new(vec![HirStmt::Expr { expr: call, span: *span }]),
            elifs: Vec::new(),
            else_block: None,
            span: *span,
        });
        idx_node
    }
}
