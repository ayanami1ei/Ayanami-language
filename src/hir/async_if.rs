//! A1b-2：单个 if 分支内的 await（y=默认; if cond { y = await g(args) }; post）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

/// 尝试展开 if 分支 await；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    // 先尝试多 await 分支（if/else 任一侧 ≥2，或单分支 ≥2）；单个继续走本模块
    if crate::hir::async_if_else_multi::try_expand(stmt, returns, out)? { return Ok(true); }
    if crate::hir::async_if_multi::try_expand(stmt, returns, out)? { return Ok(true); }
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    if let Some(shape) = detect_else(body) {
        return expand_else(stmt, shape, returns, out);
    }
    let Some((y, g, gargs, cond)) = detect(body) else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let g_ret = returns.get(&g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
    let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| fail(
        "unsupported await value type in if-branch (v1)".into()))?;
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
    let mut arg_self = Vec::new();
    for a in &gargs {
        let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be parameters (v1)".into())) };
        if !params.iter().any(|(p, _)| p == n) {
            return Err(fail(format!("await argument `{}` must be a parameter (v1)", n.as_str())));
        }
        arg_self.push(format!("self.{}", n.as_str()));
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
        inits.join(", "), g.as_str(), pnames.join(", "), y_init);
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._state == 0 {{\n\
         \x20           if {take}({p_self}) {{ self.__w = {g}({arg_self}); self._state = 1 }} else {{ self._state = 2 }}\n\
         \x20       }}\n\
         \x20       if self._state == 1 {{\n\
         \x20           p1 = self.__w.poll(ref cx)\n\
         \x20           match p1 {{\n\
         \x20               Ready(v) => {{ self.y = v; self._state = 2 }}\n\
         \x20               Pending => {{ return Poll::Pending() }}\n\
         \x20           }}\n\
         \x20       }}\n\
         \x20       return Poll::Ready({after}({args}))\n    }}\n}}\n",
        plist = plist.join(", "), name = name.as_str(), ret = ret,
        p_self = self_args.join(", "), arg_self = arg_self.join(", "),
        args = format!("{}, self.y", self_args.join(", ")),
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    // take helper：原参数 + 条件
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
    // after helper：原参数 + y + 后置语句
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

/// 匹配 `y = <默认>; if cond { y = await g(args) }; ...`。
fn detect(body: &Block) -> Option<(Symbol, Symbol, Vec<Expr>, Expr)> {
    let [Stmt::Assign { name: pre, .. }, Stmt::If { cond, then_block, elifs, else_block, .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() || else_block.is_some() { return None; }
    let [Stmt::Assign { name: y, value, .. }] = then_block.stmts.as_slice() else { return None };
    if pre != y { return None; }
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Some(Expr::FnCall { name: g, args: gargs, .. }) = args.first() else { return None };
    Some((*y, *g, gargs.clone(), cond.clone()))
}


/// 匹配 `y = <默认>; if cond { y = await g1(a1) } else { y = await g2(a2) }; ...`
fn detect_else(body: &Block) -> Option<(Symbol, Symbol, Vec<Expr>, Symbol, Vec<Expr>, Expr)> {
    let [Stmt::Assign { name: pre, .. },
         Stmt::If { cond, then_block, elifs, else_block: Some(eb), .. }, ..] =
        body.stmts.as_slice() else { return None };
    if !elifs.is_empty() { return None; }
    let [Stmt::Assign { name: y1, value: v1, .. }] = then_block.stmts.as_slice() else { return None };
    let [Stmt::Assign { name: y2, value: v2, .. }] = eb.stmts.as_slice() else { return None };
    if pre != y1 || y1 != y2 { return None; }
    let (g1, a1) = awaited(v1)?;
    let (g2, a2) = awaited(v2)?;
    Some((*y1, g1, a1, g2, a2, cond.clone()))
}

fn awaited(e: &Expr) -> Option<(Symbol, Vec<Expr>)> {
    let Expr::FnCall { name: an, args, .. } = e else { return None };
    if an.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    Some((*g, gargs.clone()))
}

/// if/else 双分支：两个 future 槽，Ready 后都进入 state3（合并）。
fn expand_else(
    stmt: &Stmt,
    shape: (Symbol, Symbol, Vec<Expr>, Symbol, Vec<Expr>, Expr),
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<bool> {
    let (y, g1, a1, g2, a2, cond) = shape;
    let Stmt::FnDecl { name, params, body, .. } = stmt else { unreachable!() };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let r1 = returns.get(&g1).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g1.as_str())))?;
    let r2 = returns.get(&g2).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g2.as_str())))?;
    if ty_src(r1) != ty_src(r2) {
        return Err(fail("if/else await values must have the same type (v1)".into()));
    }
    let (y_ty, y_init) = await_default(r1).ok_or_else(|| fail(
        "unsupported await value type (v1)".into()))?;
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
    let args = |list: &[Expr]| -> Result<(String, String)> {
        let (mut names, mut reals) = (Vec::new(), Vec::new());
        for a in list {
            let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be parameters (v1)".into())) };
            if !params.iter().any(|(p, _)| p == n) {
                return Err(fail(format!("await argument `{}` must be a parameter (v1)", n.as_str())));
            }
            names.push(n.as_str().to_string());
            reals.push(format!("self.{}", n.as_str()));
        }
        Ok((names.join(", "), reals.join(", ")))
    };
    let (a1n, a1s) = args(&a1)?;
    let (a2n, a2s) = args(&a2)?;
    let fut = format!("__{}_Fut", name.as_str());
    let take = format!("__async_take_{}", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let fields = format!(
        "{}\n    __{}_Fut __wt\n    __{}_Fut __we\n    {} y\n    int _state",
        fields.join("\n    "), g1.as_str(), g2.as_str(), y_ty);
    let inits = format!(
        "{}, __wt = {}({}), __we = {}({}), y = {}, _state = 0",
        inits.join(", "), g1.as_str(), a1n, g2.as_str(), a2n, y_init);
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._state == 0 {{\n\
         \x20           if {take}({sa}) {{ self.__wt = {g1}({a1s}); self._state = 1 }} else {{ self.__we = {g2}({a2s}); self._state = 2 }}\n\
         \x20       }}\n\
         \x20       if self._state == 1 {{\n\
         \x20           p1 = self.__wt.poll(ref cx)\n\
         \x20           match p1 {{\n\
         \x20               Ready(v) => {{ self.y = v; self._state = 3 }}\n\
         \x20               Pending => {{ return Poll::Pending() }}\n\
         \x20           }}\n\
         \x20       }}\n\
         \x20       if self._state == 2 {{\n\
         \x20           p2 = self.__we.poll(ref cx)\n\
         \x20           match p2 {{\n\
         \x20               Ready(v) => {{ self.y = v; self._state = 3 }}\n\
         \x20               Pending => {{ return Poll::Pending() }}\n\
         \x20           }}\n\
         \x20       }}\n\
         \x20       return Poll::Ready({after}({sa}, self.y))\n    }}\n}}\n",
        plist = plist.join(", "), name = name.as_str(), ret = ret,
        sa = self_args.join(", "),
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
        ap.push((y, r1.clone()));
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
