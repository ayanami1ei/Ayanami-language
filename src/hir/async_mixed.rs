//! A1b-3a 过渡桥：直线函数中 await 与同步语句交错。
//!
//! 支持 `x = await g(args)` 与普通赋值/表达式语句混排（无 if/while/match），
//! 参数与局部提升为 `__v_*` 字段；每个 await 一个 future 字段与两段状态。
//! 类型优先 AST 侧轻量推断；失败时用 HIR 探针补充（方法调用/字段读取结果），再失败则回退。
use std::collections::{HashMap, HashSet};
use crate::error::{Error, Result};
use crate::hir::async_mixed_expr::{render_default, render_expr, render_sync};
use crate::hir::async_mixed_types::{hir_to_ast, infer, same_type, type_default};
use crate::hir::async_transform_util::ty_src;
use crate::intern::Symbol;
use crate::hir::ir::HirType;
use crate::parser::ast::{Expr, Stmt, Type};
use crate::span::Span;
struct Await {
    target: Symbol,
    g: Symbol,
    placeholders: Vec<String>,
    reals: Vec<String>,
}
/// 尝试展开直线 mixed await；不匹配返回 false。
pub(crate) fn try_expand(
    stmt: &Stmt,
    returns: &HashMap<Symbol, Type>,
    out: &mut Vec<Stmt>,
    hir_locals: Option<&HashMap<Symbol, HirType>>,
) -> Result<bool> {
    let Stmt::FnDecl { name, params, body, .. } = stmt else { return Ok(false) };
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let mut env: HashMap<Symbol, Type> = params.iter().cloned().collect();
    let mut local_order: Vec<Symbol> = Vec::new();
    let mut segments: Vec<Vec<Stmt>> = vec![Vec::new()];
    let mut awaits: Vec<Await> = Vec::new();
    for st in &body.stmts {
        if let Stmt::Assign { name: target, value, .. } = st {
            if let Some((g, gargs, _)) = await_parts(value) {
                let g_ret = match returns.get(&g) { Some(t) => t.clone(), None => return Ok(false) };
                if let Some(old) = env.get(target) {
                    if !same_type(old, &g_ret) { return Ok(false); }
                } else {
                    env.insert(*target, g_ret);
                    local_order.push(*target);
                }
                let cur: HashSet<Symbol> = env.keys().copied().collect();
                let mut placeholders = Vec::new();
                let mut reals = Vec::new();
                for a in gargs {
                    let ph = render_default(a, &env).ok_or_else(|| fail(
                        "unsupported await argument (v1)".into()))?;
                    let real = render_expr(a, &cur).ok_or_else(|| fail(
                        "unsupported await argument (v1)".into()))?;
                    placeholders.push(ph);
                    reals.push(real);
                }
                awaits.push(Await { target: *target, g, placeholders, reals });
                segments.push(Vec::new());
                continue;
            }
        }
        match st {
            Stmt::Assign { name: target, value, span, .. } => {
                let ty = infer(value, &env, *span).or_else(|| {
                    hir_locals.and_then(|m| m.get(target)).and_then(|t| hir_to_ast(t, *span))
                });
                let Some(ty) = ty else { return Ok(false) };
                if let Some(old) = env.get(target) {
                    if !same_type(old, &ty) { return Ok(false); }
                } else {
                    env.insert(*target, ty);
                    local_order.push(*target);
                }
                segments.last_mut().unwrap().push(st.clone());
            }
            Stmt::ExprStmt { .. } => segments.last_mut().unwrap().push(st.clone()),
            Stmt::If { .. } | Stmt::While { .. } | Stmt::Block(_) => {
                if !sync_ctrl_ok(st, &env) { return Ok(false); }
                segments.last_mut().unwrap().push(st.clone());
            }
            Stmt::Return { value: Some(_), .. } => {
                // 只允许函数体最后一条 return
                let is_last = std::ptr::eq(st, body.stmts.last().unwrap());
                if !is_last { return Ok(false); }
                segments.last_mut().unwrap().push(st.clone());
            }
            _ => return Ok(false),
        }
    }
    if awaits.is_empty() { return Ok(false); }
    // 仅当在最后一个 await 之前/之间有同步语句时才接管；纯前缀交给 await_prefix
    let has_pre_sync = segments[..awaits.len()]
        .iter()
        .any(|s| s.iter().any(|st| !matches!(st, Stmt::Return { .. })));
    if !has_pre_sync { return Ok(false); }
    // 末段必须有 return 或尾表达式
    let final_seg = segments.last().unwrap();
    let has_return = matches!(final_seg.last(), Some(Stmt::Return { value: Some(_), .. }));
    if !has_return && body.tail.is_none() { return Ok(false); }
    if has_return && body.tail.is_some() { return Ok(false); }
    let Stmt::FnDecl { vis, return_type, span: _, .. } = stmt else { unreachable!() };
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let mut plist = Vec::new();
    let mut fields = Vec::new();
    let mut inits = Vec::new();
    for (n, t) in params {
        let ts = ty_src(t).ok_or_else(|| fail(format!(
            "unsupported parameter type for `{}` (v1)", n.as_str())))?;
        plist.push(format!("{} {}", ts, n.as_str()));
        fields.push(format!("{} __v_{}", ts, n.as_str()));
        inits.push(format!("__v_{} = {}", n.as_str(), n.as_str()));
    }
    for n in &local_order {
        if params.iter().any(|(p, _)| p == n) { continue; }
        let t = env.get(n).unwrap();
        let ts = ty_src(t).ok_or_else(|| fail(format!(
            "unsupported local type for `{}` (v1)", n.as_str())))?;
        let d = type_default(t).ok_or_else(|| fail(format!(
            "local `{}` has no default initializer (v1)", n.as_str())))?;
        fields.push(format!("{} __v_{}", ts, n.as_str()));
        inits.push(format!("__v_{} = {}", n.as_str(), d));
    }
    for (i, a) in awaits.iter().enumerate() {
        fields.push(format!("__{}_Fut __w{}", a.g.as_str(), i));
        inits.push(format!("__w{} = {}({})", i, a.g.as_str(), a.placeholders.join(", ")));
    }
    fields.push("int _state".to_string());
    inits.push("_state = 0".to_string());
    let promoted: HashSet<Symbol> = env.keys().copied().collect();
    let fut = format!("__{}_Fut", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let mut poll = String::new();
    for i in 0..awaits.len() {
        let seg_state = 2 * i;
        let poll_state = 2 * i + 1;
        let mut body = String::new();
        for st in &segments[i] {
            body.push_str(&render_sync(st, &promoted).ok_or_else(|| fail(
                "unsupported statement between awaits (v1)".into()))?);
            body.push('\n');
        }
        body.push_str(&format!(
            "self.__w{} = {}({}); self._state = {poll_state}",
            i, awaits[i].g.as_str(), awaits[i].reals.join(", ")));
        poll.push_str(&format!("if self._state == {seg_state} {{\n{body}\n}}\n"));
        poll.push_str(&format!(
            "if self._state == {poll_state} {{\n\
             p{} = self.__w{}.poll(ref cx)\n\
             match p{} {{\n\
             Ready(v) => {{ self.__v_{} = v; self._state = {} }}\n\
             Pending => {{ return Poll::Pending() }}\n\
             }}\n}}\n",
            i, i, i, awaits[i].target.as_str(), 2 * i + 2));
    }
    let final_state = 2 * awaits.len();
    let mut final_body = String::new();
    for st in segments.last().unwrap() {
        final_body.push_str(&render_sync(st, &promoted).ok_or_else(|| fail(
            "unsupported final statement (v1)".into()))?);
        final_body.push('\n');
    }
    if let Some(tail) = &body.tail {
        let r = render_expr(tail, &promoted).ok_or_else(|| fail(
            "unsupported tail expression (v1)".into()))?;
        final_body.push_str(&format!("return Poll::Ready({});", r));
    }
    poll.push_str(&format!("if self._state == {final_state} {{\n{final_body}\n}}\n"));
    poll.push_str("return Poll::Pending()\n");
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
    out.extend(generated.stmts);
    Ok(true)
}
fn await_parts(value: &Expr) -> Option<(Symbol, &[Expr], Span)> {
    let Expr::FnCall { name, args, .. } = value else { return None };
    if name.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, span, .. } = args.first()? else { return None };
    Some((*g, gargs, *span))
}

/// 纯同步控制语句：不含 await，且只写已提升/已有局部（不声明新局部）。
fn sync_ctrl_ok(st: &Stmt, env: &HashMap<Symbol, Type>) -> bool {
    match st {
        Stmt::Assign { name, value, .. } => await_parts(value).is_none() && env.contains_key(name),
        Stmt::ExprStmt { .. } => true,
        Stmt::Return { value: Some(_), .. } => true,
        Stmt::If { then_block, elifs, else_block, .. } => {
            then_block.stmts.iter().all(|s| sync_ctrl_ok(s, env))
                && elifs.iter().all(|(_, b)| b.stmts.iter().all(|s| sync_ctrl_ok(s, env)))
                && else_block.as_ref().map_or(true, |b| b.stmts.iter().all(|s| sync_ctrl_ok(s, env)))
        }
        Stmt::While { body, .. } => body.stmts.iter().all(|s| sync_ctrl_ok(s, env)),
        Stmt::Block(b) => b.stmts.iter().all(|s| sync_ctrl_ok(s, env)),
        Stmt::Break { .. } | Stmt::Continue { .. } => true,
        _ => false,
    }
}
