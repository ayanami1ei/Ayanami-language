use super::*;

// ── 局部名收集（供 FnCall 判定：可能是 FnPtr 局部变量调用） ──

pub(super) fn collect_local_names_block(block: &Block, out: &mut Vec<Symbol>) {
    for s in &block.stmts {
        collect_local_names_stmt(s, out);
    }
    if let Some(t) = &block.tail {
        collect_local_names_expr(t, out);
    }
}

pub(super) fn collect_local_names_stmt(stmt: &Stmt, out: &mut Vec<Symbol>) {
    match stmt {
        Stmt::Assign { name, value, .. } => {
            out.push(*name);
            collect_local_names_expr(value, out);
        }
        Stmt::FieldAssign { object, value, .. } => {
            collect_local_names_expr(object, out);
            collect_local_names_expr(value, out);
        }
        Stmt::IndexAssign { object, index, value, .. } => {
            collect_local_names_expr(object, out);
            collect_local_names_expr(index, out);
            collect_local_names_expr(value, out);
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value { collect_local_names_expr(v, out); }
        }
        Stmt::If { cond, then_block, elifs, else_block, .. } => {
            collect_local_names_expr(cond, out);
            collect_local_names_block(then_block, out);
            for (c, b) in elifs {
                collect_local_names_expr(c, out);
                collect_local_names_block(b, out);
            }
            if let Some(b) = else_block { collect_local_names_block(b, out); }
        }
        Stmt::For { iterator, start, end, step, body, .. } => {
            out.push(*iterator);
            collect_local_names_expr(start, out);
            collect_local_names_expr(end, out);
            if let Some(s) = step { collect_local_names_expr(s, out); }
            collect_local_names_block(body, out);
        }
        Stmt::While { cond, body, .. } => {
            collect_local_names_expr(cond, out);
            collect_local_names_block(body, out);
        }
        Stmt::ForIn { iterator, iterable, body, .. } => {
            out.push(*iterator);
            collect_local_names_expr(iterable, out);
            collect_local_names_block(body, out);
        }
        Stmt::Match { value, arms, .. } => {
            collect_local_names_expr(value, out);
            for a in arms {
                out.extend(a.pattern.bindings());
                if let Some(g) = &a.guard { collect_local_names_expr(g, out); }
                match &a.body {
                    crate::parser::ast::stmt::MatchBody::Expr(e) => collect_local_names_expr(e, out),
                    crate::parser::ast::stmt::MatchBody::Block(b) => collect_local_names_block(b, out),
                }
            }
        }
        Stmt::ExprStmt { expr, .. } => collect_local_names_expr(expr, out),
        Stmt::Attributed { stmt, .. } => collect_local_names_stmt(stmt, out),
        _ => {}
    }
}

pub(super) fn collect_local_names_expr(expr: &Expr, out: &mut Vec<Symbol>) {
    match expr {
        Expr::Binary { lhs, rhs, .. } => {
            collect_local_names_expr(lhs, out);
            collect_local_names_expr(rhs, out);
        }
        Expr::Unary { arg, .. } => collect_local_names_expr(arg, out),
        Expr::If { cond, then_block, elifs, else_block, .. } => {
            collect_local_names_expr(cond, out);
            collect_local_names_block(then_block, out);
            for (c, b) in elifs {
                collect_local_names_expr(c, out);
                collect_local_names_block(b, out);
            }
            if let Some(b) = else_block { collect_local_names_block(b, out); }
        }
        Expr::Cast { expr, .. } => collect_local_names_expr(expr, out),
        Expr::FnCall { args, .. } | Expr::MethodCall { args, .. }
        | Expr::CallExpr { args, .. } | Expr::MacroCall { args, .. } => {
            if let Expr::MethodCall { object, .. } | Expr::CallExpr { target: object, .. } = expr {
                collect_local_names_expr(object, out);
            }
            for a in args { collect_local_names_expr(a, out); }
        }
        Expr::Move(i, _) | Expr::Clone(i, _) | Expr::ToUnique(i, _)
        | Expr::Ref(i, _, _) | Expr::TryOp(i, _) => collect_local_names_expr(i, out),
        Expr::FieldAccess { object, .. } => collect_local_names_expr(object, out),
        Expr::StructLiteral { fields, .. } => {
            for (_, v) in fields { collect_local_names_expr(v, out); }
        }
        Expr::ArrayLiteral(elems, _) => {
            for e in elems { collect_local_names_expr(e, out); }
        }
        Expr::ArraySized { count, .. } => collect_local_names_expr(count, out),
        Expr::Asm { outputs, inputs, .. } => {
            for (_, e) in outputs { collect_local_names_expr(e, out); }
            for (_, e) in inputs { collect_local_names_expr(e, out); }
        }
        Expr::Index { object, index, .. } => {
            collect_local_names_expr(object, out);
            collect_local_names_expr(index, out);
        }
        Expr::Match { value, arms, .. } => {
            collect_local_names_expr(value, out);
            for a in arms {
                out.extend(a.pattern.bindings());
                if let Some(g) = &a.guard { collect_local_names_expr(g, out); }
                match &a.body {
                    crate::parser::ast::stmt::MatchBody::Expr(e) => collect_local_names_expr(e, out),
                    crate::parser::ast::stmt::MatchBody::Block(b) => collect_local_names_block(b, out),
                }
            }
        }
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for a in tuple_args { collect_local_names_expr(a, out); }
            for (_, a) in named_args { collect_local_names_expr(a, out); }
        }
        Expr::Lambda { params, body, .. } => {
            for (n, _) in params { out.push(*n); }
            for s in &body.stmts { collect_local_names_stmt(s, out); }
            if let Some(t) = &body.tail { collect_local_names_expr(t, out); }
        }
        _ => {}
    }
}
