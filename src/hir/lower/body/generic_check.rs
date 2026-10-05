//! #121：泛型体 eager 名称检查。
//!
//! 惰性单态化下，从未被实例化的泛型方法/函数体完全不检查（错误静默）。
//! 这里在收集阶段对全部 `generic_fns` 体做**自由函数名**检查：
//! 报未定义函数（如 `undefined_fn_xyz()`），不依赖特化。
//!
//! 范围与边界：不做类型检查——泛型参数 `T` 上的字段/方法访问需特化才能判定；
//! `ns.fn(...)` / `Enum::Variant(...)` 等点号形式跳过（命名空间/枚举成员另有解析路径）。

use super::*;

impl crate::hir::lower::Ctx {
    /// 对全部泛型函数/泛型 impl 方法体做未定义函数名检查。
    /// `known` = 泛型参数 + 局部名（形参/赋值/for 变量/match 绑定/lambda 形参）——
    /// `f(x)` 中的 `f` 可能是 FnPtr 局部变量而非自由函数。
    pub(crate) fn check_generic_bodies(&self) -> Result<()> {
        for (_, gp, stmt) in &self.generic_fns {
            let mut known: Vec<Symbol> = gp.iter().map(|(n, _)| *n).collect();
            if let Stmt::FnDecl { params, body, .. } = stmt {
                known.extend(params.iter().map(|(n, _)| *n));
                collect_local_names_block(body, &mut known);
                self.check_block_names(body, &known)?;
            }
        }
        Ok(())
    }

    fn check_block_names(&self, block: &Block, gp: &[Symbol]) -> Result<()> {
        for s in &block.stmts {
            self.check_stmt_names(s, gp)?;
        }
        if let Some(t) = &block.tail {
            self.check_expr_names(t, gp)?;
        }
        Ok(())
    }

    fn check_stmt_names(&self, stmt: &Stmt, gp: &[Symbol]) -> Result<()> {
        match stmt {
            Stmt::Assign { value, .. } => self.check_expr_names(value, gp),
            Stmt::FieldAssign { object, value, .. } => {
                self.check_expr_names(object, gp)?;
                self.check_expr_names(value, gp)
            }
            Stmt::IndexAssign { object, index, value, .. } => {
                self.check_expr_names(object, gp)?;
                self.check_expr_names(index, gp)?;
                self.check_expr_names(value, gp)
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value { self.check_expr_names(v, gp)?; }
                Ok(())
            }
            Stmt::If { cond, then_block, elifs, else_block, .. } => {
                self.check_expr_names(cond, gp)?;
                self.check_block_names(then_block, gp)?;
                for (c, b) in elifs {
                    self.check_expr_names(c, gp)?;
                    self.check_block_names(b, gp)?;
                }
                if let Some(b) = else_block { self.check_block_names(b, gp)?; }
                Ok(())
            }
            Stmt::For { start, end, step, body, .. } => {
                self.check_expr_names(start, gp)?;
                self.check_expr_names(end, gp)?;
                if let Some(s) = step { self.check_expr_names(s, gp)?; }
                self.check_block_names(body, gp)
            }
            Stmt::While { cond, body, .. } => {
                self.check_expr_names(cond, gp)?;
                self.check_block_names(body, gp)
            }
            Stmt::Match { value, arms, .. } => {
                self.check_expr_names(value, gp)?;
                for a in arms { self.check_expr_names(&a.body, gp)?; }
                Ok(())
            }
            Stmt::ExprStmt { expr, .. } => self.check_expr_names(expr, gp),
            Stmt::Attributed { stmt, .. } => self.check_stmt_names(stmt, gp),
            _ => Ok(()),
        }
    }

