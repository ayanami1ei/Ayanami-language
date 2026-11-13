//! A1b-2：while 循环体内的连续 await（回边到 state0，条件每轮重估）；
//! 支持 await 前后 `if cond { break/continue }`（条件限参数/循环变量）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::hir::async_while_util::{await_parts, ctrl_helper, detect, Shape};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Expr, Program, Stmt, Type};

struct PostAwait {
    target: Symbol,
    g: Symbol,
    placeholders: Vec<String>,
    reals: Vec<String>,
    ret: Type,
    ty: String,
    init: String,
}

/// 尝试展开 while 内 await；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some(shape) = detect(body) else { return Ok(false) };
    let Shape { y, awaits, cond, pre, post } = shape;
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let mut g_rets = Vec::with_capacity(awaits.len());
    for (g, _) in &awaits {
        g_rets.push(returns.get(g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", g.as_str())))?);
    }
    let first = g_rets[0];
    let first_src = ty_src(first).ok_or_else(|| fail(
        "unsupported await value type in while-loop (v1)".into()))?;
    for r in &g_rets[1..] {
        if ty_src(r).as_deref() != Some(first_src.as_str()) {
            return Err(fail("await values in loop body must have the same type (v1)".into()));
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
        "unsupported await value type in while-loop (v1)".into()))?;
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
    // 每个 await 一个 future 字段；构造器占位（后续状态进入时用真实 y 重建）
    let mut real_args = Vec::with_capacity(awaits.len());
    for (i, (g, gargs)) in awaits.iter().enumerate() {
        let mut placeholder = Vec::new();
        let mut real = Vec::new();
        for a in gargs {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
            if params.iter().any(|(p, _)| p == n) {
                placeholder.push(n.as_str().to_string());
                real.push(format!("self.{}", n.as_str()));
            } else if *n == y {
                placeholder.push(y_init.to_string());
                real.push("self.y".to_string());
            } else {
                return Err(fail(format!(
                    "await argument `{}` must be a parameter or the loop variable (v1)", n.as_str())));
            }
        }
        fields.push(format!("__{}_Fut __w{}", g.as_str(), i));
        inits.push(format!("__w{} = {}({})", i, g.as_str(), placeholder.join(", ")));
        real_args.push(real.join(", "));
    }
    // 循环后的连续 await：z_i = await h_i(args)；args 可引用参数/y/更早的 post 结果
    let rest = &body.stmts[2..];
    let mut posts: Vec<PostAwait> = Vec::new();
    let mut idx = 0;
    while idx < rest.len() {
        let Stmt::Assign { name: z, value, .. } = &rest[idx] else { break };
        let Some((h, hargs)) = await_parts(value) else { break };
        let Some(h_ret) = returns.get(&h).cloned() else { return Ok(false) };
        let Some((z_ts, z_init)) = await_default(&h_ret) else { return Ok(false) };
        if *z == y || params.iter().any(|(p, _)| p == z) || posts.iter().any(|p| p.target == *z) {
            return Ok(false);
        }
        let prior: Vec<(Symbol, Type)> = posts.iter().map(|p| (p.target, p.ret.clone())).collect();
        let (placeholders, reals) = crate::hir::async_post_args::build_post_args(
            &hargs[..], params, y, first, &prior,
        ).ok_or_else(|| fail("unsupported post await argument (v1)".into()))?;
        posts.push(PostAwait {
            target: *z,
            g: h,
            placeholders,
            reals,
            ret: h_ret,
            ty: z_ts.to_string(),
            init: z_init.to_string(),
        });
        idx += 1;
    }
    let tail: Vec<Stmt> = rest[idx..].to_vec();
    if !posts.is_empty() {
        if tail.is_empty() { return Ok(false); }
        for st in &tail {
            if let Stmt::Assign { value, .. } = st {
                if await_parts(value).is_some() { return Ok(false); }
            }
        }
    }
    for (i, pa) in posts.iter().enumerate() {
        fields.push(format!("__{}_Fut __p{}", pa.g.as_str(), i));
        fields.push(format!("{} {}", pa.ty, pa.target.as_str()));
        inits.push(format!("__p{} = {}({})", i, pa.g.as_str(), pa.placeholders.join(", ")));
        inits.push(format!("{} = {}", pa.target.as_str(), pa.init));
    }
    fields.push(format!("{} y", y_ty));
    fields.push("int _state".to_string());
    inits.push(format!("y = {}", y_init));
    inits.push("_state = 0".to_string());
    let fut = format!("__{}_Fut", name.as_str());
    let take = format!("__async_take_{}", name.as_str());
    let pre_ctl = format!("__async_pre_{}", name.as_str());
    let post_ctl = format!("__async_post_{}", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let helper_args = if self_args.is_empty() {
        "self.y".to_string()
    } else {
        format!("{}, self.y", self_args.join(", "))
    };
    let exit = awaits.len() + 1;
    let spawn = |i: usize| format!(
        "self.__w{} = {}({}); self._state = {}",
        i, awaits[i].0.as_str(), real_args[i], i + 1);
    let zero_block = if pre.is_empty() {
        spawn(0)
    } else {
        format!(
            "__c = {}({})\n\
             if __c == 1 {{ self._state = {} }} elif __c == 2 {{ continue }} \
             else {{ {} }}",
            pre_ctl, helper_args, exit, spawn(0))
    };
    let post_block = if post.is_empty() {
        "self._state = 0".to_string()
    } else {
        format!(
            "__c2 = {}({})\n\
             if __c2 == 1 {{ self._state = {} }} else {{ self._state = 0 }}",
            post_ctl, helper_args, exit)
    };
    let mut poll = String::new();
    poll.push_str("while true {\n");
    poll.push_str(&format!(
        "    if self._state == 0 {{\n\
         \x20       if {take}({helper_args}) {{\n\
         \x20           {zero_block}\n\
         \x20       }} else {{ self._state = {exit} }}\n\
         \x20   }}\n"));
    for (i, _) in awaits.iter().enumerate() {
        let state = i + 1;
        let ready = if state < awaits.len() { spawn(state) } else { post_block.clone() };
        poll.push_str(&format!(
            "    if self._state == {state} {{\n\
             \x20       p{state} = self.__w{i}.poll(ref cx)\n\
             \x20       match p{state} {{\n\
             \x20           Ready(v) => {{ self.y = v\n\
             \x20               {ready} }}\n\
             \x20           Pending => {{ return Poll::Pending() }}\n\
             \x20       }}\n\
             \x20   }}\n"));
    }
    let mut after_args = helper_args.clone();
    for pa in &posts {
        after_args.push_str(&format!(", self.{}", pa.target.as_str()));
    }
    if posts.is_empty() {
        poll.push_str(&format!(
            "    if self._state == {exit} {{ return Poll::Ready({after}({after_args})) }}\n"));
    } else {
        let first_post = &posts[0];
        poll.push_str(&format!(
            "    if self._state == {exit} {{ self.__p0 = {}({}); self._state = {} }}\n",
            first_post.g.as_str(), first_post.reals.join(", "), exit + 1));
        let final_state = exit + posts.len() + 1;
        for (i, pa) in posts.iter().enumerate() {
            let poll_state = exit + 1 + i;
            let ready = if i + 1 < posts.len() {
                let next = &posts[i + 1];
                format!("self.__p{} = {}({}); self._state = {}",
                    i + 1, next.g.as_str(), next.reals.join(", "), poll_state + 1)
            } else {
                format!("self._state = {}", final_state)
            };
            poll.push_str(&format!(
                "    if self._state == {poll_state} {{\n\
                 \x20       pp{i} = self.__p{i}.poll(ref cx)\n\
                 \x20       match pp{i} {{ Ready(v) => {{ self.{} = v; {ready} }} Pending => {{ return Poll::Pending() }} }}\n\
                 \x20   }}\n",
                pa.target.as_str()));
        }
        poll.push_str(&format!(
            "    if self._state == {final_state} {{ return Poll::Ready({after}({after_args})) }}\n"));
    }
    poll.push_str("}\nreturn Poll::Pending()\n");
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
    // take helper：参数 + y → bool（每轮重估条件）
    let mut take_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: tn, params: tp, return_type: tr, body: tb, vis: tv, .. } = &mut take_fn {
        attrs.clear();
        *tn = Symbol::intern(&take);
        *tr = Type::Bool(*span);
        *tv = Visibility::Private;
        tp.push((y, (*first).clone()));
        let mut b = tb.clone();
        b.stmts = vec![Stmt::Return { value: Some(cond), span: *span }];
        *tb = b;
    }
    let mut helpers = vec![take_fn];
    if !pre.is_empty() {
        helpers.push(ctrl_helper(stmt, &pre_ctl, y, first, &pre, *span));
    }
    if !post.is_empty() {
        helpers.push(ctrl_helper(stmt, &post_ctl, y, first, &post, *span));
    }
    // after helper：参数 + y + 后置语句
    let mut after_fn = stmt.clone();
    if let Stmt::FnDecl { attrs, name: an, params: ap, body: ab, vis: av, .. } = &mut after_fn {
        attrs.clear();
        *an = Symbol::intern(&after);
        *av = Visibility::Private;
        ap.push((y, (*first).clone()));
        for pa in &posts {
            ap.push((pa.target, pa.ret.clone()));
        }
        let mut b = ab.clone();
        b.stmts = tail;
        *ab = b;
    }
    helpers.push(after_fn);
    out.extend(generated.stmts);
    for helper in helpers {
        let hs = crate::formatter::format_program(&Program::new(vec![helper]));
        let h = crate::parser::parse_source(&hs)
            .map_err(|e| fail(format!("helper format failed: {}", e)))?;
        out.extend(h.stmts);
    }
    Ok(true)
}
