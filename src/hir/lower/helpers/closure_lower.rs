//! M2 闭包降级：环境结构体、代码函数、drop glue、vtable 与闭包值构建。
//!
//! 表示（见 docs/closures.md）：
//! - 闭包值 = 接口胖指针 `{ env, vtable }`；vtable = `[drop_glue, call]`；
//! - env = 编译器生成结构体 `__ClosureEnv_N`（字段 = 捕获变量）；
//! - 代码函数 `__lambda_N(__env: ref mut Env, 参数...)`，捕获读写落在 env 字段；
//! - drop glue `__closure_drop_N(unique Env)`：空体，靠 unique 参数作用域结束
//!   递归 drop 字段并释放环境。

use super::*;
use crate::error::{Error, Result};
use crate::hir::lower::FnSig;
use crate::hir::ty::{FnId, VarId};

/// M2：lambda 捕获环境（正在降级的 lambda body 内可见）
#[derive(Debug, Clone)]
pub struct LambdaEnv {
    /// env 参数（`ref mut __ClosureEnv_N`）在 lambda 函数内的 VarId
    pub var: VarId,
    /// env 参数类型（`ref mut Named(__ClosureEnv_N)`）
    pub ty: HirType,
    /// 捕获变量：(名字, 类型, env 字段下标)
    pub caps: Vec<(Symbol, HirType, usize)>,
}

impl LambdaEnv {
    pub fn lookup(&self, name: &Symbol) -> Option<(usize, HirType)> {
        self.caps.iter().find(|(n, _, _)| n == name).map(|(_, t, i)| (*i, t.clone()))
    }
}

/// 闭包合成接口名（每签名一个，供 vtable 命名；非用户接口）。
pub(crate) fn closure_iface_name(params: &[HirType], ret: &HirType) -> Symbol {
    let mut s = String::from("__Fn");
    for p in params {
        s.push('_');
        s.push_str(&mangle_for_symbol(p));
    }
    s.push_str("__r_");
    s.push_str(&mangle_for_symbol(ret));
    Symbol::intern(&s)
}

/// 类型名 → LLVM 安全符号片段（保留字母数字下划线，其余转 `_`，附短哈希防冲突）。
pub(crate) fn mangle_for_symbol(ty: &HirType) -> String {
    let text = crate::hir::display::display_type(ty);
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    out.push_str(&format!("_{:x}", h & 0xffffff));
    out
}

/// #119：从 lambda body 的 return 值推断返回类型（递归，取第一个带值 return）。
pub(crate) fn find_return_type(stmts: &[crate::hir::HirStmt]) -> Option<HirType> {
    for s in stmts {
        match s {
            // 比较运算在 HIR 中保留操作数类型（Bool 由 MIR 决定）→ 按 Bool 推断
            crate::hir::HirStmt::Return { value: Some(v), .. } => {
                return Some(if v.is_comparison() { HirType::Bool } else { v.expr_type() });
            }
            crate::hir::HirStmt::If { then_block, elifs, else_block, .. } => {
                if let Some(t) = find_return_type(&then_block.stmts) { return Some(t); }
                for (_, b) in elifs {
                    if let Some(t) = find_return_type(&b.stmts) { return Some(t); }
                }
                if let Some(b) = else_block {
                    if let Some(t) = find_return_type(&b.stmts) { return Some(t); }
                }
            }
            crate::hir::HirStmt::While { body, .. } => {
                if let Some(t) = find_return_type(&body.stmts) { return Some(t); }
            }
            _ => {}
        }
    }
    None
}

