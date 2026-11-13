//! 表达式级 await 的 ANF 提升：把 await 子表达式拆成临时赋值语句。
//!
//! v1 支持赋值/返回/实参/字段或索引写值/if/elif/while 条件与数值 for 体内的 await；
//! 拒绝 match、for-in、unsafe 与表达式级 if/match/lambda 内的 await。
use std::collections::HashMap;

use crate::intern::Symbol;
use crate::parser::ast::{Block, Expr, Stmt, Type};

pub(super) struct Anf<'a> {
    pub(super) returns: &'a HashMap<Symbol, Type>,
    pub(super) probe: Option<&'a HashMap<Symbol, crate::hir::ir::HirType>>,
    pub(super) types: HashMap<Symbol, Type>,
    pub(super) next: usize,
}

impl<'a> Anf<'a> {
    pub(super) fn new(
        returns: &'a HashMap<Symbol, Type>,
        probe: Option<&'a HashMap<Symbol, crate::hir::ir::HirType>>,
    ) -> Self {
        Self {
            returns,
            probe,
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

    pub(super) fn block(&mut self, b: &Block) -> Option<Block> {
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
}

pub(super) fn stmt_has_await_local(st: &Stmt) -> bool {
    crate::hir::async_cfg::stmt_has_await(st)
}

pub(super) fn expr_has_await(e: &Expr) -> bool {
    crate::hir::async_cfg::expr_has_await_pub(e)
}
