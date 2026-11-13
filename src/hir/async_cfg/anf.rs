//! 表达式级 await 的 ANF 提升：把 await 子表达式拆成临时赋值语句。
//!
//! v1 支持赋值/返回/实参/字段或索引写值中的 await；拒绝 while 条件、
//! match 分支/守卫、for 范围与表达式级 if/match/lambda 内的 await。
use std::collections::HashMap;

use crate::intern::Symbol;
use crate::parser::ast::{Block, Expr, Literal, Stmt, Type};

pub(super) struct Anf<'a> {
    pub(super) returns: &'a HashMap<Symbol, Type>,
    pub(super) types: HashMap<Symbol, Type>,
    pub(super) next: usize,
}

impl<'a> Anf<'a> {
    pub(super) fn new(returns: &'a HashMap<Symbol, Type>) -> Self {
        Self {
            returns,
            types: HashMap::new(),
            next: 0,
        }
    }

    pub(super) fn into_types(self) -> HashMap<Symbol, Type> {
        self.types
    }

    /// 归一化整个函数体；返回 (语句, 尾表达式)。
    pub(super) fn normalize_body(&mut self, body: &Block) -> Option<(Vec<Stmt>, Option<Expr>)> {
        let mut out = Vec::new();
        for st in &body.stmts {
            let mut pre = Vec::new();
            let new = self.stmt(st, &mut pre)?;
            out.extend(pre);
            out.push(new);
        }
        let tail = if let Some(t) = &body.tail {
            let mut pre = Vec::new();
            let e = self.expr(t, &mut pre)?;
            out.extend(pre);
            Some(e)
        } else {
            None
        };
        Some((out, tail))
    }

    fn block(&mut self, b: &Block) -> Option<Block> {
        if b.tail.as_ref().is_some_and(|e| expr_has_await(e)) {
            return None;
        }
        let mut out = Vec::new();
        for st in &b.stmts {
            let mut pre = Vec::new();
            let new = self.stmt(st, &mut pre)?;
            out.extend(pre);
            out.push(new);
        }
        Some(Block {
            stmts: out,
            tail: b.tail.clone(),
            span: b.span,
        })
    }

    fn stmt(&mut self, st: &Stmt, pre: &mut Vec<Stmt>) -> Option<Stmt> {
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
                start,
                end,
                step,
                body,
                span,
                iterator,
            } => {
                if expr_has_await(start)
                    || expr_has_await(end)
                    || step.as_ref().is_some_and(expr_has_await)
                    || body.stmts.iter().any(stmt_has_await_local)
                {
                    return None;
                }
                Stmt::For {
                    iterator: *iterator,
                    start: start.clone(),
                    end: end.clone(),
                    step: step.clone(),
                    body: body.clone(),
                    span: *span,
                }
            }
            Stmt::ForIn {
                iterator,
                iterable,
                body,
                span,
            } => {
                if expr_has_await(iterable) || body.stmts.iter().any(stmt_has_await_local) {
                    return None;
                }
                Stmt::ForIn {
                    iterator: *iterator,
                    iterable: iterable.clone(),
                    body: body.clone(),
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

    fn else_chain(
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

fn stmt_has_await_local(st: &Stmt) -> bool {
    crate::hir::async_cfg::stmt_has_await(st)
}

fn expr_has_await(e: &Expr) -> bool {
    crate::hir::async_cfg::expr_has_await_pub(e)
}
