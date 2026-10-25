//! A1b-2：while 循环体内的单个 await（回边到 state0，条件每轮重估）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

/// 尝试展开 while 内 await；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some((y, g, gargs, cond)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let g_ret = returns.get(&g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
    let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| fail(
        "unsupported await value type in while-loop (v1)".into()))?;
    let Stmt::FnDecl { vis, return_type, span, .. } = stmt else { unreachable!() };
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let mut pnames = Vec::new();
    let mut plist = Vec::new();
    let mut fields = Vec::new();
    let mut inits = Vec::new();
    let mut self_args = Vec::new();
    for (n, t) in params {
        let ts = ty_src(t).ok_or_else(|| fail(format!(
            "unsupported parameter type for `{}` (v1)", n.as_str())))?;
        pnames.push(n.as_str().to_string());
        plist.push(format!("{} {}", ts, n.as_str()));
        fields.push(format!("{} {}", ts, n.as_str()));
        inits.push(format!("{} = {}", n.as_str(), n.as_str()));
        self_args.push(format!("self.{}", n.as_str()));
    }
    // 构造器占位实参：参数用名字、循环变量用默认值
    let mut placeholder = Vec::new();
    let mut real = Vec::new();
    for a in &gargs {
        let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
        if params.iter().any(|(p, _)| p == n) {
            placeholder.push(n.as_str().to_string());
            real.push(format!("self.{}", n.as_str()));
        } else if *n == y {
            placeholder.push(y_init.to_string());
            real.push(format!("self.{}", n.as_str()));
        } else {
            return Err(fail(format!(
                "await argument `{}` must be a parameter or the loop variable (v1)", n.as_str())));
        }
    }
    let fut = format!("__{}_Fut", name.as_str());
    let take = format!("__async_take_{}", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let fields = format!(
        "{}\n    __{}_Fut __w\n    {} y\n    int _state",
        fields.join("\n    "), g.as_str(), y_ty);
    let inits = format!(
        "{}, __w = {}({}), y = {}, _state = 0",
        inits.join(", "), g.as_str(), placeholder.join(", "), y_init);
    let take_args = format!("{}, self.y", self_args.join(", "));
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       while true {{\n\
         \x20           if self._state == 0 {{\n\
         \x20               if {take}({take_args}) {{ self.__w = {g}({real}); self._state = 1 }} else {{ self._state = 2 }}\n\
         \x20           }}\n\
         \x20           if self._state == 1 {{\n\
         \x20               p1 = self.__w.poll(ref cx)\n\
         \x20               match p1 {{\n\
         \x20                   Ready(v) => {{ self.y = v; self._state = 0 }}\n\
         \x20                   Pending => {{ return Poll::Pending() }}\n\
         \x20               }}\n\
         \x20           }}\n\
         \x20           if self._state == 2 {{ return Poll::Ready({after}({take_args})) }}\n\
         \x20       }}\n\
         \x20       return Poll::Pending()\n    }}\n}}\n",
        plist = plist.join(", "), name = name.as_str(), ret = ret,
        real = real.join(", "),
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    // take helper：参数 + y → bool（每轮重估条件）
    let mut take_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: tn, params: tp, return_type: tr, body: tb, vis: tv, .. } = &mut take_fn {
        attrs.clear();
        *tn = Symbol::intern(&take);
        *tr = Type::Bool(*span);
        *tv = Visibility::Private;
        tp.push((y, g_ret.clone()));
        let mut b = tb.clone();
        b.stmts = vec![Stmt::Return { value: Some(cond), span: *span }];
        *tb = b;
    }
    // after helper：参数 + y + 后置语句
    let mut after_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: an, params: ap, body: ab, vis: av, .. } = &mut after_fn {
        attrs.clear();
        *an = Symbol::intern(&after);
        *av = Visibility::Private;
        ap.push((y, g_ret.clone()));
        let mut b = ab.clone();
        b.stmts = body.stmts[2..].to_vec();
        *ab = b;
    }
    out.extend(generated.stmts);
    for helper in [take_fn, after_fn] {
        let hs = crate::formatter::format_program(&Program::new(vec![helper]));
        let h = crate::parser::parse_source(&hs)
            .map_err(|e| fail(format!("helper format failed: {}", e)))?;
        out.extend(h.stmts);
    }
    Ok(true)
}

/// 匹配 `y = <默认>; while cond { y = await g(args) }; ...`。
fn detect(body: &Block) -> Option<(Symbol, Symbol, Vec<Expr>, Expr)> {
    let [Stmt::Assign { name: pre, .. }, Stmt::While { cond, body: wbody, .. }, ..] =
        body.stmts.as_slice() else { return None };
    let [Stmt::Assign { name: y, value, .. }] = wbody.stmts.as_slice() else { return None };
    if pre != y { return None; }
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Some(Expr::FnCall { name: g, args: gargs, .. }) = args.first() else { return None };
    Some((*y, *g, gargs.clone(), cond.clone()))
}
