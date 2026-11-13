//! A1b-2 直线 await 前缀（CFG 关闭时的回退路径）。
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::hir::async_transform_util::{await_default, ty_src, type_name};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};

pub(super) fn await_prefix(
    body: &Block,
    span: &crate::span::Span,
) -> Result<Vec<(Symbol, Symbol, Vec<Expr>)>> {
    let mut out = Vec::new();
    for st in &body.stmts {
        let Stmt::Assign { name, value, .. } = st else {
            break;
        };
        let Expr::FnCall { name: an, args, .. } = value else {
            break;
        };
        if an.as_str() != "__await" {
            break;
        }
        let Some(Expr::FnCall {
            name: g,
            args: gargs,
            ..
        }) = args.first()
        else {
            return Err(Error::Hir(format!(
                "#[async] v1: `await` target must be a direct function call (at {}:{})",
                span.start_line, span.start_col
            )));
        };
        out.push((*name, *g, gargs.clone()));
    }
    for st in body.stmts.iter().skip(out.len()) {
        if crate::hir::async_cfg::stmt_has_await(st) {
            return Err(Error::Hir(format!(
                "#[async] v1: `await` must be a straight-line prefix; awaits inside loops/branches are not supported (at {}:{})",
                span.start_line, span.start_col
            )));
        }
    }
    Ok(out)
}

/// N 个直线挂起点的状态机：构造器立即建全部内层 future，poll 依次轮询。
pub(super) fn expand_await(
    stmt: &Stmt,
    awaits: &[(Symbol, Symbol, Vec<Expr>)],
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
) -> Result<()> {
    let Stmt::FnDecl {
        vis,
        name,
        params,
        return_type,
        body,
        ..
    } = stmt
    else {
        unreachable!()
    };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let fut = format!("__{}_Fut", name.as_str());
    let after = format!("__async_after_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let mut plist = Vec::new();
    let mut fields = Vec::new();
    let mut inits = Vec::new();
    let mut self_args = Vec::new();
    for (n, t) in params {
        let ts = ty_src(t).ok_or_else(|| {
            fail(format!(
                "unsupported parameter type for `{}` (v1)",
                n.as_str()
            ))
        })?;
        plist.push(format!("{} {}", ts, n.as_str()));
        fields.push(format!("{} {}", ts, n.as_str()));
        inits.push(format!("{} = {}", n.as_str(), n.as_str()));
        self_args.push(format!("self.{}", n.as_str()));
    }
    let mut await_y: Vec<(Symbol, Type)> = Vec::new();
    let mut y_defaults: Vec<&'static str> = Vec::new();
    let mut polls = String::new();
    for (i, (y, g, gargs)) in awaits.iter().enumerate() {
        let g_ret = returns.get(g).ok_or_else(|| {
            fail(format!(
                "await target `{}` is not an #[async] function (v1)",
                g.as_str()
            ))
        })?;
        let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| {
            fail(format!(
                "unsupported await value type `{}` (v1)",
                type_name(g_ret)
            ))
        })?;
        let mut arg_names = Vec::with_capacity(gargs.len());
        for a in gargs {
            let Expr::Ident(n, _) = a else {
                return Err(fail("await call arguments must be identifiers (v1)".into()));
            };
            if params.iter().any(|(p, _)| p == n) {
                arg_names.push(n.as_str().to_string());
            } else if let Some(pos) = await_y.iter().position(|(v, _)| v == n) {
                // 占位：构造器用该 await 变量的默认值，真实值在进入本状态时重建 future
                arg_names.push(y_defaults[pos].to_string());
            } else {
                return Err(fail(format!(
                    "await call argument `{}` must be a parameter or an earlier await value (v1)",
                    n.as_str()
                )));
            }
        }
        fields.push(format!("__{}_Fut __w{}", g.as_str(), i));
        fields.push(format!("{} {}", y_ty, y.as_str()));
        inits.push(format!(
            "__w{} = {}({})",
            i,
            g.as_str(),
            arg_names.join(", ")
        ));
        inits.push(format!("{} = {}", y.as_str(), y_init));
        self_args.push(format!("self.{}", y.as_str()));
        await_y.push((*y, g_ret.clone()));
        y_defaults.push(y_init);
        let next = i + 1;
        let mut enter = String::new();
        if let Some((_, gnext, argsnext)) = awaits.get(next) {
            let mut real = Vec::with_capacity(argsnext.len());
            for a in argsnext {
                let Expr::Ident(n, _) = a else {
                    return Err(fail("await call arguments must be identifiers (v1)".into()));
                };
                if params.iter().any(|(p, _)| p == n) || await_y.iter().any(|(v, _)| v == n) {
                    real.push(format!("self.{}", n.as_str()));
                } else {
                    return Err(fail(format!(
                        "await call argument `{}` must be a parameter or earlier await value (v1)",
                        n.as_str()
                    )));
                }
            }
            enter = format!(
                "; self.__w{} = {}({})",
                next,
                gnext.as_str(),
                real.join(", ")
            );
        }
        polls.push_str(&format!(
            "        if self._state == {i} {{\n\
             \x20           p{i} = self.__w{i}.poll(ref cx)\n\
             \x20           match p{i} {{\n\
             \x20               Ready(v) => {{ self.{y} = v{enter}; self._state = {next} }}\n\
             \x20               Pending => {{ return Poll::Pending() }}\n\
             \x20           }}\n\
             \x20       }}\n",
            y = y.as_str()
        ));
    }
    let field_body = fields.join("\n    ");
    let init_body = inits.join(", ") + ", _state = 0";
    let src = format!(
        "{vis_s}struct {fut} {{\n    {field_body}\n    int _state\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {init_body} }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         {polls}\
         \x20       return Poll::Ready({after}({args}))\n    }}\n}}\n",
        plist = plist.join(", "),
        args = self_args.join(", "),
        name = name.as_str(),
        ret = ret,
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    out.extend(generated.stmts);
    let mut helper = stmt.clone();
    if let Stmt::FnDecl {
        attrs,
        name: hn,
        params: hp,
        body: hb,
        vis: hv,
        ..
    } = &mut helper
    {
        attrs.clear();
        *hn = Symbol::intern(&after);
        *hv = Visibility::Private;
        for (y, ty) in &await_y {
            hp.push((*y, ty.clone()));
        }
        let mut post = hb.clone();
        post.stmts = body.stmts[awaits.len()..].to_vec();
        *hb = post;
    }
    let hsrc = crate::formatter::format_program(&Program::new(vec![helper]));
    let h = crate::parser::parse_source(&hsrc)
        .map_err(|e| fail(format!("helper format failed: {}", e)))?;
    out.extend(h.stmts);
    Ok(())
}
