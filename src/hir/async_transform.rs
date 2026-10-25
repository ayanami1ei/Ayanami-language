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
use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Program, Stmt, Type};
use crate::hir::async_transform_util::{await_default, type_name, ty_src};
/// 展开程序中的 `#[async]` 函数（cfg 过滤之后、HIR 降级之前）。
pub fn expand(program: &Program) -> Result<Program> {
    // await 目标函数的返回类型表（仅同程序 #[async] 函数）
    let mut returns: HashMap<Symbol, Type> = HashMap::new();
    for stmt in &program.stmts {
        if let Stmt::FnDecl { attrs, name, return_type, .. } = stmt {
            if attrs.iter().any(is_async) {
                returns.insert(*name, return_type.clone());
            }
        }
    }
    let mut out = Vec::with_capacity(program.stmts.len());
    for stmt in &program.stmts {
        match stmt {
            Stmt::FnDecl { attrs, .. } if attrs.iter().any(is_async) => expand_one(stmt, &returns, &mut out)?,
            Stmt::FnDecl { attrs, body, .. }
                if !attrs.iter().any(is_async) && body.stmts.iter().any(stmt_has_direct_await) =>
            {
                return Err(Error::Hir(
                    "`await` is only allowed inside #[async] functions".into(),
                ));
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
fn is_async(a: &crate::parser::ast::Attr) -> bool {
    a.qualifier.is_empty() && a.name.as_str() == "async"
}
/// 语句里是否直接出现 `__await(...)`（await 语法糖在解析期降级为该调用）。
fn stmt_has_direct_await(st: &Stmt) -> bool {
    let is_await = |e: &Expr| matches!(e, Expr::FnCall { name, .. } if name.as_str() == "__await");
    match st {
        Stmt::Assign { value, .. } => is_await(value),
        Stmt::ExprStmt { expr, .. } => is_await(expr),
        Stmt::Return { value: Some(e), .. } => is_await(e),
        _ => false,
    }
}
/// 递归检查循环/分支/块内的 await（v1 不支持）。
fn stmt_has_nested_await(st: &Stmt) -> bool {
    let any = |b: &Block| b.stmts.iter().any(stmt_has_nested_await);
    match st {
        Stmt::If { then_block, elifs, else_block, .. } => {
            any(then_block)
                || elifs.iter().any(|(_, b)| any(b))
                || else_block.as_ref().map_or(false, any)
        }
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForIn { body, .. } => any(body),
        Stmt::Block(b) => any(b),
        Stmt::Attributed { stmt, .. } => stmt_has_nested_await(stmt),
        _ => stmt_has_direct_await(st),
    }
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

fn expand_one(stmt: &Stmt, returns: &HashMap<Symbol, Type>, out: &mut Vec<Stmt>) -> Result<()> {
    let Stmt::FnDecl {
        vis, name, generic_params, params, return_type, extern_c, is_unsafe, body, span, ..
    } = stmt else { unreachable!() };
    let fail = |msg: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), msg));
    if !generic_params.is_empty() {
        return Err(fail("generic functions are not supported (v1)".into()));
    }
    if *extern_c || *is_unsafe {
        return Err(fail("extern/unsafe functions are not supported (v1)".into()));
    }
    if matches!(return_type, Type::Void(_) | Type::Default) {
        return Err(fail("a non-void return type is required (v1)".into()));
    }
    // A1b-2：直线 await 前缀（0..N 个连续 `x = await g(args)`，目标为同程序 #[async] 直接调用）
    if crate::hir::async_elif::try_expand(stmt, returns, out)? { return Ok(()); }
    if crate::hir::async_if::try_expand(stmt, returns, out)? { return Ok(()); }
    if crate::hir::async_while::try_expand(stmt, returns, out)? { return Ok(()); }
    if crate::hir::async_mixed::try_expand(stmt, returns, out)? { return Ok(()); }
    let awaits = await_prefix(body, span)?;
    if !awaits.is_empty() {
        return expand_await(stmt, &awaits, returns, out);
    }
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let mut ptypes = Vec::with_capacity(params.len());
    for (pn, pt) in params {
        let t = ty_src(pt)
            .ok_or_else(|| fail(format!("unsupported parameter type for `{}` (v1)", pn.as_str())))?;
        ptypes.push(t);
    }
    let fut = format!("__{}_Fut", name.as_str());
    let run = format!("__async_run_{}", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let plist: Vec<String> = params.iter().zip(&ptypes)
        .map(|((n, _), t)| format!("{} {}", t, n.as_str())).collect();
    let fields: Vec<String> = params.iter().zip(&ptypes)
        .map(|((n, _), t)| format!("{} {}", t, n.as_str())).collect();
    let inits: Vec<String> = params.iter()
        .map(|(n, _)| format!("{} = {}", n.as_str(), n.as_str())).collect();
    let args: Vec<String> = params.iter()
        .map(|(n, _)| format!("self.{}", n.as_str())).collect();
    let field_decl = if fields.is_empty() {
        "int _done".to_string()
    } else {
        format!("{}\n    int _done", fields.join("\n    "))
    };
    let src = format!(
        "{vis_s}struct {fut} {{\n    {field_decl}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits}{isep}_done = 0 }}\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
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
        Error::Hir(format!("#[async] expansion for `{}` produced invalid code: {}", name.as_str(), e))
    })?;
    out.extend(generated.stmts);
    let mut runner = stmt.clone();
    if let Stmt::FnDecl { attrs, name: rn, vis: rv, .. } = &mut runner {
        attrs.clear();
        *rn = Symbol::intern(&run);
        *rv = Visibility::Private;
    }
    out.push(runner);
    Ok(())
}
/// 识别函数体开头的连续 `x = await g(args)` 前缀。
fn await_prefix(body: &Block, span: &crate::span::Span) -> Result<Vec<(Symbol, Symbol, Vec<Expr>)>> {
    let mut out = Vec::new();
    for st in &body.stmts {
        let Stmt::Assign { name, value, .. } = st else { break };
        let Expr::FnCall { name: an, args, .. } = value else { break };
        if an.as_str() != "__await" { break; }
        let Some(Expr::FnCall { name: g, args: gargs, .. }) = args.first() else {
            return Err(Error::Hir(format!(
                "#[async] v1: `await` target must be a direct function call (at {}:{})",
                span.start_line, span.start_col
            )));
        };
        out.push((*name, *g, gargs.clone()));
    }
    for st in body.stmts.iter().skip(out.len()) {
        if stmt_has_nested_await(st) {
            return Err(Error::Hir(format!(
                "#[async] v1: `await` must be a straight-line prefix; awaits inside loops/branches are not supported (at {}:{})",
                span.start_line, span.start_col
            )));
        }
    }
    Ok(out)
}

/// N 个直线挂起点的状态机：构造器立即建全部内层 future，poll 依次轮询。
fn expand_await(
    stmt: &Stmt, awaits: &[(Symbol, Symbol, Vec<Expr>)],
    returns: &HashMap<Symbol, Type>, out: &mut Vec<Stmt>,
) -> Result<()> {
    let Stmt::FnDecl { vis, name, params, return_type, body, .. } = stmt else { unreachable!() };
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
        let ts = ty_src(t)
            .ok_or_else(|| fail(format!("unsupported parameter type for `{}` (v1)", n.as_str())))?;
        plist.push(format!("{} {}", ts, n.as_str()));
        fields.push(format!("{} {}", ts, n.as_str()));
        inits.push(format!("{} = {}", n.as_str(), n.as_str()));
        self_args.push(format!("self.{}", n.as_str()));
    }
    let mut await_y: Vec<(Symbol, Type)> = Vec::new();
    let mut y_defaults: Vec<&'static str> = Vec::new();
    let mut polls = String::new();
    for (i, (y, g, gargs)) in awaits.iter().enumerate() {
        let g_ret = returns.get(g).ok_or_else(|| fail(format!(
            "await target `{}` is not an #[async] function (v1)", g.as_str())))?;
        let (y_ty, y_init) = await_default(g_ret).ok_or_else(|| fail(format!(
            "unsupported await value type `{}` (v1)", type_name(g_ret))))?;
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
                    n.as_str())));
            }
        }
        fields.push(format!("__{}_Fut __w{}", g.as_str(), i));
        fields.push(format!("{} {}", y_ty, y.as_str()));
        inits.push(format!("__w{} = {}({})", i, g.as_str(), arg_names.join(", ")));
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
                        n.as_str())));
                }
            }
            enter = format!("; self.__w{} = {}({})", next, gnext.as_str(), real.join(", "));
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
        plist = plist.join(", "), args = self_args.join(", "),
        name = name.as_str(), ret = ret,
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    out.extend(generated.stmts);
    let mut helper = stmt.clone();
    if let Stmt::FnDecl { attrs, name: hn, params: hp, body: hb, vis: hv, .. } = &mut helper {
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
