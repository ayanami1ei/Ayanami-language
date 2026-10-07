//! M2 闭包辅助：捕获分析（AST 层自由变量收集）与闭包符号命名。
//!
//! 捕获语义（见 docs/closures.md）：
//! - lambda body 中引用、在外层作用域可见、且未在 lambda 内先声明的变量按值捕获；
//! - `x = v` 且外层有 `x` → 捕获 + 可变写（FnMut）；外层没有 → lambda 局部声明；
//! - `ref` / `ref mut` 类型不可捕获（引用不可存字段）。

use super::*;
use crate::error::{Error, Result};
use std::collections::HashSet;

/// 收集 lambda 的捕获变量（按首次出现顺序返回 `(名字, 类型)`）。
pub(crate) fn collect_captures(
    ctx: &crate::hir::lower::Ctx,
    params: &[(Symbol, Type)],
    body: &Block,
) -> Result<Vec<(Symbol, HirType)>> {
    let mut declared: HashSet<Symbol> = params.iter().map(|(n, _)| *n).collect();
    let mut caps: Vec<(Symbol, HirType)> = Vec::new();
    walk_block(ctx, body, &mut declared, &mut caps)?;
    Ok(caps)
}

/// 名字在外层可见（外层函数局部/参数，或外层 lambda 的捕获）→ 类型。
fn enclosing_var_type(ctx: &crate::hir::lower::Ctx, name: &Symbol) -> Option<HirType> {
    if let Some(env) = &ctx.lambda_env {
        if let Some((_, ty)) = env.lookup(name) {
            return Some(ty);
        }
    }
    ctx.lookup_var(name).map(|(_, ty, _)| ty)
}

/// 变量“使用”：若为外层作用域变量则记为捕获。
fn use_name(
    ctx: &crate::hir::lower::Ctx,
    name: &Symbol,
    declared: &HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
    span: &Span,
) -> Result<()> {
    if declared.contains(name) || caps.iter().any(|(n, _)| n == name) {
        return Ok(());
    }
    if let Some(ty) = enclosing_var_type(ctx, name) {
        ensure_capturable(name, &ty, span)?;
        caps.push((*name, ty));
    }
    Ok(())
}

/// 变量“赋值”：外层有同名变量 → 捕获（可变写）；否则为 lambda 局部声明。
fn assign_name(
    ctx: &crate::hir::lower::Ctx,
    name: &Symbol,
    declared: &mut HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
    span: &Span,
) -> Result<()> {
    if declared.contains(name) || caps.iter().any(|(n, _)| n == name) {
        return Ok(());
    }
    if let Some(ty) = enclosing_var_type(ctx, name) {
        ensure_capturable(name, &ty, span)?;
        caps.push((*name, ty));
    } else {
        declared.insert(*name);
    }
    Ok(())
}

fn ensure_capturable(name: &Symbol, ty: &HirType, span: &Span) -> Result<()> {
    if matches!(ty, HirType::Ref(..)) {
        return Err(Error::Hir(format!(
            "cannot capture `{}`: it is a reference (references cannot be stored in closures yet); \
             copy the value first (at {}:{})",
            name.as_str(), span.start_line, span.start_col
        )));
    }
    Ok(())
}

fn walk_block(
    ctx: &crate::hir::lower::Ctx,
    block: &Block,
    declared: &mut HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
) -> Result<()> {
    // 块级作用域：块内声明不外泄
    let mut local = declared.clone();
    for s in &block.stmts {
        walk_stmt(ctx, s, &mut local, caps)?;
    }
    if let Some(t) = &block.tail {
        walk_expr(ctx, t, &mut local, caps)?;
    }
    Ok(())
}

fn walk_stmt(
    ctx: &crate::hir::lower::Ctx,
    stmt: &Stmt,
    declared: &mut HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
) -> Result<()> {
    match stmt {
        Stmt::Assign { name, value, span, .. } => {
            walk_expr(ctx, value, declared, caps)?;
            assign_name(ctx, name, declared, caps, span)?;
        }
        Stmt::FieldAssign { object, value, .. } => {
            walk_expr(ctx, object, declared, caps)?;
            walk_expr(ctx, value, declared, caps)?;
        }
        Stmt::IndexAssign { object, index, value, .. } => {
            walk_expr(ctx, object, declared, caps)?;
            walk_expr(ctx, index, declared, caps)?;
            walk_expr(ctx, value, declared, caps)?;
        }
        Stmt::Return { value: Some(v), .. } => walk_expr(ctx, v, declared, caps)?,
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            walk_expr(ctx, cond, declared, caps)?;
            walk_block(ctx, then_block, declared, caps)?;
            for (c, b) in elifs {
                walk_expr(ctx, c, declared, caps)?;
                walk_block(ctx, b, declared, caps)?;
            }
            if let Some(b) = else_block {
                walk_block(ctx, b, declared, caps)?;
            }
        }
        Stmt::For { iterator, start, end, step, body, .. } => {
            walk_expr(ctx, start, declared, caps)?;
            walk_expr(ctx, end, declared, caps)?;
            if let Some(s) = step {
                walk_expr(ctx, s, declared, caps)?;
            }
            let mut local = declared.clone();
            local.insert(*iterator);
            walk_block(ctx, body, &mut local, caps)?;
        }
        Stmt::While { cond, body, .. } => {
            walk_expr(ctx, cond, declared, caps)?;
            walk_block(ctx, body, declared, caps)?;
        }
        Stmt::Match { value, arms, .. } => walk_match(ctx, value, arms, declared, caps)?,
        Stmt::ExprStmt { expr, .. } => walk_expr(ctx, expr, declared, caps)?,
        Stmt::Attributed { stmt, .. } => walk_stmt(ctx, stmt, declared, caps)?,
        // 不引入捕获的语句（函数/类型声明等）：lambda body 中一般不可出现
        _ => {}
    }
    Ok(())
}

