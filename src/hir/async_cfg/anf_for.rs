//! 数值 for 体内 await：降为带迭代变量类型 cast 的 while。
use crate::hir::async_mixed_types::hir_to_ast;
use crate::parser::ast::{BinaryOp, Block, Expr, Literal, Stmt};

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
