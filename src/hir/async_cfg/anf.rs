//! 表达式级 await 的 ANF 提升：把 await 子表达式拆成临时赋值语句。
//!
//! v1 支持赋值/返回/实参/字段或索引写值中的 await；拒绝 while 条件、
//! match 分支/守卫、for 范围与表达式级 if/match/lambda 内的 await。
use std::collections::HashMap;

use crate::intern::Symbol;
use crate::parser::ast::{Block, Expr, Stmt, Type};

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
                let mut new_elifs = Vec::with_capacity(elifs.len());
                for (c, b) in elifs {
                    // elif 条件只在前面分支不成立时求值，不能提前到 If 前。
                    if expr_has_await(c) {
                        return None;
                    }
                    new_elifs.push((c.clone(), self.block(b)?));
                }
                let else_block = match else_block {
                    Some(b) => Some(self.block(b)?),
                    None => None,
                };
                Stmt::If {
                    cond,
                    then_block,
                    elifs: new_elifs,
                    else_block,
                    span: *span,
                }
            }
            Stmt::While { cond, body, span } => {
                // while 条件每轮求值；提前提升会改变语义。
                if expr_has_await(cond) {
                    return None;
                }
                Stmt::While {
                    cond: cond.clone(),
                    body: self.block(body)?,
                    span: *span,
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
}

fn stmt_has_await_local(st: &Stmt) -> bool {
    crate::hir::async_cfg::stmt_has_await(st)
}

fn expr_has_await(e: &Expr) -> bool {
    crate::hir::async_cfg::expr_has_await_pub(e)
}
