//! #121 / C1：泛型体 eager 检查。
//!
//! - 名称检查：未定义自由函数（不依赖特化，覆盖从未实例化的方法体）；
//! - C1：泛型参数 `T` 上的方法调用按约束 `[T: Iface]` 检查（浅层类型推断；
//!   类型不确定时跳过，不误报）；`T` 上字段访问报错。
//!
//! `ns.fn(...)` / `Enum::Variant(...)` 等点号形式跳过（命名空间/枚举成员另有解析路径）。

use super::*;
use super::generic_types::GType;

/// 检查上下文：当前泛型参数与约束、已知名（泛型参数+局部名）、变量浅层类型。
struct CheckCtx {
    /// 当前泛型参数与约束（impl + 方法合并）
    gp: Vec<(Symbol, Option<Symbol>)>,
    /// 泛型参数 + 局部名（`f(x)` 中 `f` 可能是 FnPtr 局部变量）
    known: Vec<Symbol>,
    /// 变量 → 浅层类型
    env: HashMap<Symbol, GType>,
}

impl crate::hir::lower::Ctx {
    /// 对全部泛型函数/泛型 impl 方法体做未定义函数名与 T 上方法约束检查。
    pub(crate) fn check_generic_bodies(&self) -> Result<()> {
        for (_, gp, stmt) in &self.generic_fns {
            if let Stmt::FnDecl { params, body, .. } = stmt {
                let mut cx = CheckCtx {
                    gp: gp.clone(),
                    known: gp.iter().map(|(n, _)| *n).collect(),
                    env: HashMap::new(),
                };
                for (n, t) in params {
                    cx.known.push(*n);
                    let ty = self.ast_gtype(t, &cx.gp);
                    cx.env.insert(*n, ty);
                }
                super::generic_locals::collect_local_names_block(body, &mut cx.known);
                self.check_block_names(body, &mut cx)?;
            }
        }
        Ok(())
    }

    fn check_block_names(&self, block: &Block, cx: &mut CheckCtx) -> Result<()> {
        for s in &block.stmts {
            self.check_stmt_names(s, cx)?;
        }
        if let Some(t) = &block.tail {
            self.check_expr_names(t, cx)?;
        }
        Ok(())
    }

    fn check_stmt_names(&self, stmt: &Stmt, cx: &mut CheckCtx) -> Result<()> {
        match stmt {
            Stmt::Assign { name, value, .. } => {
                self.check_expr_names(value, cx)?;
                let ty = self.infer_gtype(value, &cx.gp, &cx.env);
                cx.env.insert(*name, ty);
                Ok(())
            }
            Stmt::FieldAssign { object, value, .. } => {
                self.check_expr_names(object, cx)?;
                self.check_expr_names(value, cx)
            }
            Stmt::IndexAssign { object, index, value, .. } => {
                self.check_expr_names(object, cx)?;
                self.check_expr_names(index, cx)?;
                self.check_expr_names(value, cx)
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value { self.check_expr_names(v, cx)?; }
                Ok(())
            }
            Stmt::If { cond, then_block, elifs, else_block, .. } => {
                self.check_expr_names(cond, cx)?;
                self.check_block_names(then_block, cx)?;
                for (c, b) in elifs {
                    self.check_expr_names(c, cx)?;
                    self.check_block_names(b, cx)?;
                }
                if let Some(b) = else_block { self.check_block_names(b, cx)?; }
                Ok(())
            }
            Stmt::For { iterator, start, end, step, body, .. } => {
                self.check_expr_names(start, cx)?;
                self.check_expr_names(end, cx)?;
                if let Some(s) = step { self.check_expr_names(s, cx)?; }
                cx.env.insert(*iterator, GType::Other);
                self.check_block_names(body, cx)
            }
            Stmt::While { cond, body, .. } => {
                self.check_expr_names(cond, cx)?;
                self.check_block_names(body, cx)
            }
            Stmt::Match { value, arms, .. } => {
                self.check_expr_names(value, cx)?;
                for a in arms {
                    for (n, _) in &a.bindings {
                        cx.env.insert(*n, GType::Other);
                    }
                    self.check_expr_names(&a.body, cx)?;
                }
                Ok(())
            }
            Stmt::ExprStmt { expr, .. } => self.check_expr_names(expr, cx),
            Stmt::Attributed { stmt, .. } => self.check_stmt_names(stmt, cx),
            _ => Ok(()),
        }
    }

