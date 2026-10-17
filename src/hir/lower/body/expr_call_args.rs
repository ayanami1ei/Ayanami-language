//! 调用实参装配（比较零扩展 #155、接口装箱、隐式转换、移动标记、局部类型细化 #185）；
//! 从 expr_call.rs 拆出，保持文件 ≤300 行。
use super::*;

impl crate::hir::lower::Ctx {
    /// 实参装配：逐形参做隐式转换/装箱/移动标记。
    pub(super) fn wrap_call_args(&mut self, hir_args: Vec<HirNodeBox>, param_tys: &[HirType], extern_call: bool) -> Vec<HirNodeBox> {
        hir_args.into_iter().enumerate().map(|(i, arg)| {
            if i >= param_tys.len() { return arg; }
            let arg = self.refine_local_arg(arg, &param_tys[i]);
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
            } else if arg.is_comparison() && matches!(strip_ownership_ref(&param_tys[i]), HirType::Bool) {
                // #182：比较表达式实参 + Bool 形参 → 结果类型标注为 Bool（值即 i1）
                retype_comparison_bool(arg)
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
                }).collect()
    }

    /// #185：实参为局部（或 `ref` 包裹的局部）且形参是单态化类型（局部仍为裸基名）时，
    /// 细化局部类型并重建节点——否则调用以裸布局/错误 ABI 传参
    /// （裸 Option 16 字节 vs 单态化 24 字节 → 崩溃）。
    fn refine_local_arg(&mut self, arg: HirNodeBox, param_ty: &HirType) -> HirNodeBox {
        let (var, is_ref, mutable) = match arg.as_local() {
            Some(v) => (v, false, false),
            None => match arg.as_ref_expr().and_then(|inner| inner.as_local()) {
                Some(v) => {
                    let m = matches!(arg.expr_type(), HirType::Ref(_, true));
                    (v, true, m)
                }
                None => return arg,
            },
        };
        let pt = strip_ownership_ref(param_ty).clone();
        let cur = self.locals.get(var.0).map(|l| l.ty.clone());
        match (cur, &pt) {
            (Some(HirType::Named(cn)), HirType::Named(pn))
                if !cn.as_str().contains('<')
                    && pn.as_str().contains('<')
                    && crate::hir::lower::strip_generic_name(pn) == cn =>
            {
                self.update_var_type(var, pt.clone());
                if is_ref {
                    SRef {
                        expr: SVar { var, ty: pt.clone() }.into(),
                        mutable,
                        ty: HirType::Ref(Box::new(pt), mutable),
                    }.into()
                } else {
                    SVar { var, ty: pt }.into()
                }
            }
            _ => arg,
        }
    }
}
