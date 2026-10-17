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

    /// 数组读写统一的越界检查入口：`[T; n]` 静态长度 / 动态字段兄弟边界；
    /// 非数组或无边界信息时原样返回索引。返回访问/写入使用的索引节点。
    pub(crate) fn check_array_index(&mut self, object: &HirNodeBox, index: HirNodeBox, span: &Span) -> HirNodeBox {
        let inner_ty = strip_ownership(object.expr_type());
        let deref_ty = strip_ownership(match &inner_ty {
            HirType::Ref(inner, _) => inner.as_ref().clone(),
            other => other.clone(),
        });
        match &deref_ty {
            HirType::ArraySized(_, n) => self.insert_bounds_check(index, *n, span),
            HirType::Array(_) => match self.sibling_len_expr(object) {
                Some(len_expr) => self.insert_bounds_check_dyn(index, len_expr, span),
                None => index,
            },
            _ => index,
        }
    }

    /// #175：动态数组字段的兄弟边界表达式（`s.data` → `s.capability`/`s.len`）；无则 None。
    /// **容量优先**（与 drop/clone 约定一致）：可增长缓冲（ArrayList/HashMap）的
    /// 可访问槽数是 capacity/cap，`len` 只是逻辑长度（push 写 `data[len]`、探测越 len）。
    /// 仅按名匹配（避免误用无关整数字段造成假 panic）；基址须为可寻址的局部/字段链。
    pub(crate) fn sibling_len_expr(&self, object: &HirNodeBox) -> Option<HirNodeBox> {
        let (base, _field, array_idx) = object.as_field_access()?;
        if !Self::place_rooted(base) {
            return None;
        }
        let base_ty = strip_ownership_ref(&base.expr_type()).clone();
        let HirType::Named(name) = &base_ty else { return None };
        let fields = self.struct_defs.get(name).or_else(|| {
            let n = name.as_str();
            n.find('<').map(|pos| Symbol::intern(&n[..pos])).and_then(|b| self.struct_defs.get(&b))
        })?;
        let is_int = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { bits: 64, .. });
        let find_named = |names: &[&str]| -> Option<(usize, &HirStructField)> {
            fields.iter().enumerate().find(|(i, f)| {
                *i != array_idx && is_int(&f.ty)
                    && names.iter().any(|n| f.name.as_str().as_str() == *n)
            })
        };
        // 容量优先（可增长缓冲），无容量字段才用逻辑长度
        let (len_idx, len_field) = find_named(&["capability", "capacity", "cap"])
            .or_else(|| find_named(&["len", "length", "size", "count"]))?;
        Some(SField {
            object: base.clone(),
            field: len_field.name,
            field_index: len_idx,
            ty: HirType::Int,
        }.into())
    }

    /// 基址是否可寻址（局部或字段链）——保证复制求值无副作用。
    fn place_rooted(node: &HirNodeBox) -> bool {
        if node.as_local().is_some() {
            return true;
        }
        match node.as_field_access() {
            Some((base, _, _)) => Self::place_rooted(base),
            None => false,
        }
    }

    /// 运行时长度表达式的越界检查（#175：动态数组字段 + 兄弟 len）；返回访问用索引临时量。
    pub(crate) fn insert_bounds_check_dyn(&mut self, index: HirNodeBox, len_expr: HirNodeBox, span: &Span) -> HirNodeBox {
        let idx_node = self.hoist_to_temp(index, "__idx", HirType::Int, span);
        let len_node = self.hoist_to_temp(len_expr, "__len", HirType::Int, span);
        let usize_ty = HirType::IntN { bits: 64, signed: false };
        let idx_u: HirNodeBox = SCast { expr: idx_node.clone(), ty: usize_ty.clone() }.into();
        let len_u: HirNodeBox = SCast { expr: len_node.clone(), ty: usize_ty.clone() }.into();
        let cond: HirNodeBox = SBin { op: BinaryOp::Ge, lhs: idx_u, rhs: len_u, ty: usize_ty }.into();
        let fid = self.ensure_bounds_helper();
        let mut args: Vec<HirNodeBox> = self.caller_hidden_args(span, 3);
        args.push(idx_node.clone());
        args.push(len_node);
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

    /// 把表达式提升到临时局部（避免重复求值）。
    fn hoist_to_temp(&mut self, value: HirNodeBox, name: &str, ty: HirType, span: &Span) -> HirNodeBox {
        let var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern(name), ty.clone(), false));
        let node: HirNodeBox = SVar { var, ty }.into();
        self.pending_stmts.push(HirStmt::Assign { target: node.clone(), value, span: *span });
        node
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
