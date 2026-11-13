//! A1b-2：if/elif/else 各一个 await 之后的连续 await。
//! y=默认; if c0 { y=await g0 } elif c1 { ... } else { ... }; z_i=await h_i; tail
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Branch {
    cond: Option<Expr>,
    g: Symbol,
    args: Vec<Expr>,
}

struct Post {
    target: Symbol,
    g: Symbol,
    placeholders: Vec<String>,
    reals: Vec<String>,
    ret: Type,
    ty: String,
    init: String,
}

/// 尝试展开「if/elif/else 单 await + 连续 post await」；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, vis, return_type, span, .. } = stmt else { return Ok(false) };
    let Some((y, branches, raws, tail)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let first = returns.get(&branches[0].g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", branches[0].g.as_str())))?;
    let first_src = ty_src(first).ok_or_else(|| fail("unsupported await value type (v1)".into()))?;
    for b in &branches {
        let r = returns.get(&b.g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", b.g.as_str())))?;
        if ty_src(r).as_deref() != Some(first_src.as_str()) {
            return Err(fail("if/elif/else await values must have the same type (v1)".into()));
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
        "unsupported await value type in if/elif/else (v1)".into()))?;
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
    let branch_real = |args: &[Expr]| -> Result<String> {
        let mut real = Vec::new();
        for a in args {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be parameters (v1)".into())) };
            if !params.iter().any(|(p, _)| p == n) {
                return Err(fail(format!("await argument `{}` must be a parameter (v1)", n.as_str())));
            }
            real.push(format!("self.{}", n.as_str()));
        }
        Ok(real.join(", "))
    };
    let mut branch_ph = Vec::new();
    let mut branch_re = Vec::new();
    for (j, b) in branches.iter().enumerate() {
        fields.push(format!("__{}_Fut __w{}", b.g.as_str(), j));
        let mut ph = Vec::new();
        for a in &b.args {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be parameters (v1)".into())) };
            if !params.iter().any(|(p, _)| p == n) {
                return Err(fail(format!("await argument `{}` must be a parameter (v1)", n.as_str())));
            }
            ph.push(n.as_str().to_string());
        }
        inits.push(format!("__w{} = {}({})", j, b.g.as_str(), ph.join(", ")));
        branch_ph.push(ph);
        branch_re.push(branch_real(&b.args)?);
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
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let args = self_args.join(", ");
    let mut after_args = if args.is_empty() { "self.y".to_string() } else { format!("{}, self.y", args) };
    for p in &posts {
        after_args.push_str(&format!(", self.{}", p.target.as_str()));
    }
    let branch_count = branches.len();
    let merge = branch_count + 1;
    let final_state = merge + 1 + posts.len();
    let mut poll = String::new();
    let mut dispatch = String::new();
    let mut take_fns = Vec::new();
    for (j, b) in branches.iter().enumerate() {
        let enter = format!("self.__w{} = {}({}); self._state = {}", j, b.g.as_str(), branch_re[j], j + 1);
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
            let kw = if dispatch.is_empty() { "if" } else { "elif" };
            dispatch.push_str(&format!("        {kw} {tn}({args}) {{ {enter} }}\n"));
        } else {
            dispatch.push_str(&format!("        else {{ {enter} }}\n"));
        }
    }
    if !branches.iter().any(|b| b.cond.is_none()) {
        dispatch.push_str(&format!("        else {{ self._state = {merge} }}\n"));
    }
    poll.push_str(&format!(
        "    if self._state == 0 {{\n{dispatch}    }}\n"));
    for j in 0..branch_count {
        let state = j + 1;
        poll.push_str(&format!(
            "    if self._state == {state} {{\n\
             \x20       p{state} = self.__w{j}.poll(ref cx)\n\
             \x20       match p{state} {{\n\
             \x20           Ready(v) => {{ self.y = v; self._state = {merge} }}\n\
             \x20           Pending => {{ return Poll::Pending() }}\n\
             \x20       }}\n\
             \x20   }}\n"));
    }
    if let Some(first_post) = posts.first() {
        poll.push_str(&format!(
            "    if self._state == {merge} {{ self.__p0 = {}({}); self._state = {} }}\n",
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
            "    if self._state == {state} {{\n\
             \x20       pp{i} = self.__p{i}.poll(ref cx)\n\
             \x20       match pp{i} {{\n\
             \x20           Ready(v) => {{ self.{} = v; {ready} }}\n\
             \x20           Pending => {{ return Poll::Pending() }}\n\
             \x20       }}\n\
             \x20   }}\n",
            p.target.as_str()));
    }
    poll.push_str(&format!("    return Poll::Ready({after}({after_args}))\n"));
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
    let mut after_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: an, params: ap, body: ab, vis: av, .. } = &mut after_fn {
        attrs.clear();
        *an = Symbol::intern(&after);
        *av = Visibility::Private;
        ap.push((y, (*first).clone()));
        for p in &posts {
            ap.push((p.target, p.ret.clone()));
        }
        let mut b = ab.clone();
        b.stmts = tail;
        *ab = b;
    }
    take_fns.push(after_fn);
    out.extend(generated.stmts);
    for helper in take_fns {
        let hs = crate::formatter::format_program(&Program::new(vec![helper]));
        let h = crate::parser::parse_source(&hs)
            .map_err(|e| fail(format!("helper format failed: {}", e)))?;
        out.extend(h.stmts);
    }
    Ok(true)
}

/// 匹配 `y=默认; if c0 { y=await g0 } elif ... [else ...]; (z_i=await h_i)+; tail`（至少一个 elif）。
#[allow(clippy::type_complexity)]
fn detect(body: &Block) -> Option<(Symbol, Vec<Branch>, Vec<(Symbol, Symbol, Vec<Expr>)>, Vec<Stmt>)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block, .. }, ..] =
        body.stmts.as_slice() else { return None };
    if elifs.is_empty() { return None; }
    let one = |b: &Block| -> Option<(Symbol, Vec<Expr>)> {
        let [Stmt::Assign { name, value, .. }] = b.stmts.as_slice() else { return None };
        if name != pre { return None; }
        let Expr::FnCall { name: an, args, .. } = value else { return None };
        if an.as_str() != "__await" { return None; }
        let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
        Some((*g, gargs.clone()))
    };
    let mut branches = Vec::new();
    let (g0, a0) = one(then_block)?;
    branches.push(Branch { cond: Some(cond.clone()), g: g0, args: a0 });
    for (c, b) in elifs {
        let (g, a) = one(b)?;
        branches.push(Branch { cond: Some(c.clone()), g, args: a });
    }
    if let Some(b) = else_block {
        let (g, a) = one(b)?;
        branches.push(Branch { cond: None, g, args: a });
    }
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
    Some((*pre, branches, raws, tail))
}
