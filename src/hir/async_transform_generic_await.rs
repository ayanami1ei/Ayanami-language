//! A1c-2：泛型函数的最小单挂起 await 形态。
//!
//! 仅接受：`y = await target(args); return y`，target 为同程序非泛型 async 函数。
//! 构造器立即建立内层 future；poll 只转发其状态，因此无需为泛型参数制造默认值。
use super::async_transform_util::ty_src;
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::{Block, Expr, Stmt, Type};
use std::collections::{HashMap, HashSet};

pub(super) fn expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    generic_async_fns: &HashSet<Symbol>,
    out: &mut Vec<Stmt>,
) -> Result<()> {
    let Stmt::FnDecl {
        vis,
        name,
        generic_params,
        params,
        return_type,
        body,
        ..
    } = stmt
    else {
        unreachable!()
    };
    let fail = |msg: &str| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), msg));
    let (local, target, target_args) = match_single_await(body)
        .ok_or_else(|| fail("generic await supports only `y = await target(args); return y`"))?;
    if !returns.contains_key(&target) {
        return Err(fail("generic await target is not an async function"));
    }
    if generic_async_fns.contains(&target) {
        return Err(fail("generic await target is not supported in this slice"));
    }
    let target_ret = returns.get(&target).and_then(ty_src)
        .ok_or_else(|| fail("unsupported await target return type"))?;
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type"))?;
    if ret != target_ret {
        return Err(fail("return type must equal the awaited target return type"));
    }

    let names: Vec<String> = generic_params.iter().map(|(n, _)| n.as_str()).collect();
    let gargs = format!("[{}]", names.join(", "));
    let gdecl = generic_decl(generic_params);
    let fut = Symbol::intern(&format!("__{}_Fut", name.as_str()));
    let future_ty = format!("{}{}", fut.as_str(), gargs);
    let target_fut = format!("__{}_Fut", target.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };

    let mut ptypes = Vec::new();
    let mut fields = Vec::new();
    let mut inits = Vec::new();
    for (pn, pt) in params {
        if matches!(pn.as_str().as_str(), "__inner" | "_state" | "__p" | "__v") {
            return Err(fail("parameter name collides with generated async state"));
        }
        let ty = ty_src(pt).ok_or_else(|| fail("unsupported parameter type"))?;
        ptypes.push(format!("{} {}", ty, pn.as_str()));
        fields.push(format!("{} {}", ty, pn.as_str()));
        inits.push(format!("{} = {}", pn.as_str(), pn.as_str()));
    }
    fields.push(format!("{} __inner", target_fut));
    fields.push("int _state".into());
    inits.push(format!("__inner = {}({})", target.as_str(), target_args));
    inits.push("_state = 0".into());
    let src = format!(
        "{vis_s}struct {fut}{gdecl} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}{gdecl}({params}) -> {future_ty} {{\n    return {future_ty} {{ {inits} }}\n}}\n\
         impl{gdecl} {future_ty} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._state == 0 {{\n\
         \x20           __p = self.__inner.poll(ref cx)\n\
         \x20           match __p {{\n\
         \x20               Ready(__v) => {{ self._state = 1; return Poll::Ready(__v) }}\n\
         \x20               Pending => {{ }}\n\
         \x20           }}\n\
         \x20       }}\n\
         \x20       return Poll::Pending()\n    }}\n}}\n",
        fields = fields.join("\n    "),
        params = ptypes.join(", "),
        inits = inits.join(", "),
        name = name.as_str(),
        fut = fut.as_str(),
        gdecl = gdecl,
        future_ty = future_ty,
        ret = ret,
        vis_s = vis_s,
    );
    let generated = crate::parser::parse_source(&src).map_err(|e| {
        Error::Hir(format!(
            "#[async] generic await expansion for `{}` produced invalid code: {} (local `{}`)",
            name.as_str(), e, local.as_str()
        ))
    })?;
    out.extend(generated.stmts);
    Ok(())
}

fn match_single_await(body: &Block) -> Option<(Symbol, Symbol, String)> {
    if body.stmts.len() != 2 {
        return None;
    }
    let Stmt::Assign { name, value: Expr::FnCall { name: await_name, args, .. }, .. } =
        &body.stmts[0]
    else {
        return None;
    };
    if await_name.as_str() != "__await" || args.len() != 1 {
        return None;
    }
    let Expr::FnCall { name: target, args: target_args, generic_args, .. } = &args[0] else {
        return None;
    };
    if !generic_args.is_empty() {
        return None;
    }
    let Stmt::Return { value: Some(Expr::Ident(ret_name, _)), .. } = &body.stmts[1] else {
        return None;
    };
    if ret_name != name {
        return None;
    }
    let args = target_args.iter()
        .map(crate::formatter::format_expr)
        .collect::<Vec<_>>()
        .join(", ");
    Some((*name, *target, args))
}

fn generic_decl(params: &[(Symbol, Vec<Symbol>)]) -> String {
    let text = params.iter().map(|(n, bounds)| {
        if bounds.is_empty() {
            n.as_str().to_string()
        } else {
            format!(
                "{}: {}",
                n.as_str(),
                bounds.iter().map(|b| b.as_str()).collect::<Vec<_>>().join(" + ")
            )
        }
    });
    format!("[{}]", text.collect::<Vec<_>>().join(", "))
}
