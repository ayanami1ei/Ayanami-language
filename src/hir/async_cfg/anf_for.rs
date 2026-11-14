//! 数值 for 体内 await：降为带迭代变量类型 cast 的 while。
use crate::hir::async_mixed_types::hir_to_ast;
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, Block, Expr, Literal, Stmt, Type};

use super::anf::{Anf, expr_has_await, stmt_has_await_local};

impl Anf<'_> {
    pub(super) fn for_stmt(
        &mut self,
        iterator: crate::intern::Symbol,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        body: &Block,
        span: crate::span::Span,
    ) -> Option<Stmt> {
        let body_await = body.stmts.iter().any(stmt_has_await_local);
        if expr_has_await(start) || expr_has_await(end) || step.is_some_and(expr_has_await) {
            return None;
        }
        if !body_await {
            return Some(Stmt::For {
                iterator,
                start: start.clone(),
                end: end.clone(),
                step: step.cloned(),
                body: body.clone(),
                span,
            });
        }
        let iter_ty = self
            .probe
            .and_then(|m| m.get(&iterator))
            .and_then(|t| hir_to_ast(t, span))?;
        let body = self.block(body)?;
        let cast = |e: Expr| Expr::Cast {
            expr: Box::new(e),
            ty: iter_ty.clone(),
            span,
        };
        let init = Stmt::Assign {
            name: iterator,
            is_mut: false,
            value: cast(start.clone()),
            span,
        };
        let cond = Expr::Binary {
            op: BinaryOp::Lt,
            lhs: Box::new(Expr::Ident(iterator, span)),
            rhs: Box::new(cast(end.clone())),
            span,
        };
        let step_expr = step
            .cloned()
            .unwrap_or_else(|| Expr::Literal(Literal::Int(1, span)));
        let step_stmt = Stmt::Assign {
            name: iterator,
            is_mut: false,
            value: Expr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::Ident(iterator, span)),
                rhs: Box::new(cast(step_expr)),
                span,
            },
            span,
        };
        let mut body_stmts = body.stmts;
        patch_continues(&mut body_stmts, &step_stmt);
        body_stmts.push(step_stmt);
        let while_stmt = Stmt::While {
            cond,
            body: Block::new(body_stmts, span),
            span,
        };
        Some(Stmt::Block(Block::new(vec![init, while_stmt], span)))
    }

    pub(super) fn for_in_stmt(
        &mut self,
        iterator: Symbol,
        iterable: &Expr,
        body: &Block,
        span: crate::span::Span,
    ) -> Option<Stmt> {
        let body_await = body.stmts.iter().any(stmt_has_await_local);
        if expr_has_await(iterable) {
            return None;
        }
        if !body_await {
            return Some(Stmt::ForIn {
                iterator,
                iterable: iterable.clone(),
                body: body.clone(),
                span,
            });
        }
        if body.tail.is_some() {
            return None;
        }
        let elem_ty = self
            .probe
            .and_then(|m| m.get(&iterator))
            .and_then(|t| hir_to_ast(t, span))?;
        let iter_ty = self.iter_expr_type(iterable, span)?;
        let body = self.block(body)?;
        let idx = self.for_next;
        self.for_next += 1;
        let slot = Symbol::intern(&format!("__async_it{}", idx));
        let next = Symbol::intern(&format!("__async_next{}", idx));
        let option = |t: Type| Type::Generic(Symbol::intern("Option"), vec![t], span);
        self.types.insert(slot, option(iter_ty.clone()));
        self.types.insert(next, option(elem_ty.clone()));

        let init = Stmt::Assign {
            name: slot,
            is_mut: false,
            value: Expr::EnumConstruct {
                enum_name: Symbol::intern("Option"),
                variant_name: Symbol::intern("Some"),
                tuple_args: vec![iterable.clone()],
                named_args: Vec::new(),
                span,
            },
            span,
        };
        let iter_expr = Expr::FieldAccess {
            object: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Ident(slot, span)),
                field: Symbol::intern("_data_Some"),
                span,
            }),
            field: Symbol::intern("_0"),
            span,
        };
        let assign_next = Stmt::Assign {
            name: next,
            is_mut: false,
            value: Expr::MethodCall {
                object: Box::new(iter_expr),
                method: Symbol::intern("next"),
                args: Vec::new(),
                span,
            },
            span,
        };
        let cond = Expr::Binary {
            op: BinaryOp::Eq,
            lhs: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Ident(next, span)),
                field: Symbol::intern("_tag"),
                span,
            }),
            rhs: Box::new(Expr::Literal(Literal::Int(0, span))),
            span,
        };
        let elem = Expr::FieldAccess {
            object: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Ident(next, span)),
                field: Symbol::intern("_data_Some"),
                span,
            }),
            field: Symbol::intern("_0"),
            span,
        };
        let bind = Stmt::Assign {
            name: iterator,
            is_mut: false,
            value: elem,
            span,
        };
        let mut then_stmts = vec![bind];
        then_stmts.extend(body.stmts);
        let if_stmt = Stmt::If {
            cond,
            then_block: Block::new(then_stmts, span),
            elifs: Vec::new(),
            else_block: Some(Block::new(vec![Stmt::Break { span }], span)),
            span,
        };
        let while_stmt = Stmt::While {
            cond: Expr::Literal(Literal::Bool(true, span)),
            body: Block::new(vec![assign_next, if_stmt], span),
            span,
        };
        Some(Stmt::Block(Block::new(vec![init, while_stmt], span)))
    }

    fn iter_expr_type(&self, e: &Expr, span: crate::span::Span) -> Option<Type> {
        match e {
            Expr::Ident(n, _) => self
                .types
                .get(n)
                .cloned()
                .or_else(|| self.probe?.get(n).and_then(|t| hir_to_ast(t, span))),
            Expr::StructLiteral {
                type_name,
                generic_args,
                ..
            } => {
                if generic_args.is_empty() {
                    Some(Type::Named(*type_name, span))
                } else {
                    Some(Type::Generic(*type_name, generic_args.clone(), span))
                }
            }
            _ => None,
        }
    }
}

fn patch_continues(stmts: &mut [Stmt], step: &Stmt) {
    for st in stmts {
        match st {
            Stmt::Continue { span } => {
                *st = Stmt::Block(Block::new(
                    vec![step.clone(), Stmt::Continue { span: *span }],
                    *span,
                ));
            }
            Stmt::If {
                then_block,
                elifs,
                else_block,
                ..
            } => {
                patch_continues(&mut then_block.stmts, step);
                for (_, b) in elifs {
                    patch_continues(&mut b.stmts, step);
                }
                if let Some(b) = else_block {
                    patch_continues(&mut b.stmts, step);
                }
            }
            Stmt::Block(b) => patch_continues(&mut b.stmts, step),
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let crate::parser::ast::stmt::MatchBody::Block(b) = &mut arm.body {
                        patch_continues(&mut b.stmts, step);
                    }
                }
            }
            Stmt::Unsafe { block, .. } => patch_continues(&mut block.stmts, step),
            Stmt::Attributed { stmt, .. } => {
                patch_continues(std::slice::from_mut(stmt.as_mut()), step);
            }
            // continue 属于内层循环自身。
            Stmt::While { .. } | Stmt::For { .. } | Stmt::ForIn { .. } => {}
            _ => {}
        }
    }
}
