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
use crate::error::{Error, Result};
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Program, Stmt, Type};

/// 展开程序中的 `#[async]` 函数（cfg 过滤之后、HIR 降级之前）。
pub fn expand(program: &Program) -> Result<Program> {
    let mut out = Vec::with_capacity(program.stmts.len());
    for stmt in &program.stmts {
        match stmt {
            Stmt::FnDecl { attrs, .. } if attrs.iter().any(is_async) => expand_one(stmt, &mut out)?,
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

fn contains_async(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::FnDecl { attrs, .. } => attrs.iter().any(is_async),
        Stmt::Namespace { items, .. } => items.iter().any(contains_async),
        Stmt::ImplBlock { methods, .. } => methods.iter().any(contains_async),
        Stmt::Attributed { stmt, .. } => contains_async(stmt),
        _ => false,
    }
}

fn expand_one(stmt: &Stmt, out: &mut Vec<Stmt>) -> Result<()> {
    let Stmt::FnDecl {
        vis, name, generic_params, params, return_type, extern_c, is_unsafe, ..
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

/// 生成源码用的类型串（仅 v1 允许的类型）。
fn ty_src(ty: &Type) -> Option<String> {
    match ty {
        Type::Int(_) => Some("int".into()),
        Type::Float(_) => Some("float".into()),
        Type::Char(_) => Some("char".into()),
        Type::Bool(_) => Some("bool".into()),
        Type::Named(s, _) => Some(s.as_str().to_string()),
        Type::Generic(n, args, _) => {
            let mut out = Vec::with_capacity(args.len());
            for a in args { out.push(ty_src(a)?); }
            Some(format!("{}[{}]", n.as_str(), out.join(", ")))
        }
        _ => None,
    }
}
