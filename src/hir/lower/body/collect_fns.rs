use super::*;

impl crate::hir::lower::Ctx {
    // ----------------------------------------------------------------
    //  阶段 1：收集函数和接口签名
    // ----------------------------------------------------------------

    pub(crate) fn collect_fns(&mut self, stmts: &[Stmt]) -> Result<()> {
        // M6.3：先收集全部 `const fn`（常量初始化可能引用后声明者）
        self.collect_const_fns(stmts, "")?;
        self.collect_fns_with_ns(stmts, "")
    }

    /// M6.3：收集 `const fn`（含命名空间）；泛型/extern 的 const fn 报错。
    fn collect_const_fns(&mut self, stmts: &[Stmt], prefix: &str) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { name, is_const: true, generic_params, extern_c, span, .. } => {
                    if !generic_params.is_empty() || *extern_c {
                        return Err(Error::Hir(format!(
                            "const fn cannot be generic or extern (at {}:{})",
                            span.start_line, span.start_col
                        )));
                    }
                    let full = if prefix.is_empty() {
                        *name
                    } else {
                        Symbol::intern(&format!("{}.{}", prefix, name))
                    };
                    self.const_fns.insert(full, stmt.clone());
                }
                Stmt::Namespace { name, items, .. } => {
                    let p = if prefix.is_empty() {
                        name.as_str().to_string()
                    } else {
                        format!("{}.{}", prefix, name)
                    };
                    self.collect_const_fns(items, &p)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
