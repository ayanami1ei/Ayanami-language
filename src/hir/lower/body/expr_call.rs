use super::*;

impl crate::hir::lower::Ctx {
    /// `explicit`：显式泛型实参（`ns.fn[T](...)`，普通调用为 None）
    pub(crate) fn lower_fn_call(&mut self, name: &Symbol, args: &Vec<Expr>, explicit: Option<&Vec<Type>>, span: &Span) -> Result<HirNodeBox> {
        // `Enum::Variant(args)` 被解析器合并为 `Enum.Variant`，按前缀类型分流
        if let Some((enum_name, variant_name)) = name.as_str().split_once('.') {
            let enum_sym = Symbol::intern(enum_name);
            if self.is_enum_type(&enum_sym) {
                return self.lower_enum_construct(
                    &enum_sym,
                    &Symbol::intern(variant_name),
                    args,
                    &vec![],
                    span,
                );
            }
        }

        // Step 1: lower all arguments
        let mut hir_args: Vec<HirNodeBox> = args.iter()
            .map(|a| self.lower_expr(a))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // M4：`Ptr::new(v)` 内置构造 —— 堆分配拥有指针（稳定地址）
        if name.as_str() == "Ptr.new" {
            if hir_args.len() != 1 {
                return Err(Error::Hir(format!(
                    "Ptr::new expects exactly 1 argument (at {}:{})", span.start_line, span.start_col
                )));
            }
            let arg = hir_args.pop().unwrap();
            let inner = strip_ownership(expr_type(&arg));
            // 阶段 1：仅指针表示内层类型可装箱（标量 unique 当前非指针表示）
            if !matches!(inner,
                HirType::Named(_) | HirType::FatPtr { .. } | HirType::Unique(_)
                | HirType::Array(_) | HirType::ArraySized(_, _) | HirType::Closure(..))
            {
                return Err(Error::Hir(format!(
                    "Ptr[T] currently supports struct/enum/interface/array elements only (got {}) (at {}:{})",
                    hir_type_display(&inner), span.start_line, span.start_col
                )));
            }
            return Ok(SToUnique { expr: arg, ty: HirType::Unique(Box::new(inner)) }.into());
        }

        // Step 2: extract arg types（引用实参另存解引用类型，用于回退解析）
        let arg_types: Vec<HirType> = hir_args.iter().map(expr_type).collect();
        let deref_arg_types: Vec<HirType> = arg_types.iter().map(deref_type).collect();

        // Step 3：显式泛型实参优先（零参构造函数特化后参数表相同，
        // 重载解析无法区分返回类型，必须先按显式实参特化）
        let explicit_id = match explicit {
            Some(types) => self.specialize_generic_call_with(name, &arg_types, Some(types), span).ok(),
            None => None,
        };
        let lit_mask: Vec<bool> = hir_args.iter().map(is_numeric_literal).collect();
        // Step 3: resolve overloaded function（原类型 → 字面量适配 → 解引用类型）
        let fn_id = match explicit_id {
            Some(fid) => fid,
            None => match self.resolve_fn_call(name, &arg_types)
            .or_else(|| self.resolve_fn_call_literals(name, &arg_types, &lit_mask))
            .or_else(|| self.resolve_fn_call(name, &deref_arg_types))
        {
            Some(fid) => fid,
            None => {
                // Step 3b: try generic specialization
                let spec = self.specialize_generic_call_with(name, &arg_types, explicit, span);
                let spec = match spec {
                    Ok(fid) => Ok(fid),
                    Err(_) => self.specialize_generic_call_with(name, &deref_arg_types, explicit, span),
                };
                match spec {
                    Ok(fid) => fid,
                    Err(e) => {
                        // Step 3c: 变量调用（裸函数指针 / 闭包 / 捕获的闭包变量）
                        let callable: Option<(HirNodeBox, HirType)> = if let Some(env) = &self.lambda_env {
                            env.lookup(name).map(|(idx, cty)| {
                                let base: HirNodeBox = SVar { var: env.var, ty: env.ty.clone() }.into();
                                let node: HirNodeBox = SField { object: base, field: *name, field_index: idx, ty: cty.clone() }.into();
                                (node, cty)
                            })
                        } else {
                            None
                        }.or_else(|| {
                            self.lookup_var(name).map(|(var_id, ty, _)| (SVar { var: var_id, ty: ty.clone() }.into(), ty))
                        });
                        if let Some((mut callee, mut cty)) = callable {
                            // M2：`ref Fn` / `ref fn` 形参 → 解引用后调用
                            if let HirType::Ref(inner, _) = &cty {
                                if matches!(&**inner, HirType::FnPtr(..) | HirType::Closure(..)) {
                                    if matches!(&**inner, HirType::Closure(_, _, _, true)) {
                                        return Err(Error::Hir(format!(
                                            "cannot call a borrowed FnOnce closure; take it by value (at {}:{})",
                                            span.start_line, span.start_col
                                        )));
                                    }
                                    cty = (**inner).clone();
                                    callee = SDeref { expr: callee, ty: cty.clone() }.into();
                                }
                            }
                            match &cty {
                                HirType::FnPtr(param_tys, ret_ty) => {
                                    let param_tys = param_tys.clone();
                                    let args: Vec<HirNodeBox> = hir_args.into_iter().enumerate().map(|(i, a)| {
                                        if i < param_tys.len() { wrap_arg_for_param(a, &param_tys[i]) } else { a }
                                    }).collect();
                                    let args = self.adapt_enum_args(args, &param_tys)?;
                                    return Ok(SCallP { fn_ptr: callee, args, ty: *ret_ty.clone() }.into());
                                }
                                HirType::Closure(param_tys, ret_ty, _, once) => {
                                    let param_tys = param_tys.clone();
                                    let args: Vec<HirNodeBox> = hir_args.into_iter().enumerate().map(|(i, a)| {
                                        if i < param_tys.len() { wrap_arg_for_param(a, &param_tys[i]) } else { a }
                                    }).collect();
                                    let args = self.adapt_enum_args(args, &param_tys)?;
                                    let iface = closure_iface_name(&param_tys, ret_ty);
                                    // FnOnce：调用消费闭包（临时变量持有；函数/作用域结束 drop 释放 env）
                                    let receiver = if *once {
                                        let tmp = VarId(self.locals.len());
                                        self.locals.push(HirLocal::new(
                                            Symbol::intern(&format!("__fn_once_{}", tmp.0)), cty.clone(), true));
                                        let tmp_node: HirNodeBox = SVar { var: tmp, ty: cty.clone() }.into();
                                        self.pending_stmts.push(HirStmt::Assign {
                                            target: tmp_node.clone(), value: implicit_move(callee), span: *span,
                                        });
                                        tmp_node
                                    } else { callee };
                                    return Ok(SVCall {
                                        receiver,
                                        interface: iface,
                                        method_index: 0,
                                        args,
                                        concrete_type: Symbol::intern("__closure"),
                                        ty: (**ret_ty).clone(),
                                    }.into());
                                }
                                _ => {}
                            }
                        }
                        // 传播真实原因；前缀类型未知时补 import 提示
                        if let Some((prefix, _)) = name.as_str().split_once('.') {
                            let pfx = Symbol::intern(prefix);
                            let base = crate::hir::lower::strip_generic_name(&pfx);
                            let known = self.is_enum_type(&pfx)
                                || self.struct_defs.contains_key(&pfx)
                                || self.struct_defs.contains_key(&base)
                                || self.type_ifaces.contains_key(&pfx);
                            if !known {
                                return Err(Error::Hir(format!(
                                    "{}（类型 `{}` 未知：若来自包，请确认已 import 对应模块）",
                                    e, prefix
                                )));
                            }
                        }
                        return Err(e);
                    }
                }
            }
        } };

        // Step 4: wrap args into fat pointers where needed, apply implicit moves
        // extern "C" 声明：实参按借用传递（C 侧不消费所有权）
        let extern_call = self.extern_fn_ids.contains(&fn_id);
        // Pre-register vtables for generic impl → interface (before the closure that can't use ?)
        let param_tys: Vec<HirType> = self.fns[fn_id.0].params.iter()
            .map(|(_, t)| t.clone())
            .collect();
        // M2：extern C 的裸函数指针形参 → 直接函数名降级为 FnPtr（安全代码不产生裸指针值）。
        // 统一后 FnPtr 形参只可能来自 extern 声明/定义（安全源码的 fn 类型会报错）。
        for (i, ast) in args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if matches!(param_tys[i], HirType::FnPtr(..)) {
                hir_args[i] = self.lower_extern_fn_arg(ast, span)?;
            }
        }
        for (i, arg) in hir_args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                let arg_ty = expr_type(arg);
                if let Some(ct) = Self::extract_concrete_type_name(&arg_ty) {
                    let base_ct = crate::hir::lower::strip_generic_name(&ct);
                    if !self.type_ifaces.contains_key(&ct) || !self.type_ifaces[&ct].contains(iface_name) {
                        if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                            && self.type_ifaces[&base_ct].contains(iface_name) {
                            // already registered
                        } else if self.check_generic_fns_for_iface(&base_ct, iface_name) {
                            self.register_generic_vtable(&ct, &base_ct, iface_name)?;
                        }
                    }
                }
            }
        }
        for (i, arg) in hir_args.iter().enumerate() {
            if i >= param_tys.len() { break; }
            if matches!(&param_tys[i], HirType::Ref(_, true)) && matches!(arg.expr_type(), HirType::Ref(_, false)) {
                return Err(Error::Hir(format!(
                    "cannot pass an immutable reference as a mutable reference (at {}:{})",
                    span.start_line, span.start_col
                )));
            }
        }
        hir_args = hir_args.into_iter().enumerate().map(|(i, arg)| {
            if i >= param_tys.len() { return arg; }
            // #155：比较表达式（i1）→ 数值形参：显式零扩展
            let arg = if arg.is_comparison() {
                let pt = strip_ownership_ref(&param_tys[i]);
                if matches!(pt, HirType::Int | HirType::IntN { .. } | HirType::Float | HirType::F32) {
                    SBoolToNum { expr: arg, ty: pt.clone() }.into()
                } else { arg }
            } else { arg };
            let arg_ty = expr_type(&arg);
            // Check if param expects FatPtr and arg is a concrete type that implements the interface
            if let HirType::FatPtr { name: iface_name, .. } = &param_tys[i] {
                let concrete_type = match &arg_ty {
                    HirType::Unique(inner) => {
                        if let HirType::Named(n) = inner.as_ref() { Some(*n) } else { None }
                    }
                    HirType::Named(n) => Some(*n),
                    HirType::Int => Some(Symbol::intern("int")),
                    HirType::Float => Some(Symbol::intern("float")),
                    HirType::Char => Some(Symbol::intern("char")),
                    HirType::Bool => Some(Symbol::intern("bool")),
                    _ => None,
                };
                if let Some(ct) = concrete_type {
                    if self.type_ifaces.contains_key(&ct)
                        && self.type_ifaces[&ct].contains(iface_name) {
                        // Build fat pointer
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                    let base_ct = crate::hir::lower::strip_generic_name(&ct);
                    if base_ct != ct && self.type_ifaces.contains_key(&base_ct)
                        && self.type_ifaces[&base_ct].contains(iface_name) {
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                    if self.type_ifaces.contains_key(&ct) && self.type_ifaces[&ct].contains(iface_name) {
                        return self.make_fatptr_arg(arg, &param_tys[i], ct, *iface_name);
                    }
                }
            }
            if matches!(param_tys[i], HirType::Unique(_) | HirType::Ref(..)) {
                if extern_call { wrap_arg_borrowed(arg, &param_tys[i]) } else { wrap_arg_for_param(arg, &param_tys[i]) }
            } else if matches!(arg_ty, HirType::Ref(..)) {
                // 值形参 + 引用实参：自动解引用
                wrap_arg_for_param(arg, &param_tys[i])
            } else if matches!(param_tys[i], HirType::IntN { .. }) && arg_ty == HirType::Int && as_int_literal(&arg).is_some() {
                // 整数字面量 → 定宽整数形参（值形参）
                retype_int_literal(arg, &param_tys[i])
            } else if matches!(param_tys[i], HirType::F32) && arg_ty == HirType::Float && as_float_literal(&arg).is_some() {
                // 浮点字面量 → f32 形参（值形参）
                retype_float_literal(arg, &param_tys[i])
            } else if implicit_cast_ok(&arg_ty, &param_tys[i]) {
                // 按值基元参数的隐式数值转换（char→int / int→float / char→float）
                SCast { expr: arg, ty: strip_ownership_ref(&param_tys[i]).clone() }.into()
            } else if matches!(arg_ty, HirType::Unique(_)) {
                // #73：导入 .lcl 的形参可能是未包装形式（如 `[T]` → Array），拥有值实参
                // 需标记移动，否则调用方仍会在帧退出时释放（String::new(buf, n) 悬空）
                if extern_call { arg } else { implicit_move(arg) }
            } else if matches!(param_tys[i], HirType::Closure(..)) {
                // M2：闭包实参（拥有值）→ 移动
                wrap_arg_for_param(arg, &param_tys[i])
            } else if extern_call {
                // extern "C" 声明：C 侧不消费所有权，命名类型实参同样按借用传递
                arg
            } else if type_needs_drop(&arg_ty, &self.struct_defs) {
                // 拥有堆数据的命名类型（String/含堆结构体/枚举）按值传参 → 移动
                // （未标记移动会导致重复传参双释放，且调用方帧退出时误释放）
                implicit_move(arg)
            } else {
                // POD 结构体：复制语义，不失效
                arg
            }
        }).collect();

        let mut hir_args = self.adapt_enum_args(hir_args, &param_tys)?;
        // 函数级 track_caller：调用点信息作为隐藏实参追加
        let hidden = self.fns[fn_id.0].hidden;
        if hidden > 0 {
            hir_args.extend(self.caller_hidden_args(span, hidden));
        }
        // M1.9：`-> !` / `#[noreturn]` 调用表达式类型为 `!`
        let ty = if self.fns[fn_id.0].is_noreturn { HirType::Never } else { self.fns[fn_id.0].return_type.clone() };
        Ok(SCall { fn_id, args: hir_args, ty }.into())
    }

}
