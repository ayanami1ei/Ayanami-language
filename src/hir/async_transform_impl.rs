//! A1c-3：impl 方法无挂起 `#[async]`。
//!
//! 首版在 wrapper 中立即执行原方法，把 Copy 结果存进已完成 Future。这样 Future
//! 不保存 receiver 借用；带 await 或非 Copy 返回先明确拒绝。
use super::async_transform_util::ty_src;
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Literal, Stmt, Type};
use std::collections::HashSet;

pub(super) fn expand_impl(stmt: &Stmt, out: &mut Vec<Stmt>) -> Result<()> {
    let Stmt::ImplBlock {
        type_name,
        generic_params,
        methods,
        ..
    } = stmt
    else {
        unreachable!()
    };
    if !generic_params.is_empty() {
        return Err(Error::Hir(
            "#[async] on methods in generic impls is not supported in this slice".into(),
        ));
    }

    let mut generated = String::new();
    let mut rewritten = Vec::with_capacity(methods.len() * 2);
    let mut async_names = HashSet::new();
    for method in methods {
        let Stmt::FnDecl {
            attrs, name, generic_params, params, span, ..
        } = method
        else {
            rewritten.push(method.clone());
            continue;
        };
        if !attrs.iter().any(super::async_transform::is_async) {
            rewritten.push(method.clone());
            continue;
        }
        if !async_names.insert(*name) {
            return Err(Error::Hir(format!(
                "#[async] overloaded method `{}::{}` is not supported in this slice",
                type_name.as_str(),
                name.as_str()
            )));
        }
        expand_method(
            type_name,
            name,
            generic_params,
            params,
            method,
            *span,
            &mut generated,
            &mut rewritten,
        )?;
    }

    let parsed = crate::parser::parse_source(&generated).map_err(|e| {
        Error::Hir(format!(
            "#[async] method expansion for `{}` produced invalid code: {}",
            type_name.as_str(),
            e
        ))
    })?;
    out.extend(parsed.stmts);

    let mut impl_out = stmt.clone();
    if let Stmt::ImplBlock { methods, .. } = &mut impl_out {
        *methods = rewritten;
    }
    out.push(impl_out);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn expand_method(
    type_name: &Symbol,
    name: &Symbol,
    generic_params: &[(Symbol, Vec<Symbol>)],
    params: &[(Symbol, Type)],
    method: &Stmt,
    span: crate::span::Span,
    generated: &mut String,
    rewritten: &mut Vec<Stmt>,
) -> Result<()> {
    let fail = |msg: &str| Error::Hir(format!("#[async] method `{}`: {}", name.as_str(), msg));
    if !generic_params.is_empty() {
        return Err(fail("generic methods are not supported in this slice"));
    }
    let Some((self_name, _)) = params.first() else {
        return Err(fail("a receiver parameter is required"));
    };
    if self_name.as_str() != "self" {
        return Err(fail("a receiver parameter is required"));
    }
    let Stmt::FnDecl {
        vis,
        return_type,
        body,
        ..
    } = method
    else {
        unreachable!()
    };
    if body.stmts.iter().any(crate::hir::async_cfg::stmt_has_await)
        || body
            .tail
            .as_ref()
            .is_some_and(|e| crate::hir::async_cfg::expr_has_await_pub(e))
    {
        return Err(fail("awaiting methods are not supported in this slice"));
    }
    if !is_copy_return(return_type) {
        return Err(fail("this slice requires a Copy primitive return type"));
    }
    let ret = ty_src(return_type).ok_or_else(|| fail("unsupported return type"))?;
    let base = format!(
        "__async_{}_{}",
        sanitize(type_name.as_str()),
        sanitize(name.as_str())
    );
    let fut_sym = Symbol::intern(&format!("{}_Fut", base));
    let runner_sym = Symbol::intern(&format!("{}_run", base));
    let vis_s = if vis.is_public() { "pub " } else { "" };
    generated.push_str(&format!(
        "{vis_s}struct {fut} {{\n    {ret} __value\n    int _done\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context cx) -> Poll[{ret}] {{\n\
         \x20       if self._done == 1 {{ return Poll::Pending() }}\n\
         \x20       self._done = 1\n\
         \x20       return Poll::Ready(self.__value)\n    }}\n}}\n",
        fut = fut_sym.as_str(),
        ret = ret
    ));

    let mut runner = method.clone();
    if let Stmt::FnDecl {
        attrs,
        name,
        vis,
        ..
    } = &mut runner
    {
        attrs.clear();
        *name = runner_sym;
        *vis = Visibility::Private;
    }
    rewritten.push(runner);

    let mut wrapper = method.clone();
    if let Stmt::FnDecl {
        attrs,
        return_type,
        body,
        ..
    } = &mut wrapper
    {
        attrs.retain(|a| !super::async_transform::is_async(a));
        *return_type = Type::Named(fut_sym, span);
        let args = params.iter().skip(1)
            .map(|(n, _)| Expr::Ident(*n, span))
            .collect();
        let call = Expr::MethodCall {
            object: Box::new(Expr::Ident(Symbol::intern("self"), span)),
            method: runner_sym,
            args,
            span,
        };
        let value = Expr::StructLiteral {
            type_name: fut_sym,
            generic_args: Vec::new(),
            fields: vec![
                (Symbol::intern("__value"), call),
                (Symbol::intern("_done"), Expr::Literal(Literal::Int(0, span))),
            ],
            span,
        };
        *body = Block::new(
            vec![Stmt::Return {
                value: Some(value),
                span,
            }],
            span,
        );
    }
    rewritten.push(wrapper);
    Ok(())
}

fn is_copy_return(ty: &Type) -> bool {
    match ty {
        Type::Int(_) | Type::Float(_) | Type::Char(_) | Type::Bool(_) => true,
        Type::Named(s, _) => matches!(
            s.as_str().as_str(),
            "i8" | "i16" | "i32" | "i64" | "i128"
                | "u8" | "u16" | "u32" | "u64" | "u128"
                | "isize" | "usize" | "f32" | "f64"
        ),
        _ => false,
    }
}

fn sanitize(s: String) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}
