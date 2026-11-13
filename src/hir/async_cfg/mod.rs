//! A1b-3：通用 async CFG（AST 状态机 + HIR 类型探针）。
//!
//! 把所有参数/局部提升为 `__f_Fut` 字段；按挂起点拆基本块，生成
//! `while true { if self._state == n { ... } }` 的 poll 状态机。
//! 条件/循环/赋值等同步语句原样进入状态块，避免形状组合膨胀。
use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};
use crate::hir::async_mixed_types::{hir_to_ast, type_default};
use crate::hir::async_transform_util::ty_src;
use crate::hir::ir::HirType;
use crate::intern::Symbol;
use crate::parser::ast::{Expr, Stmt, Type};

mod compiler;
mod compiler_await;
mod scan;
use compiler::Cfg;
pub(crate) use scan::stmt_has_await;
use scan::{body_has_await, check_supported};

pub(super) struct Local {
    pub name: Symbol,
    pub field: Symbol,
    pub ty_src: String,
    pub init: String,
    pub is_param: bool,
    pub is_copy: bool,
}

pub(super) struct FutureField {
    pub field: Symbol,
    pub ty: String,
    pub init: String,
}

/// 通用 CFG 默认启用；`AYANAMI_ASYNC_CFG=0` 可退回形状模块。
fn enabled() -> bool {
    std::env::var("AYANAMI_ASYNC_CFG")
        .map(|v| v != "0")
        .unwrap_or(true)
}

/// 尝试用通用 CFG 展开一个 `#[async]` 函数；不支持时返回 false 交给形状模块。
pub(crate) fn try_expand(
    stmt: &Stmt,
    async_fns: &HashMap<Symbol, Vec<(Symbol, Type)>>,
    out: &mut Vec<Stmt>,
    types: Option<&HashMap<Symbol, HirType>>,
) -> Result<bool> {
    if !enabled() {
        return Ok(false);
    }
    let Stmt::FnDecl {
        name,
        params,
        return_type,
        body,
        vis,
        ..
    } = stmt
    else {
        return Ok(false);
    };
    if !body_has_await(body) {
        return Ok(false);
    }
    let fail = |m: String| Error::Hir(format!("#[async] `{}`: {}", name.as_str(), m));
    let span = body.span;

    let mut locals: Vec<Local> = Vec::new();
    for (n, t) in params {
        let ty = source_ty(t).ok_or_else(|| {
            fail(format!(
                "unsupported parameter type for `{}` (v1)",
                n.as_str()
            ))
        })?;
        if ty.starts_with("ref ") || ty.starts_with("ref mut ") {
            return Ok(false);
        }
        let is_copy = types.and_then(|m| m.get(n)).is_some_and(|t| t.is_copy());
        locals.push(Local {
            name: *n,
            field: field_name(*n),
            ty_src: ty,
            init: String::new(),
            is_param: true,
            is_copy,
        });
    }
    let mut assigned = HashSet::new();
    collect_assigned(&body.stmts, &mut assigned);
    let mut assigned: Vec<Symbol> = assigned.into_iter().collect();
    assigned.sort_by_key(|n| n.as_str().to_string());
    for n in assigned {
        if params.iter().any(|(p, _)| *p == n) {
            continue;
        }
        let Some(hir_ty) = types.and_then(|m| m.get(&n)) else {
            return Ok(false);
        };
        let Some(ast_ty) = hir_to_ast(hir_ty, span) else {
            return Ok(false);
        };
        let Some(ty) = source_ty(&ast_ty) else {
            return Ok(false);
        };
        let Some(init) = type_default(&ast_ty) else {
            return Ok(false);
        };
        let is_copy = hir_ty.is_copy();
        locals.push(Local {
            name: n,
            field: field_name(n),
            ty_src: ty,
            init: init.to_string(),
            is_param: false,
            is_copy,
        });
    }
    if locals.iter().any(|l| bad_local_name(l.name)) {
        return Ok(false);
    }
    if locals.len() != locals.iter().map(|l| l.field).collect::<HashSet<_>>().len() {
        return Ok(false);
    }

    let tail = body.tail.as_ref().map(|t| (**t).clone());
    let terminal = tail.is_some() || matches!(body.stmts.last(), Some(Stmt::Return { .. }));
    if !terminal {
        return Ok(false);
    }
    let mut stmts = body.stmts.clone();
    if let Some(t) = tail {
        stmts.push(Stmt::Return {
            value: Some(t),
            span,
        });
    }
    if !check_supported(&stmts, async_fns) {
        return Ok(false);
    }
    let mut cfg = Cfg::new(locals, async_fns.clone());
    let entry = cfg.compile(&stmts)?;
    if cfg.unsupported() {
        return Ok(false);
    }

    let ret = source_ty(return_type).ok_or_else(|| fail("unsupported return type (v1)".into()))?;
    let fut = format!("__{}_Fut", name.as_str());
    let vis_s = if vis.is_public() { "pub " } else { "" };
    let plist: Vec<String> = params
        .iter()
        .map(|(n, t)| source_ty(t).map(|s| format!("{} {}", s, n.as_str())))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| fail("unsupported parameter type (v1)".into()))?;

    let mut fields: Vec<String> = cfg
        .locals()
        .iter()
        .map(|l| format!("{} {}", l.ty_src, l.field.as_str()))
        .collect();
    fields.extend(
        cfg.futures()
            .iter()
            .map(|f| format!("{} {}", f.ty, f.field.as_str())),
    );
    fields.push("int _state".to_string());

    let mut inits: Vec<String> = cfg
        .locals()
        .iter()
        .map(|l| {
            if l.is_param {
                format!("{} = {}", l.field.as_str(), l.name.as_str())
            } else {
                format!("{} = {}", l.field.as_str(), l.init)
            }
        })
        .collect();
    inits.extend(
        cfg.futures()
            .iter()
            .map(|f| format!("{} = {}", f.field.as_str(), f.init)),
    );
    inits.push(format!("_state = {}", entry));

    let mut dispatch = String::new();
    for (i, st) in cfg.states().iter().enumerate() {
        let code = st
            .as_deref()
            .ok_or_else(|| fail("internal CFG state not filled".into()))?;
        dispatch.push_str(&format!("if self._state == {i} {{\n{code}}}\n"));
    }

    let src = format!(
        "{vis_s}struct {fut} {{\n    {fields}\n}}\n\
         {vis_s}fn {name}({plist}) -> {fut} {{\n    return {fut} {{ {inits} }};\n}}\n\
         impl {fut} {{\n    #[state]\n    fn poll(ref mut self, ref Context __async_cx) -> Poll[{ret}] {{\n\
         while true {{\n{dispatch}\
         return Poll::Pending()\n    }}\n    }}\n}}\n",
        fields = fields.join("\n    "),
        plist = plist.join(", "),
        inits = inits.join(", "),
        name = name.as_str(),
        ret = ret,
    );
    let generated = crate::parser::parse_source(&src)
        .map_err(|e| fail(format!("produced invalid code: {}", e)))?;
    out.extend(generated.stmts);
    Ok(true)
}

