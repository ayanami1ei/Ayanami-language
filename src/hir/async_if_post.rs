//! A1b-2：单分支 if await 之后的连续 await（y=默认; if cond { y=await g }; z_i=await h_i; tail）。
//! 无 post await 时仍由 async_if.rs 处理。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Post {
    target: Symbol,
    g: Symbol,
    placeholders: Vec<String>,
    reals: Vec<String>,
    ret: Type,
    ty: String,
    init: String,
}

/// 尝试展开「if 单分支 await + 连续 post await」；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, vis, return_type, span, .. } = stmt else { return Ok(false) };
    let Some((y, g, gargs, cond, raws, tail)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let g_ret = returns.get(&g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
    let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| fail(
        "unsupported await value type in if-branch (v1)".into()))?;
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
    let mut branch_ph = Vec::new();
    let mut branch_real = Vec::new();
    for a in &gargs {
        let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
        if params.iter().any(|(p, _)| p == n) {
            branch_ph.push(n.as_str().to_string());
            branch_real.push(format!("self.{}", n.as_str()));
        } else if *n == y {
            branch_ph.push(y_init.to_string());
            branch_real.push("self.y".to_string());
        } else {
            return Err(fail(format!(
                "await argument `{}` must be a parameter or the branch variable (v1)", n.as_str())));
        }
    }
    let mut posts = Vec::with_capacity(raws.len());
    for (z, h, hargs) in raws {
        let h_ret = returns.get(&h).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", h.as_str())))?.clone();
        let (z_ts, z_init) = await_default(&h_ret).ok_or_else(|| fail(
            "unsupported post await value type (v1)".into()))?;
        if z == y || params.iter().any(|(p, _)| p == &z) || posts.iter().any(|p: &Post| p.target == z) {
            return Err(fail(format!("post await target `{}` collides (v1)", z.as_str())));
        }
        let prior: Vec<(Symbol, Type)> = posts.iter().map(|p| (p.target, p.ret.clone())).collect();
        let (placeholders, reals) = crate::hir::async_post_args::build_post_args(
            &hargs[..], params, y, g_ret, &prior,
        ).ok_or_else(|| fail("unsupported post await argument (v1)".into()))?;
        posts.push(Post { target: z, g: h, placeholders, reals, ret: h_ret, ty: z_ts.to_string(), init: z_init.to_string() });
    }
    fields.push(format!("__{}_Fut __w0", g.as_str()));
    inits.push(format!("__w0 = {}({})", g.as_str(), branch_ph.join(", ")));
    for (i, p) in posts.iter().enumerate() {
        fields.push(format!("__{}_Fut __p{}", p.g.as_str(), i));
        inits.push(format!("__p{} = {}({})", i, p.g.as_str(), p.placeholders.join(", ")));
        fields.push(format!("{} {}", p.ty, p.target.as_str()));
        inits.push(format!("{} = {}", p.target.as_str(), p.init));
    }
    fields.push(format!("{} y", y_ty));
    fields.push("int _state".to_string());
    inits.push(format!("y = {}", y_init));
    inits.push("_state = 0".to_string());
    let fut = format!("__{}_Fut", name.as_str());
    let take = format!("__async_take_{}", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let args = self_args.join(", ");
    let mut after_args = if args.is_empty() { "self.y".to_string() } else { format!("{}, self.y", args) };
    for p in &posts {
        after_args.push_str(&format!(", self.{}", p.target.as_str()));
    }
    let final_state = 3 + posts.len();
    let mut poll = String::new();
    poll.push_str(&format!(
        "if self._state == 0 {{\n\
         \x20   if {take}({args}) {{\n\
         \x20       self.__w0 = {}({}); self._state = 1\n\
         \x20   }} else {{ self._state = 2 }}\n}}\n",
        g.as_str(), branch_real.join(", ")));
    poll.push_str(&format!(
        "if self._state == 1 {{\n\
         \x20   p1 = self.__w0.poll(ref cx)\n\
         \x20   match p1 {{\n\
         \x20       Ready(v) => {{ self.y = v; self._state = 2 }}\n\
         \x20       Pending => {{ return Poll::Pending() }}\n\
         \x20   }}\n}}\n"));
    if let Some(first) = posts.first() {
        poll.push_str(&format!(
            "if self._state == 2 {{\n\
             \x20   self.__p0 = {}({}); self._state = 3\n}}\n",
            first.g.as_str(), first.reals.join(", ")));
    }
    for (i, p) in posts.iter().enumerate() {
        let state = 3 + i;
        let ready = if i + 1 < posts.len() {
            let next = &posts[i + 1];
            format!("self.__p{} = {}({}); self._state = {}", i + 1, next.g.as_str(), next.reals.join(", "), state + 1)
        } else {
            format!("self._state = {}", final_state)
        };
        poll.push_str(&format!(
            "if self._state == {state} {{\n\
             \x20   pp{i} = self.__p{i}.poll(ref cx)\n\
             \x20   match pp{i} {{\n\
             \x20       Ready(v) => {{ self.{} = v; {ready} }}\n\
             \x20       Pending => {{ return Poll::Pending() }}\n\
             \x20   }}\n}}\n",
            p.target.as_str()));
    }
    poll.push_str(&format!("return Poll::Ready({after}({after_args}))\n"));
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         {poll}\
         }}\n}}\n",
        fields = fields.join("\n    "), plist = plist.join(", "), inits = inits.join(", "),
        name = name.as_str(), ret = ret,
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    let mut take_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: tn, return_type: tr, body: tb, vis: tv, .. } = &mut take_fn {
        attrs.clear();
        *tn = Symbol::intern(&take);
        *tr = Type::Bool(*span);
        *tv = Visibility::Private;
        let mut b = tb.clone();
        b.stmts = vec![Stmt::Return { value: Some(cond), span: *span }];
        *tb = b;
    }
    let mut after_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: an, params: ap, body: ab, vis: av, .. } = &mut after_fn {
        attrs.clear();
        *an = Symbol::intern(&after);
        *av = Visibility::Private;
        ap.push((y, g_ret.clone()));
        for p in &posts {
            ap.push((p.target, p.ret.clone()));
        }
        let mut b = ab.clone();
        b.stmts = tail;
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

/// 匹配 `y=默认; if cond { y=await g(args) }; (z_i=await h_i(args))+; tail`（无 else/elif）。
fn detect(body: &Block) -> Option<(Symbol, Symbol, Vec<Expr>, Expr, Vec<(Symbol, Symbol, Vec<Expr>)>, Vec<Stmt>)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block: None, .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() { return None; }
    let [Stmt::Assign { name: y, value, .. }] = then_block.stmts.as_slice() else { return None };
    if y != pre { return None; }
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    let rest = &body.stmts[2..];
    let mut raws = Vec::new();
    let mut idx = 0;
    while idx < rest.len() {
        let Stmt::Assign { name: z, value, .. } = &rest[idx] else { break };
        let Expr::FnCall { name: an, args, .. } = value else { break };
        if an.as_str() != "__await" { break; }
        let Expr::FnCall { name: h, args: hargs, .. } = args.first()? else { break };
        raws.push((*z, *h, hargs.clone()));
        idx += 1;
    }
    if raws.is_empty() { return None; }
    let tail = rest[idx..].to_vec();
    if tail.is_empty() { return None; }
    for st in &tail {
        if let Stmt::Assign { value, .. } = st {
            if matches!(value, Expr::FnCall { name, .. } if name.as_str() == "__await") { return None; }
        }
    }
    Some((*y, *g, gargs.clone(), cond.clone(), raws, tail))
}
