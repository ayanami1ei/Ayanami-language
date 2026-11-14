//! A1b-2：if/elif(/else) 多分支 await（各分支一个 await，汇合到 merge 状态）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Branch {
    cond: Option<Expr>,          // None = else
    await_: Option<(Symbol, Vec<Expr>)>,
}

/// 尝试展开 elif 链；条件/分支结构不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some((y, branches)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let first_g = branches.iter().find_map(|b| b.await_.as_ref().map(|(g, _)| *g))
        .ok_or_else(|| fail("elif chain has no await (v1)".into()))?;
    let first_ret = returns.get(&first_g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", first_g.as_str())))?.clone();
    let (y_ty, y_init) = await_default(&first_ret)
        .ok_or_else(|| fail("unsupported await value type (v1)".into()))?;
    let Stmt::FnDecl { vis, return_type, span, .. } = stmt else { unreachable!() };
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let mut plist = Vec::new();
    let mut fields = Vec::new();
    let mut inits = Vec::new();
    let mut self_args = Vec::new();
    for (n, t) in params {
        let ts = ty_src(t).ok_or_else(|| fail(format!(
            "unsupported parameter type for `{}` (v1)", n.as_str())))?;
        plist.push(format!("{} {}", ts, n.as_str()));
        fields.push(format!("{} {}", ts, n.as_str()));
        inits.push(format!("{} = {}", n.as_str(), n.as_str()));
        self_args.push(format!("self.{}", n.as_str()));
    }
    let arg_render = |list: &[Expr]| -> Result<String> {
        let mut out = Vec::new();
        for a in list {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be parameters (v1)".into())) };
            if !params.iter().any(|(p, _)| p == n) {
                return Err(fail(format!("await argument `{}` must be a parameter (v1)", n.as_str())));
            }
            out.push(format!("self.{}", n.as_str()));
        }
        Ok(out.join(", "))
    };
    let fut = format!("__{}_Fut", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let merge = branches.iter().filter(|b| b.await_.is_some()).count() + 1;
    let mut state = 1;
    let mut dispatch = String::new();
    let mut polls = String::new();
    let mut take_fns = Vec::new();
    for (j, b) in branches.iter().enumerate() {
        let enter = match &b.await_ {
            Some((g, gargs)) => {
                let g_ret = returns.get(g).ok_or_else(|| fail(format!(
                    "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
                if ty_src(g_ret) != ty_src(&first_ret) {
                    return Err(fail("elif await values must have the same type (v1)".into()));
                }
                let real = arg_render(gargs)?;
                fields.push(format!("__{}_Fut __w{}", g.as_str(), j));
                inits.push(format!("__w{} = {}({})", j, g.as_str(),
                    gargs.iter().map(|a| match a {
                        Expr::Ident(n, _) => n.as_str().to_string(),
                        _ => String::new(),
                    }).collect::<Vec<_>>().join(", ")));
                let st = state;
                state += 1;
                polls.push_str(&format!(
                    "        if self._state == {st} {{\n\
                     \x20           p = self.__w{j}.poll(ref cx)\n\
                     \x20           match p {{\n\
                     \x20               Ready(v) => {{ self.y = v; self._state = {merge} }}\n\
                     \x20               Pending => {{ return Poll::Pending() }}\n\
                     \x20           }}\n\
                     \x20       }}\n"
                ));
                format!("self.__w{} = {}({}); self._state = {}", j, g.as_str(), real, st)
            }
            None => format!("self._state = {}", merge),
        };
        let kw = if j == 0 { "if" } else if b.cond.is_some() { "elif" } else { "else" };
        if let Some(cond) = &b.cond {
            let tn = format!("__async_take_{}_{}", name.as_str(), j);
            let mut tf = stmt.clone();
            if let Stmt::FnDecl { attrs, name: hn, return_type: tr, body: tb, vis: tv, .. } = &mut tf {
                attrs.clear();
                *hn = Symbol::intern(&tn);
                *tr = Type::Bool(*span);
                *tv = Visibility::Private;
                let mut nb = tb.clone();
                nb.stmts = vec![Stmt::Return { value: Some(cond.clone()), span: *span }];
                *tb = nb;
            }
            take_fns.push(tf);
            dispatch.push_str(&format!("        {kw} {tn}({sa}) {{ {enter} }}\n", sa = self_args.join(", ")));
        } else {
            dispatch.push_str(&format!("        else {{ {enter} }}\n"));
        }
    }
    if !branches.iter().any(|b| b.cond.is_none()) {
        dispatch.push_str(&format!("        else {{ self._state = {} }}\n", merge));
    }
    let field_body = format!("{}\n    {} y\n    int _state", fields.join("\n    "), y_ty);
    let init_body = format!("{}, y = {}, _state = 0", inits.join(", "), y_init);
    let src = format!(
        "{vis_s}struct {fut} {{\n    {field_body}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {init_body} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._state == 0 {{\n{dispatch}\x20       }}\n{polls}\
         \x20       return Poll::Ready({after}({sa}, self.y))\n    }}\n}}\n",
        plist = plist.join(", "), name = name.as_str(), ret = ret, sa = self_args.join(", "),
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    let mut after_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: an, params: ap, body: ab, vis: av, .. } = &mut after_fn {
        attrs.clear();
        *an = Symbol::intern(&after);
        *av = Visibility::Private;
        ap.push((y, first_ret.clone()));
        let mut nb = ab.clone();
        nb.stmts = body.stmts[2..].to_vec();
        *ab = nb;
    }
    out.extend(generated.stmts);
    take_fns.push(after_fn);
    for helper in take_fns {
        let hs = crate::formatter::format_program(&Program::new(vec![helper]));
        let h = crate::parser::parse_source(&hs)
            .map_err(|e| fail(format!("helper format failed: {}", e)))?;
        out.extend(h.stmts);
    }
    Ok(true)
}

/// 匹配 `y=<默认>; if c0 { ... } elif c1 { ... } [else { ... }]; ...`（至少一个 elif）。
fn detect(body: &Block) -> Option<(Symbol, Vec<Branch>)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block, .. }, ..] = body.stmts.as_slice() else { return None };
    if elifs.is_empty() { return None; }
    let y = branch_var(then_block)?;
    if *pre != y { return None; }
    let mut branches = vec![Branch { cond: Some(cond.clone()), await_: branch_await(then_block) }];
    for (c, b) in elifs {
        branches.push(Branch { cond: Some(c.clone()), await_: branch_await(b) });
    }
    if let Some(b) = else_block {
        branches.push(Branch { cond: None, await_: branch_await(b) });
    }
    Some((y, branches))
}

fn branch_var(b: &Block) -> Option<Symbol> {
    let [Stmt::Assign { name, .. }] = b.stmts.as_slice() else { return None };
    Some(*name)
}

fn branch_await(b: &Block) -> Option<(Symbol, Vec<Expr>)> {
    let [Stmt::Assign { value, .. }] = b.stmts.as_slice() else { return None };
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    Some((*g, gargs.clone()))
}
