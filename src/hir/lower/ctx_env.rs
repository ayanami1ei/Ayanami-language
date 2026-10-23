//! E1a：环境注入辅助——泛型 bound 驱动的环境需求与调用点提供值解析。
use super::*;

impl Ctx {
    /// 被调泛型函数的环境需求：泛型 bound 基名小写命中「当前函数提供者键」的 cap。
    pub(crate) fn env_need_caps(&self, gf_params: &[(Symbol, Vec<Symbol>)]) -> Vec<(Symbol, Symbol)> {
        let Some(mine) = self.env_providers.get(&self.current_fn) else { return Vec::new() };
        let mut out: Vec<(Symbol, Symbol)> = Vec::new();
        for (gp, bounds) in gf_params {
            for b in bounds {
                let key = Symbol::intern(&b.as_str().to_lowercase());
                if mine.contains_key(&key) && !out.iter().any(|(c, _)| *c == key) {
                    out.push((key, *gp));
                }
            }
        }
        out
    }

    /// 当前函数作用域内某能力 cap 的提供值：`#[env::cap(v)]`（最近/本函数）优先；
    /// 其次本函数的环境值参数 `__env_<cap>`（泛型传播用）。
    pub(crate) fn env_provider_node(&mut self, cap: &Symbol, span: &Span) -> Result<(HirNodeBox, HirType)> {
        if let Some((expr, _)) = self.env_providers.get(&self.current_fn)
            .and_then(|m| m.get(cap)).cloned()
        {
            let val = self.lower_expr(&expr)?;
            let ty = strip_ownership_ref(&val.expr_type()).clone();
            let node: HirNodeBox = SRef { expr: val, mutable: true, ty: HirType::Ref(Box::new(ty.clone()), true) }.into();
            return Ok((node, ty));
        }
        let pname = Symbol::intern(&format!("__env_{}", cap.as_str()));
        if let Some((id, ty, _)) = self.lookup_var(&pname) {
            if let HirType::Ref(inner, _) = ty {
                return Ok((SVar { var: id, ty: HirType::Ref(inner.clone(), true) }.into(), inner.as_ref().clone()));
            }
        }
        Err(Error::Hir(format!(
            "no environment provider for `{}` at {}:{} (add #[env::{}(...)] on the caller)",
            cap.as_str(), span.start_line, span.start_col, cap.as_str()
        )))
    }
}

impl Ctx {
    /// E1a：把环境值形参注入（特化后的）AST 形参表（可见形参之后、位置隐藏形参之前），
    /// 体降级据此注册 `__env_<cap>` 局部。
    pub(crate) fn inject_env_params(
        &self,
        new_params: &mut Vec<(Symbol, crate::parser::ast::Type)>,
        env_needs: &[(Symbol, Symbol)],
        mappings: &HashMap<Symbol, HirType>,
    ) {
        if env_needs.is_empty() { return; }
        let hn = count_hidden_names(new_params);
        let at = new_params.len().saturating_sub(hn);
        let env_ps: Vec<(Symbol, crate::parser::ast::Type)> = env_needs.iter()
            .map(|(cap, gp)| (
                Symbol::intern(&format!("__env_{}", cap.as_str())),
                hir_type_to_ast_type(&HirType::Ref(Box::new(
                    mappings.get(gp).cloned().unwrap_or(HirType::Named(*gp))), true)),
            ))
            .collect();
        for (i, p) in env_ps.into_iter().enumerate() {
            new_params.insert(at + i, p);
        }
    }
}
