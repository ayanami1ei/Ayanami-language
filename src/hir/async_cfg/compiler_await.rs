//! await 状态生成：future 字段、poll 状态与 Ready 写回。
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::Expr;

use super::FutureField;
use super::compiler::Cfg;

impl Cfg {
    pub(super) fn compile_await(
        &mut self,
        target: Symbol,
        g: Symbol,
        args: &[Expr],
        next: usize,
    ) -> Result<usize> {
        let local = self
            .locals
            .iter()
            .find(|l| l.name == target)
            .ok_or_else(|| {
                Error::Hir(format!(
                    "async CFG: await target `{}` is not a promoted local (v1)",
                    target.as_str()
                ))
            })?;
        let target_field = local.field;
        let idx = self.futures.len();
        let field = Symbol::intern(&format!("__w{}", idx));
        let params = self.async_fns.get(&g).ok_or_else(|| {
            Error::Hir(format!(
                "async CFG: await target `{}` is not an #[async] function (v1)",
                g.as_str()
            ))
        })?;
        let mut defaults = Vec::with_capacity(params.len());
        for (_, t) in params {
            let d = crate::hir::async_mixed_types::type_default(t).ok_or_else(|| {
                Error::Hir(format!(
                    "async CFG: unsupported dummy argument type for `{}` (v1)",
                    g.as_str()
                ))
            })?;
            defaults.push(d.to_string());
        }
        self.futures.push(FutureField {
            field,
            ty: format!("__{}_Fut", g.as_str()),
            init: format!("{}({})", g.as_str(), defaults.join(", ")),
        });

        let poll = self.alloc();
        let poll_code = format!(
            "__async_p{idx} = self.{field}.poll(ref __async_cx)\n\
             match __async_p{idx} {{\n\
             Ready(__async_v{idx}) => {{ self.{target_field} = __async_v{idx}; self._state = {next}; continue }}\n\
             Pending => {{ return Poll::Pending() }}\n\
             }}\n",
            field = field.as_str(),
            target_field = target_field.as_str(),
        );
        self.fill(poll, poll_code);

        let arg_src: Vec<String> = args.iter().map(|a| self.render_expr(a)).collect();
        let entry = self.alloc();
        let mut code = self.loads();
        code.push_str(&format!(
            "self.{field} = {g}({args});\n",
            field = field.as_str(),
            g = g.as_str(),
            args = arg_src.join(", "),
        ));
        code.push_str(&self.stores());
        code.push_str(&format!("self._state = {}; continue\n", poll));
        self.fill(entry, code);
        Ok(entry)
    }
}