    fn check_expr_names(&self, expr: &Expr, gp: &[Symbol]) -> Result<()> {
        match expr {
            Expr::FnCall { name, args, span, .. } => {
                let known = name.as_str().contains('.')
                    || gp.contains(name)
                    || self.fn_map.contains_key(name)
                    || self.generic_fns.iter().any(|(n, _, _)| n == name);
                if !known {
                    return Err(Error::Hir(format!(
                        "undefined function `{}` (at {}:{})",
                        name.as_str(), span.start_line, span.start_col
                    )));
                }
                for a in args { self.check_expr_names(a, gp)?; }
                Ok(())
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.check_expr_names(lhs, gp)?;
                self.check_expr_names(rhs, gp)
            }
            Expr::Unary { arg, .. } => self.check_expr_names(arg, gp),
            Expr::If { cond, then_block, elifs, else_block, .. } => {
                self.check_expr_names(cond, gp)?;
                self.check_block_names(then_block, gp)?;
                for (c, b) in elifs {
                    self.check_expr_names(c, gp)?;
                    self.check_block_names(b, gp)?;
                }
                if let Some(b) = else_block { self.check_block_names(b, gp)?; }
                Ok(())
            }
            Expr::Cast { expr, .. } => self.check_expr_names(expr, gp),
            Expr::Literal(_) | Expr::Suffixed { .. } | Expr::Ident(..) | Expr::Null(_) => Ok(()),
            Expr::Move(i, _) | Expr::Clone(i, _) | Expr::ToUnique(i, _)
            | Expr::Ref(i, _, _) | Expr::TryOp(i, _) => self.check_expr_names(i, gp),
            Expr::MethodCall { object, args, .. } => {
                self.check_expr_names(object, gp)?;
                for a in args { self.check_expr_names(a, gp)?; }
                Ok(())
            }
            Expr::FieldAccess { object, .. } => self.check_expr_names(object, gp),
            Expr::StructLiteral { fields, .. } => {
                for (_, v) in fields { self.check_expr_names(v, gp)?; }
                Ok(())
            }
            Expr::ArrayLiteral(elems, _) => {
                for e in elems { self.check_expr_names(e, gp)?; }
                Ok(())
            }
            Expr::ArraySized { count, .. } => self.check_expr_names(count, gp),
            Expr::Asm { outputs, inputs, .. } => {
                for (_, e) in outputs { self.check_expr_names(e, gp)?; }
                for (_, e) in inputs { self.check_expr_names(e, gp)?; }
                Ok(())
            }
            Expr::Index { object, index, .. } => {
                self.check_expr_names(object, gp)?;
                self.check_expr_names(index, gp)
            }
            Expr::CallExpr { target, args, .. } => {
                self.check_expr_names(target, gp)?;
                for a in args { self.check_expr_names(a, gp)?; }
                Ok(())
            }
            Expr::Match { value, arms, .. } => {
                self.check_expr_names(value, gp)?;
                for a in arms { self.check_expr_names(&a.body, gp)?; }
                Ok(())
            }
            Expr::EnumConstruct { tuple_args, named_args, .. } => {
                for a in tuple_args { self.check_expr_names(a, gp)?; }
                for (_, a) in named_args { self.check_expr_names(a, gp)?; }
                Ok(())
            }
            Expr::Lambda { body, .. } => {
                for s in body { self.check_stmt_names(s, gp)?; }
                Ok(())
            }
            Expr::MacroCall { args, .. } => {
                for a in args { self.check_expr_names(a, gp)?; }
                Ok(())
            }
        }
    }
}

// ── 局部名收集（供 FnCall 判定：可能是 FnPtr 局部变量调用） ──

fn collect_local_names_block(block: &Block, out: &mut Vec<Symbol>) {
    for s in &block.stmts {
        collect_local_names_stmt(s, out);
    }
    if let Some(t) = &block.tail {
        collect_local_names_expr(t, out);
    }
}

fn collect_local_names_stmt(stmt: &Stmt, out: &mut Vec<Symbol>) {
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
        Stmt::Match { value, arms, .. } => {
            collect_local_names_expr(value, out);
            for a in arms {
                for (n, _) in &a.bindings { out.push(*n); }
                collect_local_names_expr(&a.body, out);
            }
        }
        Stmt::ExprStmt { expr, .. } => collect_local_names_expr(expr, out),
        Stmt::Attributed { stmt, .. } => collect_local_names_stmt(stmt, out),
        _ => {}
    }
}

fn collect_local_names_expr(expr: &Expr, out: &mut Vec<Symbol>) {
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
                for (n, _) in &a.bindings { out.push(*n); }
                collect_local_names_expr(&a.body, out);
            }
        }
        Expr::EnumConstruct { tuple_args, named_args, .. } => {
            for a in tuple_args { collect_local_names_expr(a, out); }
            for (_, a) in named_args { collect_local_names_expr(a, out); }
        }
        Expr::Lambda { params, body, .. } => {
            for (n, _) in params { out.push(*n); }
            for s in body { collect_local_names_stmt(s, out); }
        }
        _ => {}
    }
}
