//! A1b：`#[async]` AST 变换（第一切片：无 await；顶层非泛型自由函数）。
//!
//! 设计见 docs/async.md §7.1。展开：
//!   struct __f_Fut { T a, int _done }
//!   fn f(T a) -> __f_Fut { return __f_Fut { a = a, _done = 0 } }
//!   fn __async_run_f(T a) -> R { <原体> }
//!   impl __f_Fut { fn poll(ref mut self, ref Context cx) -> Poll[R] { ... } }
//!
//! `#[async]` 在 HIR 之前被本 pass 消费，因此不进入 HIR 注解校验；
//! 宏展开期由 macro_expand::expand_util::is_compiler_attr 放行。
use super::async_transform_await::{await_prefix, expand_await};
use crate::error::{Error, Result};
use crate::hir::async_transform_util::{attach_qualified_attrs, ty_src};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Program, Stmt, Type};
use std::collections::{HashMap, HashSet};
/// 展开程序中的 `#[async]` 函数（cfg 过滤之后、HIR 降级之前）。
pub fn expand(
    program: &Program,
    types: Option<&crate::hir::lower::AsyncLocalTypes>,
) -> Result<Program> {
    // await 目标函数的返回类型表（仅同程序 #[async] 函数）
    let mut returns: HashMap<Symbol, Type> = HashMap::new();
    let mut async_fns: HashMap<Symbol, Vec<(Symbol, Type)>> = HashMap::new();
    let mut generic_async_fns: HashSet<Symbol> = HashSet::new();
    let mut fn_returns: HashMap<Symbol, Type> = HashMap::new();
    for stmt in &program.stmts {
        if let Stmt::FnDecl {
            attrs,
            name,
            generic_params,
            return_type,
            params,
            ..
        } = stmt
        {
            fn_returns.insert(*name, return_type.clone());
            if attrs.iter().any(is_async) {
                returns.insert(*name, return_type.clone());
                async_fns.insert(*name, params.clone());
                if !generic_params.is_empty() {
                    generic_async_fns.insert(*name);
                }
            }
        }
    }
    let mut out = Vec::with_capacity(program.stmts.len());
    for stmt in &program.stmts {
        match stmt {
            Stmt::FnDecl { attrs, name, .. } if attrs.iter().any(is_async) => {
                let start = out.len();
                expand_one(
                    stmt,
                    &returns,
                    &async_fns,
                    &generic_async_fns,
                    &fn_returns,
                    &mut out,
                    types,
                )?;
                attach_qualified_attrs(&mut out[start..], *name, attrs);
            }
            Stmt::FnDecl { attrs, body, .. }
                if !attrs.iter().any(is_async)
                    && (body.stmts.iter().any(crate::hir::async_cfg::stmt_has_await)
                        || body
                            .tail
                            .as_ref()
                            .is_some_and(|e| crate::hir::async_cfg::expr_has_await_pub(e))) =>
            {
                return Err(Error::Hir(
                    "`await` is only allowed inside #[async] functions".into(),
                ));
            }
            Stmt::ImplBlock { methods, .. } if methods.iter().any(contains_async) => {
                crate::hir::async_transform_impl::expand_impl(stmt, &mut out)?;
            }
            _ if contains_async(stmt) => {
                return Err(Error::Hir(
                    "#[async] v1 is only supported on top-level free functions".into(),
                ));
            }
            _ => out.push(stmt.clone()),
        }
    }
    Ok(Program::new(out))
}
pub(super) fn is_async(a: &crate::parser::ast::Attr) -> bool {
    a.qualifier.is_empty() && a.name.as_str() == "async"
}
fn contains_async(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::FnDecl { attrs, .. } => attrs.iter().any(is_async),
        Stmt::Namespace { items, .. } => items.iter().any(contains_async),
        Stmt::ImplBlock { methods, .. } => methods.iter().any(contains_async),
        Stmt::Attributed { stmt, .. } => contains_async(stmt),
        _ => false,
    }
}

