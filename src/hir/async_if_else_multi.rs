//! A1b-2：if/else 分支内连续多个 await（至少一侧 ≥2；单 await 仍走 async_if）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Shape {
    y: Symbol,
    then_awaits: Vec<(Symbol, Vec<Expr>)>,
    else_awaits: Vec<(Symbol, Vec<Expr>)>,
    cond: Expr,
}

/// 尝试展开 if/else 多 await；不匹配或两侧都只有 1 个时返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some(shape) = detect(body) else { return Ok(false) };
    let Shape { y, then_awaits, else_awaits, cond } = shape;
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let first = returns.get(&then_awaits[0].0).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", then_awaits[0].0.as_str())))?;
    let first_src = ty_src(first).ok_or_else(|| fail(
        "unsupported await value type in if/else branch (v1)".into()))?;
    for (g, _) in then_awaits.iter().chain(else_awaits.iter()) {
        let r = returns.get(g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
        if ty_src(r).as_deref() != Some(first_src.as_str()) {
            return Err(fail("await values in if/else branches must have the same type (v1)".into()));
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
        "unsupported await value type in if/else branch (v1)".into()))?;
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
    let build_args = |gargs: &[Expr]| -> Result<(String, String)> {
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
        Ok((placeholder.join(", "), real.join(", ")))
    };
    let mut then_real = Vec::with_capacity(then_awaits.len());
    for (i, (g, gargs)) in then_awaits.iter().enumerate() {
        let (ph, real) = build_args(gargs)?;
        fields.push(format!("__{}_Fut __wt{}", g.as_str(), i));
        inits.push(format!("__wt{} = {}({})", i, g.as_str(), ph));
        then_real.push(real);
    }
    let mut else_real = Vec::with_capacity(else_awaits.len());
    for (i, (g, gargs)) in else_awaits.iter().enumerate() {
        let (ph, real) = build_args(gargs)?;
        fields.push(format!("__{}_Fut __we{}", g.as_str(), i));
        inits.push(format!("__we{} = {}({})", i, g.as_str(), ph));
        else_real.push(real);
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
    let then_n = then_awaits.len();
    let else_n = else_awaits.len();
    let else_start = then_n + 1;
    let exit = then_n + else_n + 1;
    let spawn_t = |i: usize| format!(
        "self.__wt{} = {}({}); self._state = {}",
        i, then_awaits[i].0.as_str(), then_real[i], i + 1);
    let spawn_e = |i: usize| format!(
        "self.__we{} = {}({}); self._state = {}",
        i, else_awaits[i].0.as_str(), else_real[i], else_start + i);
    let mut poll = String::new();
    poll.push_str(&format!(
        "if self._state == 0 {{\n\
         \x20   if {take}({args}) {{\n\
         \x20       {}\n\
         \x20   }} else {{\n\
         \x20       {}\n\
         \x20   }}\n}}\n", spawn_t(0), spawn_e(0)));
    for i in 0..then_n {
        let state = i + 1;
        let ready = if i + 1 < then_n { spawn_t(i + 1) } else { format!("self._state = {}", exit) };
        poll.push_str(&format!(
            "if self._state == {state} {{\n\
             \x20   pt{state} = self.__wt{i}.poll(ref cx)\n\
             \x20   match pt{state} {{\n\
             \x20       Ready(v) => {{ self.y = v\n\
             \x20           {ready} }}\n\
             \x20       Pending => {{ return Poll::Pending() }}\n\
             \x20   }}\n}}\n"));
    }
    for j in 0..else_n {
        let state = else_start + j;
        let ready = if j + 1 < else_n { spawn_e(j + 1) } else { format!("self._state = {}", exit) };
        poll.push_str(&format!(
            "if self._state == {state} {{\n\
             \x20   pe{state} = self.__we{j}.poll(ref cx)\n\
             \x20   match pe{state} {{\n\
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

/// 匹配 `y = <默认>; if cond { (y = await g(args))+ } else { (y = await h(args))+ }; ...`。
fn detect(body: &Block) -> Option<Shape> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block: Some(eb), .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() { return None; }
    let then_awaits = await_chain(&then_block.stmts, pre)?;
    let else_awaits = await_chain(&eb.stmts, pre)?;
    if then_awaits.len() < 2 && else_awaits.len() < 2 { return None; }
    Some(Shape { y: *pre, then_awaits, else_awaits, cond: cond.clone() })
}

fn await_chain(stmts: &[Stmt], y: &Symbol) -> Option<Vec<(Symbol, Vec<Expr>)>> {
    let mut out = Vec::new();
    for st in stmts {
        let Stmt::Assign { name, value, .. } = st else { return None };
        if name != y { return None; }
        let Expr::FnCall { name: an, args, .. } = value else { return None };
        if an.as_str() != "__await" { return None; }
        let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
        out.push((*g, gargs.clone()));
    }
    if out.is_empty() { None } else { Some(out) }
}
