//! ANF 语句重写：if/while/for 与 await 子表达式提升。
use crate::hir::async_mixed_types::hir_to_ast;
use crate::intern::Symbol;
use crate::parser::ast::{BinaryOp, Block, Expr, Literal, Stmt, Type};

use super::anf::{Anf, expr_has_await, stmt_has_await_local};

impl Anf<'_> {
    pub(super) fn stmt(&mut self, st: &Stmt, pre: &mut Vec<Stmt>) -> Option<Stmt> {
        Some(match st {
            Stmt::Assign {
                name,
                is_mut,
                value,
                span,
            } => Stmt::Assign {
                name: *name,
                is_mut: *is_mut,
                value: self.expr(value, pre)?,
                span: *span,
            },
            Stmt::ExprStmt { expr, span } => Stmt::ExprStmt {
                expr: self.expr(expr, pre)?,
                span: *span,
            },
            Stmt::Return { value, span } => Stmt::Return {
                value: match value {
                    Some(e) => Some(self.expr(e, pre)?),
                    None => None,
                },
                span: *span,
            },
            Stmt::FieldAssign {
                object,
                field,
                value,
                span,
            } => Stmt::FieldAssign {
                object: Box::new(self.expr(object, pre)?),
                field: *field,
                value: self.expr(value, pre)?,
                span: *span,
            },
            Stmt::IndexAssign {
                object,
                index,
                value,
                span,
            } => Stmt::IndexAssign {
                object: Box::new(self.expr(object, pre)?),
                index: Box::new(self.expr(index, pre)?),
                value: self.expr(value, pre)?,
                span: *span,
            },
            Stmt::DerefAssign {
                target,
                value,
                span,
            } => Stmt::DerefAssign {
                target: Box::new(self.expr(target, pre)?),
                value: Box::new(self.expr(value, pre)?),
                span: *span,
            },
            Stmt::If {
                cond,
                then_block,
                elifs,
                else_block,
                span,
            } => {
                let cond = self.expr(cond, pre)?;
                let then_block = self.block(then_block)?;
                let else_block = self.else_chain(elifs, else_block.as_ref(), *span)?;
                Stmt::If {
                    cond,
                    then_block,
                    elifs: Vec::new(),
                    else_block,
                    span: *span,
                }
            }
            Stmt::While { cond, body, span } => {
                // 条件每轮求值：把 cond 的 await 提升留在循环体内，
                // 变换为 `while true { <cond_pre>; if cond { body } else { break } }`。
                let mut cond_pre = Vec::new();
                let cond = self.expr(cond, &mut cond_pre)?;
                let body = self.block(body)?;
                if cond_pre.is_empty() {
                    Stmt::While {
                        cond,
                        body,
                        span: *span,
                    }
                } else {
                    let else_block = Block::new(vec![Stmt::Break { span: *span }], *span);
                    let if_stmt = Stmt::If {
                        cond,
                        then_block: body,
                        elifs: Vec::new(),
                        else_block: Some(else_block),
                        span: *span,
                    };
                    cond_pre.push(if_stmt);
                    Stmt::While {
                        cond: Expr::Literal(Literal::Bool(true, *span)),
                        body: Block::new(cond_pre, *span),
                        span: *span,
                    }
                }
            }
            Stmt::Block(b) => Stmt::Block(self.block(b)?),
            Stmt::Match { value, arms, .. } => {
                if expr_has_await(value)
                    || arms.iter().any(|a| {
                        a.guard.as_ref().is_some_and(expr_has_await)
                            || match &a.body {
                                crate::parser::ast::stmt::MatchBody::Expr(e) => expr_has_await(e),
                                crate::parser::ast::stmt::MatchBody::Block(b) => {
                                    b.stmts.iter().any(stmt_has_await_local)
                                        || b.tail.as_ref().is_some_and(|e| expr_has_await(e))
                                }
                            }
                    })
                {
                    return None;
                }
                st.clone()
            }
            Stmt::For {
                iterator,
                start,
                end,
                step,
                body,
                span,
            } => self.for_stmt(*iterator, start, end, step.as_ref(), body, *span)?,
            Stmt::ForIn {
                iterator,
                iterable,
                body,
                span,
            } => {
                if expr_has_await(iterable) {
                    return None;
                }
                let body_await = body.stmts.iter().any(stmt_has_await_local);
                if !body_await {
                    return Some(Stmt::ForIn {
                        iterator: *iterator,
                        iterable: iterable.clone(),
                        body: body.clone(),
                        span: *span,
                    });
                }
                // 只支持 `for v in ident { ... }`；表达式 iterable 需要额外的迭代器字段类型。
                let Expr::Ident(it, it_span) = iterable else {
                    return None;
                };
                if body.tail.is_some() {
                    return None;
                }
                let elem_ty = self
                    .probe
                    .and_then(|m| m.get(iterator))
                    .and_then(|t| hir_to_ast(t, *span))?;
                let body = self.block(body)?;
                let next = Symbol::intern(&format!("__async_next{}", self.for_next));
                self.for_next += 1;
                self.types.insert(
                    next,
                    Type::Generic(Symbol::intern("Option"), vec![elem_ty], *span),
                );
                let assign_next = Stmt::Assign {
                    name: next,
                    is_mut: false,
                    value: Expr::MethodCall {
                        object: Box::new(Expr::Ident(*it, *it_span)),
                        method: Symbol::intern("next"),
                        args: Vec::new(),
                        span: *span,
                    },
                    span: *span,
                };
                let cond = Expr::Binary {
                    op: BinaryOp::Eq,
                    lhs: Box::new(Expr::FieldAccess {
                        object: Box::new(Expr::Ident(next, *span)),
                        field: Symbol::intern("_tag"),
                        span: *span,
                    }),
                    rhs: Box::new(Expr::Literal(Literal::Int(0, *span))),
                    span: *span,
                };
                let elem = Expr::FieldAccess {
                    object: Box::new(Expr::FieldAccess {
                        object: Box::new(Expr::Ident(next, *span)),
                        field: Symbol::intern("_data_Some"),
                        span: *span,
                    }),
                    field: Symbol::intern("_0"),
                    span: *span,
                };
                let bind = Stmt::Assign {
                    name: *iterator,
                    is_mut: false,
                    value: elem,
                    span: *span,
                };
                let mut then_stmts = vec![bind];
                then_stmts.extend(body.stmts);
                let if_stmt = Stmt::If {
                    cond,
                    then_block: Block::new(then_stmts, *span),
                    elifs: Vec::new(),
                    else_block: Some(Block::new(vec![Stmt::Break { span: *span }], *span)),
                    span: *span,
                };
                Stmt::While {
                    cond: Expr::Literal(Literal::Bool(true, *span)),
                    body: Block::new(vec![assign_next, if_stmt], *span),
                    span: *span,
                }
            }
            Stmt::Unsafe { block, span } => {
                if block.stmts.iter().any(stmt_has_await_local) {
                    return None;
                }
                Stmt::Unsafe {
                    block: block.clone(),
                    span: *span,
                }
            }
            Stmt::Attributed { attrs, stmt, span } => {
                if stmt_has_await_local(stmt) {
                    return None;
                }
                Stmt::Attributed {
                    attrs: attrs.clone(),
                    stmt: stmt.clone(),
                    span: *span,
                }
            }
            _ => st.clone(),
        })
    }

    pub(super) fn else_chain(
        &mut self,
        elifs: &[(Expr, Block)],
        else_block: Option<&Block>,
        span: crate::span::Span,
    ) -> Option<Option<Block>> {
        if elifs.is_empty() {
            return Some(match else_block {
                Some(b) => Some(self.block(b)?),
                None => None,
            });
        }
        let (cond, then_block) = &elifs[0];
        let mut pre = Vec::new();
        let cond = self.expr(cond, &mut pre)?;
        let then_block = self.block(then_block)?;
        let rest = self.else_chain(&elifs[1..], else_block, span)?;
        let nested = Stmt::If {
            cond,
            then_block,
            elifs: Vec::new(),
            else_block: rest,
            span,
        };
        pre.push(nested);
        Some(Some(Block::new(pre, span)))
    }
}