fn expand_one(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    async_fns: &HashMap<Symbol, Vec<(Symbol, Type)>>,
    generic_async_fns: &HashSet<Symbol>,
    fn_returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
    types: Option<&crate::hir::lower::AsyncLocalTypes>,
) -> Result<()> {
    let Stmt::FnDecl {
        vis,
        name,
        generic_params,
        params,
        return_type,
        extern_c,
        is_unsafe,
        body,
        span,
        ..
    } = stmt
    else {
        unreachable!()
    };
    let fail = |msg: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), msg));
    let has_await = body.stmts.iter().any(crate::hir::async_cfg::stmt_has_await)
        || body
            .tail
            .as_ref()
            .is_some_and(|e| crate::hir::async_cfg::expr_has_await_pub(e));
    if !generic_params.is_empty() && has_await {
        return crate::hir::async_transform_generic_await::expand(
            stmt,
            returns,
            generic_async_fns,
            out,
        );
    }
    if *extern_c || *is_unsafe {
        return Err(fail(
            "extern/unsafe functions are not supported (v1)".into(),
        ));
    }
    if matches!(return_type, Type::Void(_) | Type::Default) {
        return Err(fail("a non-void return type is required (v1)".into()));
    }
    // A1b-3：通用 CFG（HIR 类型探针 + AST 状态机）；不支持时回退形状模块。
    if crate::hir::async_cfg::try_expand(
        stmt,
        returns,
        async_fns,
        fn_returns,
        out,
        types.and_then(|m| m.get(name)),
    )? {
        return Ok(());
    }
    // A1b-2：直线 await 前缀（0..N 个连续 `x = await g(args)`，目标为同程序 #[async] 直接调用）
    if crate::hir::async_elif::try_expand(stmt, returns, out)? {
        return Ok(());
    }
    if crate::hir::async_if::try_expand(stmt, returns, out)? {
        return Ok(());
    }
    if crate::hir::async_while::try_expand(stmt, returns, out)? {
        return Ok(());
    }
    if crate::hir::async_mixed::try_expand(stmt, returns, out, types.and_then(|m| m.get(name)))? {
        return Ok(());
    }
    let awaits = await_prefix(body, span)?;
    if !awaits.is_empty() {
        return expand_await(stmt, &awaits, returns, out);
    }
    if has_await {
        return Err(fail(
            "awaits inside loops/branches are not supported here \
             (v1: assignment await in if/while/block)"
                .into(),
        ));
    }
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let mut ptypes = Vec::with_capacity(params.len());
    for (pn, pt) in params {
        let t = ty_src(pt).ok_or_else(|| {
            fail(format!(
                "unsupported parameter type for `{}` (v1)",
                pn.as_str()
            ))
        })?;
        ptypes.push(t);
    }
    let fut = format!("__{}_Fut", name.as_str());
    let run = format!("__async_run_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let generic_names: Vec<String> = generic_params.iter().map(|(n, _)| n.as_str()).collect();
    let generic_args = if generic_names.is_empty() {
        String::new()
    } else {
        format!("[{}]", generic_names.join(", "))
    };
    let generic_decl = if generic_params.is_empty() {
        String::new()
    } else {
        let params = generic_params.iter().map(|(n, bounds)| {
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
        format!("[{}]", params.collect::<Vec<_>>().join(", "))
    };
    let future_ty = if generic_args.is_empty() {
        fut.clone()
    } else {
        format!("{}{}", fut, generic_args)
    };
    let plist: Vec<String> = params
        .iter()
        .zip(&ptypes)
        .map(|((n, _), t)| format!("{} {}", t, n.as_str()))
        .collect();
    let fields: Vec<String> = params
        .iter()
        .zip(&ptypes)
        .map(|((n, _), t)| format!("{} {}", t, n.as_str()))
        .collect();
    let inits: Vec<String> = params
        .iter()
        .map(|(n, _)| format!("{} = {}", n.as_str(), n.as_str()))
        .collect();
    let args: Vec<String> = params
        .iter()
        .map(|(n, _)| format!("self.{}", n.as_str()))
        .collect();
    let field_decl = if fields.is_empty() {
        "int _done".to_string()
    } else {
        format!("{}\n    int _done", fields.join("\n    "))
    };
    let src = format!(
        "{vis_s}struct {fut}{generic_decl} {{\n    {field_decl}\n}}\n\
         {vis_s}fn {name}{generic_decl}({plist}) -> {future_ty} {{\n    return {future_ty} {{ {inits}{isep}_done = 0 }}\n}}\n\
         impl{generic_decl} {future_ty} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._done == 1 {{ return Poll::Pending() }}\n\
         \x20       self._done = 1\n\
         \x20       return Poll::Ready({run}({args}))\n    }}\n}}\n",
        inits = inits.join(", "),
        isep = if inits.is_empty() { "" } else { ", " },
        args = args.join(", "),
        name = name.as_str(),
        plist = plist.join(", "),
        ret = ret,
    );
    let generated = crate::parser::parse_source(&src).map_err(|e| {
        Error::Hir(format!(
            "#[async] expansion for `{}` produced invalid code: {}",
            name.as_str(),
            e
        ))
    })?;
    out.extend(generated.stmts);
    let mut runner = stmt.clone();
    if let Stmt::FnDecl {
        attrs,
        name: rn,
        vis: rv,
        ..
    } = &mut runner
    {
        attrs.clear();
        *rn = Symbol::intern(&run);
        *rv = Visibility::Private;
    }
    out.push(runner);
    Ok(())
}
