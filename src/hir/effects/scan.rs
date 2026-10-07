//! A3a-2：AST 观测点扫描（精确到调用点/`?` 的行列）。

use std::collections::HashMap;
use super::IO_NAMES;

/// AST 观测点（用于精确行列定位）。
#[derive(Clone)]
pub(super) enum Obs {
    Io(usize, usize),
    Try(usize, usize),
    Alloc(usize, usize),
    State(usize, usize),
}

impl Obs {
    /// 按效应名给出定位。
    pub(super) fn site_of(&self, kind: &str) -> Option<(usize, usize)> {
        match (self, kind) {
            (Obs::Io(l, c), "io") => Some((*l, *c)),
            (Obs::Try(l, c), "throws") => Some((*l, *c)),
            (Obs::Alloc(l, c), "alloc") => Some((*l, *c)),
            (Obs::State(l, c), "state") => Some((*l, *c)),
            _ => None,
        }
    }
}

pub(super) fn collect_ast_observations(program: &crate::parser::ast::Program) -> HashMap<String, Vec<Obs>> {
    let mut map = HashMap::new();
    for s in &program.stmts {
        scan_decl(s, "", &mut map);
    }
    map
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_string() } else { format!("{}.{}", prefix, name) }
}

fn scan_decl(stmt: &crate::parser::ast::Stmt, prefix: &str, map: &mut HashMap<String, Vec<Obs>>) {
    use crate::parser::ast::Stmt;
    match stmt {
        Stmt::FnDecl { name, body, .. } => {
            let mut obs = Vec::new();
            for s in &body.stmts {
                scan_body(s, &mut obs);
            }
            map.insert(qualify(prefix, &name.as_str()), obs);
        }
        Stmt::Namespace { name, items, .. } => {
            let p = qualify(prefix, &name.as_str());
            for s in items { scan_decl(s, &p, map); }
        }
        Stmt::ImplBlock { methods, .. } => {
            for m in methods { scan_decl(m, prefix, map); }
        }
        Stmt::Attributed { stmt, .. } => scan_decl(stmt, prefix, map),
        _ => {}
    }
}

fn scan_body(stmt: &crate::parser::ast::Stmt, obs: &mut Vec<Obs>) {
    use crate::parser::ast::Stmt;
    match stmt {
        Stmt::Assign { value, .. } => scan_expr(value, obs),
        Stmt::FieldAssign { object, value, span, .. } => {
            obs.push(Obs::State(span.start_line, span.start_col));
            scan_expr(object, obs);
            scan_expr(value, obs);
        }
        Stmt::IndexAssign { object, index, value, span } => {
            obs.push(Obs::State(span.start_line, span.start_col));
            scan_expr(object, obs);
            scan_expr(index, obs);
            scan_expr(value, obs);
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value { scan_expr(v, obs); }
        }
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            scan_expr(cond, obs);
            for s in &then_block.stmts { scan_body(s, obs); }
            for (c, b) in elifs {
                scan_expr(c, obs);
                for s in &b.stmts { scan_body(s, obs); }
            }
            if let Some(b) = else_block {
                for s in &b.stmts { scan_body(s, obs); }
            }
        }
        Stmt::For { start, end, step, body, .. } => {
            scan_expr(start, obs);
            scan_expr(end, obs);
            if let Some(s) = step { scan_expr(s, obs); }
            for s in &body.stmts { scan_body(s, obs); }
        }
        Stmt::While { cond, body, .. } => {
            scan_expr(cond, obs);
            for s in &body.stmts { scan_body(s, obs); }
        }
        Stmt::ForIn { iterable, body, .. } => {
            scan_expr(iterable, obs);
            for s in &body.stmts { scan_body(s, obs); }
        }
        Stmt::Match { value, arms, .. } => {
            scan_expr(value, obs);
            for a in arms {
                match &a.body {
                    crate::parser::ast::stmt::MatchBody::Expr(e) => scan_expr(e, obs),
                    crate::parser::ast::stmt::MatchBody::Block(b) => {
                        for s in &b.stmts { scan_body(s, obs); }
                        if let Some(t) = &b.tail { scan_expr(t, obs); }
                    }
                }
            }
        }
        Stmt::ExprStmt { expr, .. } => scan_expr(expr, obs),
        Stmt::Attributed { stmt, .. } => scan_body(stmt, obs),
        _ => {}
    }
}

