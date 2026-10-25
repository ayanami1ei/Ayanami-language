//! A1b-2：if/elif(/else) 分支内连续多个 await（至少一个分支 ≥2；单 await 走 async_elif）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

struct Branch {
    cond: Option<Expr>, // None = else
    awaits: Vec<(Symbol, Vec<Expr>)>,
}

/// 尝试展开 elif 链多 await；不匹配或所有分支都只有 1 个时返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let Some((y, branches)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let first_g = branches.iter().find_map(|b| b.awaits.first().map(|(g, _)| *g))
        .ok_or_else(|| fail("elif chain has no await (v1)".into()))?;
    let first = returns.get(&first_g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", first_g.as_str())))?;
    let first_src = ty_src(first).ok_or_else(|| fail("unsupported await value type (v1)".into()))?;
    for b in &branches {
        for (g, _) in &b.awaits {
            let r = returns.get(g).ok_or_else(|| fail(format!(
                "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
            if ty_src(r).as_deref() != Some(first_src.as_str()) {
                return Err(fail("await values in if/elif/else branches must have the same type (v1)".into()));
            }
        }
    }
    let (y_ty, y_init) = await_default(first).ok_or_else(|| fail(
        "unsupported await value type in elif chain (v1)".into()))?;
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
    let mut real_args: Vec<Vec<String>> = Vec::with_capacity(branches.len());
    for (j, b) in branches.iter().enumerate() {
        let mut reals = Vec::with_capacity(b.awaits.len());
        for (i, (g, gargs)) in b.awaits.iter().enumerate() {
            let (ph, real) = build_args(gargs)?;
            fields.push(format!("__{}_Fut __w{}_{}", g.as_str(), j, i));
            inits.push(format!("__w{}_{} = {}({})", j, i, g.as_str(), ph));
            reals.push(real);
        }
        real_args.push(reals);
    }
    fields.push(format!("{} y", y_ty));
    fields.push("int _state".to_string());
    inits.push(format!("y = {}", y_init));
    inits.push("_state = 0".to_string());
    let fut = format!("__{}_Fut", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let sa = self_args.join(", ");
    let after_args = if sa.is_empty() { "self.y".to_string() } else { format!("{}, self.y", sa) };
    let mut starts = Vec::with_capacity(branches.len());
    let mut exit = 1usize;
    for b in &branches {
        starts.push(exit);
        exit += b.awaits.len();
    }
    let spawn = |j: usize, i: usize| format!(
        "self.__w{}_{} = {}({}); self._state = {}",
        j, i, branches[j].awaits[i].0.as_str(), real_args[j][i], starts[j] + i);
    let mut dispatch = String::new();
    let mut take_fns = Vec::new();
    for (j, b) in branches.iter().enumerate() {
        let enter = if b.awaits.is_empty() {
            format!("self._state = {}", exit)
        } else {
            spawn(j, 0)
        };
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
            let kw = if j == 0 { "if" } else { "elif" };
            dispatch.push_str(&format!("        {kw} {tn}({sa}) {{ {enter} }}\n"));
        } else {
            dispatch.push_str(&format!("        else {{ {enter} }}\n"));
        }
    }
    if !branches.iter().any(|b| b.cond.is_none()) {
        dispatch.push_str(&format!("        else {{ self._state = {} }}\n", exit));
    }
    let mut polls = String::new();
    for (j, b) in branches.iter().enumerate() {
        for i in 0..b.awaits.len() {
            let state = starts[j] + i;
            let ready = if i + 1 < b.awaits.len() {
                spawn(j, i + 1)
            } else {
                format!("self._state = {}", exit)
            };
            polls.push_str(&format!(
                "        if self._state == {state} {{\n\
                 \x20           p{state} = self.__w{j}_{i}.poll(ref cx)\n\
                 \x20           match p{state} {{\n\
                 \x20               Ready(v) => {{ self.y = v\n\
                 \x20                   {ready} }}\n\
                 \x20               Pending => {{ return Poll::Pending() }}\n\
                 \x20           }}\n\
                 \x20       }}\n"
            ));
        }
    }
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._state == 0 {{\n{dispatch}\x20       }}\n{polls}\
         \x20       return Poll::Ready({after}({after_args}))\n    }}\n}}\n",
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

/// 匹配 `y=<默认>; if c0 { (y=await...)+ } elif c1 {...} [else {...}]; ...`（至少一个 elif）。
fn detect(body: &Block) -> Option<(Symbol, Vec<Branch>)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block, .. }, ..] = body.stmts.as_slice() else { return None };
    if elifs.is_empty() { return None; }
    let y = first_target(then_block)?;
    if *pre != y { return None; }
    let mut branches = vec![Branch { cond: Some(cond.clone()), awaits: chain(then_block, &y)? }];
    for (c, b) in elifs {
        branches.push(Branch { cond: Some(c.clone()), awaits: chain(b, &y)? });
    }
    if let Some(b) = else_block {
        branches.push(Branch { cond: None, awaits: chain(b, &y)? });
    }
    if branches.iter().all(|b| b.awaits.len() < 2) { return None; }
    Some((y, branches))
}

fn first_target(b: &Block) -> Option<Symbol> {
    let st = b.stmts.first()?;
    let Stmt::Assign { name, .. } = st else { return None };
    Some(*name)
}

fn chain(b: &Block, y: &Symbol) -> Option<Vec<(Symbol, Vec<Expr>)>> {
    let mut out = Vec::new();
    for st in &b.stmts {
        let Stmt::Assign { name, value, .. } = st else { return None };
        if name != y { return None; }
        let Expr::FnCall { name: an, args, .. } = value else { return None };
        if an.as_str() != "__await" { return None; }
        let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
        out.push((*g, gargs.clone()));
    }
    if out.is_empty() { None } else { Some(out) }
}
