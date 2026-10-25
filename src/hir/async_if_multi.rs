//! A1b-2：if 分支内多个连续 await（y=默认; if cond { y=await g0; ... }; post）。
//! 单个 await 仍由 async_if.rs 处理；本模块只接收 2..N 个。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Shape {
    y: Symbol,
    awaits: Vec<(Symbol, Vec<Expr>)>,
    cond: Expr,
}

/// 尝试展开 if 分支内多个 await；不匹配或只有 1 个时返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some(shape) = detect(body) else { return Ok(false) };
    if shape.awaits.len() < 2 { return Ok(false); }
    let Shape { y, awaits, cond } = shape;
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let mut g_rets = Vec::with_capacity(awaits.len());
    for (g, _) in &awaits {
        g_rets.push(returns.get(g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", g.as_str())))?);
    }
    let first = g_rets[0];
    let first_src = ty_src(first).ok_or_else(|| fail(
        "unsupported await value type in if-branch (v1)".into()))?;
    for r in &g_rets[1..] {
        if ty_src(r).as_deref() != Some(first_src.as_str()) {
            return Err(fail("await values in if-branch must have the same type (v1)".into()));
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
        "unsupported await value type in if-branch (v1)".into()))?;
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
                    "await argument `{}` must be a parameter or the branch variable (v1)", n.as_str())));
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
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let args = self_args.join(", ");
    let after_args = if args.is_empty() { "self.y".to_string() } else { format!("{}, self.y", args) };
    let exit = awaits.len() + 1;
    let spawn = |i: usize| format!(
        "self.__w{} = {}({}); self._state = {}",
        i, awaits[i].0.as_str(), real_args[i], i + 1);
    let zero_block = format!(
        "self.__w0 = {}({}); self._state = 1", awaits[0].0.as_str(), real_args[0]);
    let mut poll = String::new();
    poll.push_str(&format!(
        "if self._state == 0 {{\n\
         \x20   if {take}({args}) {{\n\
         \x20       {zero_block}\n\
         \x20   }} else {{ self._state = {exit} }}\n}}\n"));
    for i in 0..awaits.len() {
        let state = i + 1;
        let ready = if state < awaits.len() { spawn(state) } else { format!("self._state = {}", exit) };
        poll.push_str(&format!(
            "if self._state == {state} {{\n\
             \x20   p{state} = self.__w{i}.poll(ref cx)\n\
             \x20   match p{state} {{\n\
             \x20       Ready(v) => {{ self.y = v\n\
             \x20           {ready} }}\n\
             \x20       Pending => {{ return Poll::Pending() }}\n\
             \x20   }}\n}}\n"));
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
    // take helper：条件只用参数（v1）
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
    out.extend(generated.stmts);
    for helper in [take_fn, after_fn] {
        let hs = crate::formatter::format_program(&Program::new(vec![helper]));
        let h = crate::parser::parse_source(&hs)
            .map_err(|e| fail(format!("helper format failed: {}", e)))?;
        out.extend(h.stmts);
    }
    Ok(true)
}

/// 匹配 `y = <默认>; if cond { (y = await g(args))+ }; ...`（无 else/elif）。
fn detect(body: &Block) -> Option<Shape> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block: None, .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() { return None; }
    let mut awaits = Vec::new();
    for st in &then_block.stmts {
        let Stmt::Assign { name, value, .. } = st else { return None };
        if name != pre { return None; }
        let Expr::FnCall { name: an, args, .. } = value else { return None };
        if an.as_str() != "__await" { return None; }
        let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
        awaits.push((*g, gargs.clone()));
    }
    if awaits.is_empty() { return None; }
    Some(Shape { y: *pre, awaits, cond: cond.clone() })
}
