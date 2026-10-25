//! A1b-2：while 循环体内的连续 await（回边到 state0，条件每轮重估）；
//! 支持 await 前后 `if cond { break/continue }`（条件限参数/循环变量）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::hir::async_while_util::{ctrl_helper, Ctrl};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Shape {
    y: Symbol,
    /// 连续的 (async 函数, 实参) 序列，全部赋值给 `y`。
    awaits: Vec<(Symbol, Vec<Expr>)>,
    cond: Expr,
    pre: Vec<(Expr, Ctrl)>,
    post: Vec<(Expr, Ctrl)>,
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
    poll.push_str(&format!(
        "    if self._state == {exit} {{ return Poll::Ready({after}({helper_args})) }}\n}}\n\
         return Poll::Pending()\n"));
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
        let mut b = ab.clone();
        b.stmts = body.stmts[2..].to_vec();
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

/// 匹配 `y = <默认>; while cond { [if c { break/continue }] (y = await g(args))+ [if c { break/continue }] }; ...`。
fn detect(body: &Block) -> Option<Shape> {
    let [Stmt::Assign { name: pre, .. }, Stmt::While { cond, body: wbody, .. }, ..] =
        body.stmts.as_slice() else { return None };
    let mut phase = 0u8; // 0=await 前，1=await 序列，2=await 后
    let mut start = None;
    let mut end = 0;
    let mut awaits = Vec::new();
    for (i, st) in wbody.stmts.iter().enumerate() {
        if let Some((name, g, gargs)) = awaited_assignment(st) {
            if phase == 2 { return None; }
            if start.is_none() {
                if pre != &name { return None; }
                start = Some(i);
            }
            if name != *pre { return None; }
            awaits.push((g, gargs));
            end = i + 1;
            phase = 1;
        } else if phase == 1 {
            phase = 2;
        }
    }
    let start = start?;
    let mut pre_ctrl = Vec::new();
    for st in &wbody.stmts[..start] {
        pre_ctrl.push(ctrl_of(st)?);
    }
    let mut post_ctrl = Vec::new();
    for st in &wbody.stmts[end..] {
        post_ctrl.push(ctrl_of(st)?);
    }
    Some(Shape {
        y: *pre,
        awaits,
        cond: cond.clone(),
        pre: pre_ctrl,
        post: post_ctrl,
    })
}

fn awaited_assignment(st: &Stmt) -> Option<(Symbol, Symbol, Vec<Expr>)> {
    let Stmt::Assign { name, value, .. } = st else { return None };
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    Some((*name, *g, gargs.clone()))
}

fn ctrl_of(st: &Stmt) -> Option<(Expr, Ctrl)> {
    let Stmt::If { cond, then_block, elifs, else_block: None, .. } = st else { return None };
    if !elifs.is_empty() { return None; }
    match then_block.stmts.as_slice() {
        [Stmt::Break { .. }] => Some((cond.clone(), Ctrl::Break)),
        [Stmt::Continue { .. }] => Some((cond.clone(), Ctrl::Continue)),
        _ => None,
    }
}