fn walk_match(
    ctx: &crate::hir::lower::Ctx,
    value: &Expr,
    arms: &[crate::parser::ast::stmt::MatchArm],
    declared: &mut HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
) -> Result<()> {
    walk_expr(ctx, value, declared, caps)?;
    for arm in arms {
        let mut local = declared.clone();
        for b in arm.pattern.bindings() {
            local.insert(b);
        }
        if let Some(g) = &arm.guard {
            walk_expr(ctx, g, &mut local, caps)?;
        }
        match &arm.body {
            crate::parser::ast::stmt::MatchBody::Expr(e) => walk_expr(ctx, e, &mut local, caps)?,
            crate::parser::ast::stmt::MatchBody::Block(b) => walk_block(ctx, b, &mut local, caps)?,
        }
    }
    Ok(())
}

fn walk_expr(
    ctx: &crate::hir::lower::Ctx,
    expr: &Expr,
    declared: &mut HashSet<Symbol>,
    caps: &mut Vec<(Symbol, HirType)>,
) -> Result<()> {
    match expr {
        Expr::Ident(name, span) => use_name(ctx, name, declared, caps, span)?,
        Expr::FnCall { name, args, span, .. } => {
            // 变量调用 `f(x)`：f 为局部/捕获时是一次使用；全局函数名不是捕获
            use_name(ctx, name, declared, caps, span)?;
            for a in args {
                walk_expr(ctx, a, declared, caps)?;
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            walk_expr(ctx, lhs, declared, caps)?;
            walk_expr(ctx, rhs, declared, caps)?;
        }
        Expr::Unary { arg, .. } => walk_expr(ctx, arg, declared, caps)?,
        Expr::If { cond, then_block, elifs, else_block, .. } => {
            walk_expr(ctx, cond, declared, caps)?;
            walk_block(ctx, then_block, declared, caps)?;
            for (c, b) in elifs {
                walk_expr(ctx, c, declared, caps)?;
                walk_block(ctx, b, declared, caps)?;
            }
            if let Some(b) = else_block {
                walk_block(ctx, b, declared, caps)?;
            }
        }
        Expr::Cast { expr, .. } => walk_expr(ctx, expr, declared, caps)?,
        Expr::Move(inner, _) => {
            // FnOnce：移出捕获由 HIR 检测并标记为 FnOnce（closure_lower.rs）
            walk_expr(ctx, inner, declared, caps)?;
        }
        Expr::Clone(inner, _) | Expr::ToUnique(inner, _) | Expr::TryOp(inner, _) => {
            walk_expr(ctx, inner, declared, caps)?;
        }
        Expr::MethodCall { object, args, .. } => {
            walk_expr(ctx, object, declared, caps)?;
            for a in args {
                walk_expr(ctx, a, declared, caps)?;
            }
        }
        Expr::FieldAccess { object, .. } => walk_expr(ctx, object, declared, caps)?,
        Expr::StructLiteral { fields, .. } => {
            for (_, v) in fields {
                walk_expr(ctx, v, declared, caps)?;
            }
        }
        Expr::ArrayLiteral(elems, _) => {
            for e in elems {
                walk_expr(ctx, e, declared, caps)?;
            }
        }
        Expr::ArraySized { count, .. } => walk_expr(ctx, count, declared, caps)?,
        Expr::ArrayRepeat { value, count, .. } => {
            walk_expr(ctx, value, declared, caps)?;
            walk_expr(ctx, count, declared, caps)?;
        }
        Expr::Ref(inner, _, _) => walk_expr(ctx, inner, declared, caps)?,
        Expr::Asm { outputs, inputs, .. } => {
            for (_, e) in outputs {
                walk_expr(ctx, e, declared, caps)?;
            }
            for (_, e) in inputs {
                walk_expr(ctx, e, declared, caps)?;
            }
        }
        Expr::Index { object, index, .. } => {
            walk_expr(ctx, object, declared, caps)?;
            walk_expr(ctx, index, declared, caps)?;
        }
        Expr::CallExpr { target, args, .. } => {
            walk_expr(ctx, target, declared, caps)?;
            for a in args {
                walk_expr(ctx, a, declared, caps)?;
            }
        }
        Expr::Match { value, arms, .. } => walk_match(ctx, value, arms, declared, caps)?,
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for a in tuple_args {
                walk_expr(ctx, a, declared, caps)?;
            }
            for (_, a) in named_args {
                walk_expr(ctx, a, declared, caps)?;
            }
        }
        Expr::Lambda { params, body, .. } => {
            // 嵌套 lambda：其对外层函数变量的引用同样需要外层捕获
            let mut local = declared.clone();
            for (n, _) in params {
                local.insert(*n);
            }
            walk_block(ctx, body, &mut local, caps)?;
        }
        Expr::MacroCall { args, .. } => {
            for a in args {
                walk_expr(ctx, a, declared, caps)?;
            }
        }
        Expr::Literal(_) | Expr::Suffixed { .. } | Expr::Null(_) => {}
    }
    Ok(())
}