    fn check_expr_names(&self, expr: &Expr, cx: &mut CheckCtx) -> Result<()> {
        match expr {
            Expr::FnCall { name, args, span, .. } => {
                let known = name.as_str().contains('.')
                    || cx.known.contains(name)
                    || self.fn_map.contains_key(name)
                    || self.generic_fns.iter().any(|(n, _, _)| n == name);
                if !known {
                    return Err(Error::Hir(format!(
                        "undefined function `{}` (at {}:{})",
                        name.as_str(), span.start_line, span.start_col
                    )));
                }
                for a in args { self.check_expr_names(a, cx)?; }
                Ok(())
            }
            Expr::MethodCall { object, method, args, span } => {
                // C1：接收者为泛型参数 T 时按约束检查
                if let GType::Param(p) = self.infer_gtype(object, &cx.gp, &cx.env) {
                    self.check_param_method(&p, method, args.len(), span, &cx.gp)?;
                }
                self.check_expr_names(object, cx)?;
                for a in args { self.check_expr_names(a, cx)?; }
                Ok(())
            }
            Expr::FieldAccess { object, field, span } => {
                if let GType::Param(p) = self.infer_gtype(object, &cx.gp, &cx.env) {
                    return Err(Error::Hir(format!(
                        "field access `{}` on generic parameter `{}` is not allowed; use a bound method (at {}:{})",
                        field.as_str(), p.as_str(), span.start_line, span.start_col
                    )));
                }
                self.check_expr_names(object, cx)
            }
            Expr::Binary { op, lhs, rhs, span } => {
                // C2：T 参与的运算符按约束检查（映射到 add/sub/.../eq/...）
                if let GType::Param(p) = self.infer_gtype(lhs, &cx.gp, &cx.env) {
                    let op_name = binary_op_to_fn_name(op);
                    self.check_param_operator(&p, op_name, 1, span, &cx.gp, &format!("{:?}", op))?;
                }
                self.check_expr_names(lhs, cx)?;
                self.check_expr_names(rhs, cx)
            }
            Expr::Unary { op, arg, span } => {
                if let GType::Param(p) = self.infer_gtype(arg, &cx.gp, &cx.env) {
                    let op_name = unary_op_to_fn_name(op);
                    self.check_param_operator(&p, op_name, 0, span, &cx.gp, &format!("{:?}", op))?;
                }
                self.check_expr_names(arg, cx)
            }
            Expr::If { cond, then_block, elifs, else_block, .. } => {
                self.check_expr_names(cond, cx)?;
                self.check_block_names(then_block, cx)?;
                for (c, b) in elifs {
                    self.check_expr_names(c, cx)?;
                    self.check_block_names(b, cx)?;
                }
                if let Some(b) = else_block { self.check_block_names(b, cx)?; }
                Ok(())
            }
            Expr::Cast { expr, .. } => self.check_expr_names(expr, cx),
            Expr::Literal(_) | Expr::Suffixed { .. } | Expr::Ident(..) | Expr::Null(_) => Ok(()),
            Expr::Move(i, _) | Expr::Clone(i, _) | Expr::ToUnique(i, _)
            | Expr::Ref(i, _, _) | Expr::TryOp(i, _) => self.check_expr_names(i, cx),
            Expr::StructLiteral { fields, .. } => {
                for (_, v) in fields { self.check_expr_names(v, cx)?; }
                Ok(())
            }
            Expr::ArrayLiteral(elems, _) => {
                for e in elems { self.check_expr_names(e, cx)?; }
                Ok(())
            }
            Expr::ArraySized { count, .. } => self.check_expr_names(count, cx),
            Expr::Asm { outputs, inputs, .. } => {
                for (_, e) in outputs { self.check_expr_names(e, cx)?; }
                for (_, e) in inputs { self.check_expr_names(e, cx)?; }
                Ok(())
            }
            Expr::Index { object, index, span } => {
                if let GType::Param(p) = self.infer_gtype(object, &cx.gp, &cx.env) {
                    self.check_param_operator(&p, Some("index"), 1, span, &cx.gp, "[]")?;
                }
                self.check_expr_names(object, cx)?;
                self.check_expr_names(index, cx)
            }
            Expr::CallExpr { target, args, .. } => {
                self.check_expr_names(target, cx)?;
                for a in args { self.check_expr_names(a, cx)?; }
                Ok(())
            }
            Expr::Match { value, arms, .. } => {
                self.check_expr_names(value, cx)?;
                for a in arms {
                    for (n, _) in &a.bindings {
                        cx.env.insert(*n, GType::Other);
                    }
                    self.check_expr_names(&a.body, cx)?;
                }
                Ok(())
            }
            Expr::EnumConstruct { tuple_args, named_args, .. } => {
                for a in tuple_args { self.check_expr_names(a, cx)?; }
                for (_, a) in named_args { self.check_expr_names(a, cx)?; }
                Ok(())
            }
            Expr::Lambda { params, body, .. } => {
                // lambda 作用域：保存/恢复 env，避免形参类型泄漏到外层
                let saved = cx.env.clone();
                for (n, t) in params {
                    let ty = self.ast_gtype(t, &cx.gp);
                    cx.env.insert(*n, ty);
                }
                for s in body { self.check_stmt_names(s, cx)?; }
                cx.env = saved;
                Ok(())
            }
            Expr::MacroCall { args, .. } => {
                for a in args { self.check_expr_names(a, cx)?; }
                Ok(())
            }
        }
    }
}