fn scan_expr(e: &crate::parser::ast::Expr, obs: &mut Vec<Obs>) {
    use crate::parser::ast::Expr;
    match e {
        Expr::Binary { lhs, rhs, .. } => {
            scan_expr(lhs, obs);
            scan_expr(rhs, obs);
        }
        Expr::Unary { arg, .. } => scan_expr(arg, obs),
        Expr::If { cond, then_block, elifs, else_block, .. } => {
            scan_expr(cond, obs);
            for s in &then_block.stmts { scan_body(s, obs); }
            if let Some(t) = &then_block.tail { scan_expr(t, obs); }
            for (c, b) in elifs {
                scan_expr(c, obs);
                for s in &b.stmts { scan_body(s, obs); }
                if let Some(t) = &b.tail { scan_expr(t, obs); }
            }
            if let Some(b) = else_block {
                for s in &b.stmts { scan_body(s, obs); }
                if let Some(t) = &b.tail { scan_expr(t, obs); }
            }
        }
        Expr::Cast { expr, .. } => scan_expr(expr, obs),
        Expr::MacroCall { args, .. } => {
            for a in args {
                match a {
                    crate::parser::ast::expr::MacroArg::Expr(e) => scan_expr(e, obs),
                    crate::parser::ast::expr::MacroArg::Block(_) => {}
                }
            }
        }
        Expr::FnCall { name, args, span, .. } => {
            if IO_NAMES.contains(&name.as_str().as_str()) {
                obs.push(Obs::Io(span.start_line, span.start_col));
            }
            for a in args { scan_expr(a, obs); }
        }
        Expr::Move(i, _) | Expr::Clone(i, _) | Expr::Ref(i, _, _) => {
            scan_expr(i, obs);
        }
        Expr::ToUnique(i, span) => {
            obs.push(Obs::Alloc(span.start_line, span.start_col));
            scan_expr(i, obs);
        }
        Expr::MethodCall { object, args, .. } => {
            scan_expr(object, obs);
            for a in args { scan_expr(a, obs); }
        }
        Expr::FieldAccess { object, .. } => scan_expr(object, obs),
        Expr::StructLiteral { fields, .. } => {
            for (_, v) in fields { scan_expr(v, obs); }
        }
        Expr::ArrayLiteral(elems, span) => {
            obs.push(Obs::Alloc(span.start_line, span.start_col));
            for x in elems { scan_expr(x, obs); }
        }
        Expr::ArrayRepeat { value, count, span } => {
            obs.push(Obs::Alloc(span.start_line, span.start_col));
            scan_expr(value, obs);
            scan_expr(count, obs);
        }
        Expr::ArraySized { count, span, .. } => {
            obs.push(Obs::Alloc(span.start_line, span.start_col));
            scan_expr(count, obs);
        }
        Expr::Asm { outputs, inputs, .. } => {
            for (_, x) in outputs { scan_expr(x, obs); }
            for (_, x) in inputs { scan_expr(x, obs); }
        }
        Expr::Index { object, index, .. } => {
            scan_expr(object, obs);
            scan_expr(index, obs);
        }
        Expr::CallExpr { target, args, .. } => {
            scan_expr(target, obs);
            for a in args { scan_expr(a, obs); }
        }
        Expr::TryOp(inner, span) => {
            obs.push(Obs::Try(span.start_line, span.start_col));
            scan_expr(inner, obs);
        }
        Expr::Match { value, arms, .. } => {
            scan_expr(value, obs);
            for a in arms {
                match &a.body {
                    crate::parser::ast::stmt::MatchBody::Expr(e) => scan_expr(e, obs),
                    crate::parser::ast::stmt::MatchBody::Block(b) => {
                        for s in &b.stmts { scan_body(s, obs); }
                        if let Some(t) = &b.tail { scan_expr(t, obs); }
                    }
                }
            }
        }
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for x in tuple_args { scan_expr(x, obs); }
            for (_, x) in named_args { scan_expr(x, obs); }
        }
        Expr::Lambda { body, .. } => {
            for s in &body.stmts { scan_body(s, obs); }
            if let Some(t) = &body.tail { scan_expr(t, obs); }
        }
        Expr::Literal(lit) => {
            if matches!(lit, crate::parser::ast::Literal::String(..)) {
                let span = lit.span();
                obs.push(Obs::Alloc(span.start_line, span.start_col));
            }
        }
        Expr::Ident(..) | Expr::Null(_) | Expr::Suffixed { .. } => {}
    }
}
