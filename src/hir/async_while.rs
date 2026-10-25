//! A1b-2：while 循环体内的单个 await（回边到 state0，条件每轮重估）；
//! 另支持 await 前后 `if cond { break/continue }`（条件限参数/循环变量）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Literal, Program, Stmt, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctrl {
    Break,
    Continue,
}

struct Shape {
    y: Symbol,
    g: Symbol,
    gargs: Vec<Expr>,
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
    let Shape { y, g, gargs, cond, pre, post } = shape;
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let g_ret = returns.get(&g).ok_or_else(|| fail(format!(
        "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
    let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| fail(
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
    // 构造器占位实参：参数用名字、循环变量用默认值
    let mut placeholder = Vec::new();
    let mut real = Vec::new();
    for a in &gargs {
        let Expr::Ident(n, _) = a else { return Err(fail("await arguments must be identifiers (v1)".into())) };
        if params.iter().any(|(p, _)| p == n) {
            placeholder.push(n.as_str().to_string());
            real.push(format!("self.{}", n.as_str()));
        } else if *n == y {
            placeholder.push(y_init.to_string());
            real.push(format!("self.{}", n.as_str()));
        } else {
            return Err(fail(format!(
                "await argument `{}` must be a parameter or the loop variable (v1)", n.as_str())));
        }
    }
    let fut = format!("__{}_Fut", name.as_str());
    let take = format!("__async_take_{}", name.as_str());
    let pre_ctl = format!("__async_pre_{}", name.as_str());
    let post_ctl = format!("__async_post_{}", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    fields.push(format!("__{}_Fut __w", g.as_str()));
    fields.push(format!("{} y", y_ty));
    fields.push("int _state".to_string());
    inits.push(format!("__w = {}({})", g.as_str(), placeholder.join(", ")));
    inits.push(format!("y = {}", y_init));
    inits.push("_state = 0".to_string());
    let helper_args = if self_args.is_empty() {
        "self.y".to_string()
    } else {
        format!("{}, self.y", self_args.join(", "))
    };
    let pre_block = if pre.is_empty() {
        format!("self.__w = {}({}); self._state = 1", g.as_str(), real.join(", "))
    } else {
        format!(
            "__c = {}({})\n\
             if __c == 1 {{ self._state = 2 }} elif __c == 2 {{ continue }} \
             else {{ self.__w = {}({}); self._state = 1 }}",
            pre_ctl, helper_args, g.as_str(), real.join(", ")
        )
    };
    let post_block = if post.is_empty() {
        "self._state = 0".to_string()
    } else {
        format!(
            "__c2 = {}({})\n\
             if __c2 == 1 {{ self._state = 2 }} else {{ self._state = 0 }}",
            post_ctl, helper_args
        )
    };
    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       while true {{\n\
         \x20           if self._state == 0 {{\n\
         \x20               if {take}({helper_args}) {{\n\
         \x20                   {pre_block}\n\
         \x20               }} else {{ self._state = 2 }}\n\
         \x20           }}\n\
         \x20           if self._state == 1 {{\n\
         \x20               p1 = self.__w.poll(ref cx)\n\
         \x20               match p1 {{\n\
         \x20                   Ready(v) => {{ self.y = v\n\
         \x20                       {post_block} }}\n\
         \x20                   Pending => {{ return Poll::Pending() }}\n\
         \x20               }}\n\
         \x20           }}\n\
         \x20           if self._state == 2 {{ return Poll::Ready({after}({helper_args})) }}\n\
         \x20       }}\n\
         \x20       return Poll::Pending()\n    }}\n}}\n",
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
        tp.push((y, g_ret.clone()));
        let mut b = tb.clone();
        b.stmts = vec![Stmt::Return { value: Some(cond), span: *span }];
        *tb = b;
    }
    let mut helpers = vec![take_fn];
    if !pre.is_empty() {
        helpers.push(ctrl_helper(stmt, &pre_ctl, y, g_ret, &pre, *span));
    }
    if !post.is_empty() {
        helpers.push(ctrl_helper(stmt, &post_ctl, y, g_ret, &post, *span));
    }
    // after helper：参数 + y + 后置语句
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

/// 控制条件 helper：参数 + y → int（1=break，2=continue，0=继续执行）。
fn ctrl_helper(
    stmt: &Stmt,
    name: &str,
    y: Symbol,
    y_ty: &Type,
    controls: &[(Expr, Ctrl)],
    span: crate::span::Span,
) -> Stmt {
    let mut h = stmt.clone();
    if let Stmt::FnDecl { attrs, name: hn, params, return_type, body, vis, .. } = &mut h {
        attrs.clear();
        *hn = Symbol::intern(name);
        *vis = Visibility::Private;
        params.push((y, y_ty.clone()));
        *return_type = Type::Int(span);
        let mut stmts = Vec::new();
        for (cond, ctrl) in controls {
            let code = match ctrl {
                Ctrl::Break => 1,
                Ctrl::Continue => 2,
            };
            stmts.push(Stmt::If {
                cond: cond.clone(),
                then_block: Block::new(
                    vec![Stmt::Return {
                        value: Some(Expr::Literal(Literal::Int(code, span))),
                        span,
                    }],
                    span,
                ),
                elifs: Vec::new(),
                else_block: None,
                span,
            });
        }
        stmts.push(Stmt::Return {
            value: Some(Expr::Literal(Literal::Int(0, span))),
            span,
        });
        body.stmts = stmts;
        body.tail = None;
    }
    h
}

/// 匹配 `y = <默认>; while cond { [if c1 { break/continue }] y = await g(args) [if c2 { break/continue }] }; ...`。
fn detect(body: &Block) -> Option<Shape> {
    let [Stmt::Assign { name: pre, .. }, Stmt::While { cond, body: wbody, .. }, ..] =
        body.stmts.as_slice() else { return None };
    let mut await_idx = None;
    for (i, st) in wbody.stmts.iter().enumerate() {
        if let Stmt::Assign { value, .. } = st {
            if is_await(value) {
                if await_idx.is_some() { return None; }
                await_idx = Some(i);
            }
        }
    }
    let ai = await_idx?;
    let Stmt::Assign { name: y, value, .. } = &wbody.stmts[ai] else { return None };
    if pre != y { return None; }
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Some(Expr::FnCall { name: g, args: gargs, .. }) = args.first() else { return None };
    let mut pre_ctrl = Vec::new();
    for st in &wbody.stmts[..ai] {
        pre_ctrl.push(ctrl_of(st)?);
    }
    let mut post_ctrl = Vec::new();
    for st in &wbody.stmts[ai + 1..] {
        post_ctrl.push(ctrl_of(st)?);
    }
    Some(Shape {
        y: *y,
        g: *g,
        gargs: gargs.clone(),
        cond: cond.clone(),
        pre: pre_ctrl,
        post: post_ctrl,
    })
}

fn is_await(e: &Expr) -> bool {
    matches!(e, Expr::FnCall { name, .. } if name.as_str() == "__await")
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