fn field_name(n: Symbol) -> Symbol {
    Symbol::intern(&format!("__v_{}", n.as_str()))
}

fn bad_local_name(n: Symbol) -> bool {
    let s = n.as_str();
    s == "self" || s.starts_with("__async_")
}

fn source_ty(t: &Type) -> Option<String> {
    let s = ty_src(t)?;
    Some(s.replace('<', "[").replace('>', "]"))
}

fn collect_assigned(stmts: &[Stmt], out: &mut HashSet<Symbol>) {
    for st in stmts {
        match st {
            Stmt::Assign { name, .. } => {
                out.insert(*name);
            }
            Stmt::If {
                then_block,
                elifs,
                else_block,
                ..
            } => {
                collect_assigned(&then_block.stmts, out);
                for (_, b) in elifs {
                    collect_assigned(&b.stmts, out);
                }
                if let Some(b) = else_block {
                    collect_assigned(&b.stmts, out);
                }
            }
            Stmt::While { body, .. } | Stmt::Block(body) => {
                collect_assigned(&body.stmts, out);
            }
            Stmt::For { iterator, body, .. } => {
                out.insert(*iterator);
                collect_assigned(&body.stmts, out);
            }
            Stmt::ForIn { iterator, body, .. } => {
                out.insert(*iterator);
                collect_assigned(&body.stmts, out);
            }
            Stmt::Unsafe { block, .. } => collect_assigned(&block.stmts, out),
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let crate::parser::ast::stmt::MatchBody::Block(b) = &arm.body {
                        collect_assigned(&b.stmts, out);
                    }
                }
            }
            Stmt::Attributed { stmt, .. } => {
                collect_assigned(std::slice::from_ref(stmt.as_ref()), out)
            }
            _ => {}
        }
    }
}

pub(crate) fn expr_has_await_pub(e: &Expr) -> bool {
    scan::expr_has_await(e)
}

pub(super) fn await_parts(value: &Expr) -> Option<(Symbol, Vec<Expr>)> {
    let Expr::FnCall { name, args, .. } = value else {
        return None;
    };
    if name.as_str() != "__await" {
        return None;
    }
    let Expr::FnCall {
        name: g,
        args: gargs,
        ..
    } = args.first()?
    else {
        return None;
    };
    Some((*g, gargs.clone()))
}
