use super::*;

impl Ctx {
    pub(crate) fn scoped_fn_name(&self, name: Symbol) -> Symbol {
        self.scoped_fn_name_in(self.current_module, name)
    }

    pub(crate) fn scoped_fn_name_in(
        &self,
        module: Option<Symbol>,
        name: Symbol,
    ) -> Symbol {
        let Some(module) = module else { return name };
        let qualified = Symbol::intern(&format!("{}.{}", module.as_str(), name.as_str()));
        if self.fn_map.contains_key(&qualified)
            || self.generic_fns.iter().any(|(n, ..)| *n == qualified)
        {
            if std::env::var_os("AYANAMI_MODULE_TRACE").is_some() {
                eprintln!(
                    "module trace: resolved {} -> {}",
                    name.as_str(),
                    qualified.as_str()
                );
            }
            qualified
        } else {
            name
        }
    }
}