impl crate::hir::lower::Ctx {
    /// M2：有捕获 lambda → 闭包值（env + vtable + drop glue）。
    pub(crate) fn lower_closure_lambda(
        &mut self,
        params: &Vec<(Symbol, Type)>,
        return_type: &Type,
        body: &Block,
        caps: &[(Symbol, HirType)],
        name_sym: Symbol,
        n: u64,
    ) -> Result<HirNodeBox> {
        let env_sym = Symbol::intern(&format!("__ClosureEnv_{}", n));
        let drop_sym = Symbol::intern(&format!("__closure_drop_{}", n));
        let env_hir = HirType::Named(env_sym);

        // 1. env 结构体（字段 = 捕获变量，按捕获顺序）
        let env_fields: Vec<HirStructField> = caps.iter()
            .map(|(cn, ct)| HirStructField { name: *cn, ty: ct.clone() })
            .collect();
        self.struct_defs.insert(env_sym, env_fields);

        // 2. 代码函数：`fn __lambda_N(__env: ref mut Env, 参数...) -> ret`
        let env_param = Symbol::intern("__env");
        let env_ast_ty = Type::Ref(Box::new(Type::Named(env_sym, Span::default())), true, Span::default());
        let mut code_params: Vec<(Symbol, Type)> = vec![(env_param, env_ast_ty)];
        code_params.extend(params.iter().cloned());
        let hir_code_params: Vec<(Symbol, HirType)> = code_params.iter()
            .map(|(pn, pt)| (*pn, ast_type_to_hir(pt, &self.interfaces)))
            .collect();
        let hir_ret = ast_type_to_hir(return_type, &self.interfaces);
        let code_fn_id = FnId(self.fns.len());
        self.fns.push(FnSig {
            name: name_sym,
            params: hir_code_params.clone(),
            return_type: hir_ret.clone(),
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: Default::default(),
            span: Span::default(),
            hidden: 1,
            is_noreturn: matches!(hir_ret, HirType::Never),
        });
        self.fn_map.entry(name_sym).or_default().push(code_fn_id);

        // 3. drop glue：`fn __closure_drop_N(unique Env)`（空体，作用域结束递归 drop + free）
        let drop_fn_id = FnId(self.fns.len());
        let drop_params: Vec<(Symbol, Type)> =
            vec![(env_param, Type::Unique(Box::new(Type::Named(env_sym, Span::default())), Span::default()))];
        self.fns.push(FnSig {
            name: drop_sym,
            params: vec![(env_param, HirType::Unique(Box::new(env_hir.clone())))],
            return_type: HirType::Void,
            effects: crate::hir::effects::EffectDecl::default(),
            inferred: Default::default(),
            span: Span::default(),
            hidden: 0,
            is_noreturn: false,
        });
        self.fn_map.entry(drop_sym).or_default().push(drop_fn_id);
        self.lower_closure_drop_fn(drop_fn_id, drop_sym, &drop_params)?;

        // 4. 降级代码函数 body：捕获变量 → env 字段
        let env_var_ty = HirType::Ref(Box::new(env_hir.clone()), true);
        let lambda_env = LambdaEnv {
            var: VarId(0),
            ty: env_var_ty,
            caps: caps.iter().enumerate().map(|(i, (cn, ct))| (*cn, ct.clone(), i)).collect(),
        };
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::replace(&mut self.scopes, Vec::new());
        let saved_fn = self.current_fn;
        let saved_env = self.lambda_env.replace(lambda_env);
        let mut code_hir = self.lower_fn(code_fn_id, name_sym, &code_params, return_type, body, false, false, body.span, vec![], vec![], false)?;
        self.locals = saved_locals;
        self.scopes = saved_scopes;
        self.current_fn = saved_fn;
        self.lambda_env = saved_env;

        let inferred_ret = if matches!(hir_ret, HirType::Void) {
            find_return_type(&code_hir.body.stmts).unwrap_or(HirType::Void)
        } else {
            hir_ret.clone()
        };
        if inferred_ret != hir_ret {
            code_hir.return_type = inferred_ret.clone();
            self.fns[code_fn_id.0].return_type = inferred_ret.clone();
        }
        self.lambda_fns.push(code_hir);

        // 5. vtable：[drop_glue, call]（slot 0 = drop 预留槽，调用槽 = 1 + method_index）
        let lambda_param_tys: Vec<HirType> = hir_code_params[1..].iter().map(|(_, t)| t.clone()).collect();
        let iface_sym = closure_iface_name(&lambda_param_tys, &inferred_ret);
        self.vtables.push(VtableEntry {
            concrete_type: env_sym,
            interface: iface_sym,
            method_fn_ids: vec![drop_fn_id, code_fn_id],
        });

        // 6. 闭包值：MakeFatPtr(env 结构体值, vtable)
        let mut fields: Vec<(Symbol, HirNodeBox)> = Vec::new();
        for (cn, _ct) in caps {
            let src = self.capture_source_expr(cn)?;
            fields.push((*cn, implicit_move(src)));
        }
        let env_val: HirNodeBox = SStruct { type_name: env_sym, fields, ty: env_hir }.into();
        let closure_ty = HirType::Closure(lambda_param_tys, Box::new(inferred_ret), true);
        Ok(SMFP { value: env_val, concrete_type: env_sym, interface_name: iface_sym, ty: closure_ty }.into())
    }

    /// drop glue 函数体（空体即可：unique 参数在作用域结束时递归 drop + free）
    fn lower_closure_drop_fn(&mut self, fn_id: FnId, name: Symbol, params: &Vec<(Symbol, Type)>) -> Result<()> {
        let empty = Block::new(vec![], Span::default());
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_scopes = std::mem::replace(&mut self.scopes, Vec::new());
        let saved_fn = self.current_fn;
        let saved_env = self.lambda_env.take();
        let hir_fn = self.lower_fn(fn_id, name, params, &Type::Void(Span::default()), &empty, false, false, Span::default(), vec![], vec![], false)?;
        self.locals = saved_locals;
        self.scopes = saved_scopes;
        self.current_fn = saved_fn;
        self.lambda_env = saved_env;
        self.lambda_fns.push(hir_fn);
        Ok(())
    }

    /// 捕获源表达式：外层变量（SVar）或外层 lambda 的 env 字段（SField）
    fn capture_source_expr(&mut self, name: &Symbol) -> Result<HirNodeBox> {
        if let Some(outer) = &self.lambda_env {
            if let Some((idx, ty)) = outer.lookup(name) {
                let base: HirNodeBox = SVar { var: outer.var, ty: outer.ty.clone() }.into();
                return Ok(SField { object: base, field: *name, field_index: idx, ty }.into());
            }
        }
        if let Some((var, ty, _)) = self.lookup_var(name) {
            return Ok(SVar { var, ty }.into());
        }
        Err(Error::Hir(format!("internal error: captured variable `{}` not found", name.as_str())))
    }
}
