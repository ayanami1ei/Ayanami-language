//! C3 泛型体声明期检查：表达式名字解析（从 generic_check.rs 拆出，保持文件 ≤300 行）。
use super::*;
use super::generic_check::CheckCtx;
use super::generic_types::{GType, hir_is_concrete_primitive};

impl crate::hir::lower::Ctx {
    pub(super) fn check_expr_names(&self, expr: &Expr, cx: &mut CheckCtx) -> Result<()> {
        match expr {
            Expr::ArrayRepeat { value, count, .. } => {
                self.check_expr_names(value, cx)?;
                self.check_expr_names(count, cx)?;
                return Ok(());
            }
            Expr::FnCall { name, args, generic_args, span } => {
                // Phase 7：size_of/align_of 为编译期内置（仅显式类型实参形态）
                let layout_builtin = matches!(name.as_str().as_str(), "size_of" | "align_of")
                    && !generic_args.is_empty();
                let known = name.as_str().contains('.')
                    || name.as_str() == "static_assert"
                    || layout_builtin
                    || cx.known.contains(name)
                    || self.fn_map.contains_key(name)
                    || self.generic_fns.iter().any(|(n, ..)| n == name);
                if !known {
                    return Err(Error::Hir(format!(
                        "undefined function `{}` (at {}:{})",
                        name.as_str(), span.start_line, span.start_col
                    )));
                }
                // C3a：单候选自由函数实参——T 流入具体基元形参
                if !cx.env.contains_key(name) {
                    if let Some(ids) = self.fn_map.get(name) {
                        let mut cands = ids.iter()
                            .map(|id| &self.fns[id.0])
                            .filter(|s| s.params.len().saturating_sub(s.hidden) == args.len());
                        if let Some(sig) = cands.next() {
                            if cands.next().is_none() {
                                for (i, a) in args.iter().enumerate() {
                                    if let GType::Param(p) = self.infer_gtype(a, &cx.gp, &cx.env) {
                                        if let Some((_, pt)) = sig.params.get(i) {
                                            if hir_is_concrete_primitive(pt) {
                                                let sp = a.span();
                                                return Err(Error::Hir(format!(
                                                    "passing generic parameter `{}` where `{}` is expected in call to `{}` (at {}:{})",
                                                    p.as_str(), hir_type_display(pt), name.as_str(),
                                                    sp.start_line, sp.start_col
                                                )));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
                    for n in a.pattern.bindings() {
                        cx.env.insert(n, GType::Other);
                    }
                    if let Some(g) = &a.guard { self.check_expr_names(g, cx)?; }
                    match &a.body {
                        crate::parser::ast::stmt::MatchBody::Expr(e) => self.check_expr_names(e, cx)?,
                        crate::parser::ast::stmt::MatchBody::Block(b) => self.check_block_names(b, cx)?,
                    }
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
                for s in &body.stmts { self.check_stmt_names(s, cx)?; }
                if let Some(t) = &body.tail { self.check_expr_names(t, cx)?; }
                cx.env = saved;
                Ok(())
            }
            Expr::MacroCall { args, .. } => {
                for a in args {
                    match a {
                        crate::parser::ast::expr::MacroArg::Expr(e) => self.check_expr_names(e, cx)?,
                        crate::parser::ast::expr::MacroArg::Block(_) => {}
                    }
                }
                Ok(())
            }
        }
    }
}
