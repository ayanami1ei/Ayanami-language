//! A1b-2：if 单分支内多个 await 之后的连续 post await（无 else）。
//! y=默认; if c { y=await g0; ...; y=await gN }; z_i=await h_i; tail（N>=2）
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

#[allow(clippy::type_complexity)]
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, vis, return_type, span, .. } = stmt else { return Ok(false) };
    let Some((y, branch, cond, raws, tail)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let g0 = &branch[0].0;
    let first = returns.get(g0).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g0.as_str())))?;
    let first_src = ty_src(first).ok_or_else(|| fail("unsupported await value type (v1)".into()))?;
    for (g, _) in &branch {
        let r = returns.get(g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
        if ty_src(r).as_deref() != Some(first_src.as_str()) {
            return Err(fail("await values in if-branch must have the same type (v1)".into()));
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
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
    let mut real_args = Vec::new();
    let mut branch_ph = Vec::new();
    for (i, (g, gargs)) in branch.iter().enumerate() {
        let mut ph = Vec::new();
        let mut real = Vec::new();
        for a in gargs {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
            if params.iter().any(|(p, _)| p == n) {
                ph.push(n.as_str().to_string());
                real.push(format!("self.{}", n.as_str()));
            } else if *n == y {
                ph.push(y_init.to_string());
                real.push("self.y".to_string());
            } else {
                return Err(fail(format!(
                    "await argument `{}` must be a parameter or the branch variable (v1)", n.as_str())));
            }
        }
        fields.push(format!("__{}_Fut __w{}", g.as_str(), i));
        inits.push(format!("__w{} = {}({})", i, g.as_str(), ph.join(", ")));
        branch_ph.push(ph);
        real_args.push(real.join(", "));
    }
    let mut posts = Vec::new();
    for (z, h, hargs) in raws {
        let h_ret = returns.get(&h).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", h.as_str())))?.clone();
        let (z_ts, z_init) = await_default(&h_ret).ok_or_else(|| fail(
            "unsupported post await value type (v1)".into()))?;
        if z == y || params.iter().any(|(p, _)| p == &z) || posts.iter().any(|p: &Post| p.target == z) {
            return Err(fail(format!("post await target `{}` collides (v1)", z.as_str())));
        }
        let mut placeholders = Vec::new();
        let mut reals = Vec::new();
        for a in &hargs {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
            if params.iter().any(|(p, _)| p == n) {
                placeholders.push(n.as_str().to_string());
                reals.push(format!("self.{}", n.as_str()));
            } else if *n == y {
                placeholders.push(y_init.to_string());
                reals.push("self.y".to_string());
            } else if let Some(prev) = posts.iter().find(|p: &&Post| p.target == *n) {
                placeholders.push(prev.init.clone());
                reals.push(format!("self.{}", n.as_str()));
            } else {
                return Err(fail(format!(
                    "await argument `{}` must be a parameter/branch variable/earlier post result (v1)", n.as_str())));
            }
        }
        posts.push(Post { target: z, g: h, placeholders, reals, ret: h_ret, ty: z_ts.to_string(), init: z_init.to_string() });
    }
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
    let n = branch.len();
    let merge = n + 1;
    let final_state = merge + 1 + posts.len();
    let mut poll = String::new();
    poll.push_str(&format!(
        "if self._state == 0 {{\n\
         \x20   if {take}({args}) {{\n\
         \x20       self.__w0 = {}({}); self._state = 1\n\
         \x20   }} else {{ self._state = {merge} }}\n}}\n",
        branch[0].0.as_str(), real_args[0]));
    for i in 0..n {
        let state = i + 1;
        let ready = if i + 1 < n {
            format!("self.__w{} = {}({}); self._state = {}", i + 1, branch[i + 1].0.as_str(), real_args[i + 1], state + 1)
        } else {
            format!("self._state = {}", merge)
        };
        poll.push_str(&format!(
            "if self._state == {state} {{\n\
             \x20   p{state} = self.__w{i}.poll(ref cx)\n\
             \x20   match p{state} {{\n\
             \x20       Ready(v) => {{ self.y = v; {ready} }}\n\
             \x20       Pending => {{ return Poll::Pending() }}\n\
             \x20   }}\n}}\n"));
    }
    if let Some(first_post) = posts.first() {
        poll.push_str(&format!(
            "if self._state == {merge} {{ self.__p0 = {}({}); self._state = {} }}\n",
            first_post.g.as_str(), first_post.reals.join(", "), merge + 1));
    }
    for (i, p) in posts.iter().enumerate() {
        let state = merge + 1 + i;
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
        ap.push((y, first.clone()));
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

/// 匹配 `y=默认; if c { (y=await g_i)+ }; (z_i=await h_i)+; tail`（无 else/elif，分支 >=2）。
#[allow(clippy::type_complexity)]
fn detect(body: &Block) -> Option<(Symbol, Vec<(Symbol, Vec<Expr>)>, Expr, Vec<(Symbol, Symbol, Vec<Expr>)>, Vec<Stmt>)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block: None, .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() { return None; }
    let mut branch = Vec::new();
    for st in &then_block.stmts {
        let Stmt::Assign { name, value, .. } = st else { return None };
        if name != pre { return None; }
        let Expr::FnCall { name: an, args, .. } = value else { return None };
        if an.as_str() != "__await" { return None; }
        let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
        branch.push((*g, gargs.clone()));
    }
    if branch.len() < 2 { return None; }
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
    Some((*pre, branch, cond.clone(), raws, tail))
}
