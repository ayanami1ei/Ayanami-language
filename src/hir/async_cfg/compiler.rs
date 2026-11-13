//! 通用 CFG 状态块生成：倒序编译 + 显式 `_state` 跳转。
use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};
use crate::formatter::{format_expr, format_stmt};
use crate::intern::Symbol;
use crate::parser::ast::{Block, Expr, Stmt, Type};

use super::{FutureField, Local, await_parts};

pub(super) struct Cfg {
    pub(super) states: Vec<Option<String>>,
    pub(super) locals: Vec<Local>,
    pub(super) futures: Vec<FutureField>,
    pub(super) async_fns: HashMap<Symbol, Vec<(Symbol, Type)>>,
    pub(super) promoted: HashSet<Symbol>,
    pub(super) unsupported: bool,
}

impl Cfg {
    pub(super) fn new(locals: Vec<Local>, async_fns: HashMap<Symbol, Vec<(Symbol, Type)>>) -> Self {
        let promoted = locals
            .iter()
            .filter(|l| !l.is_copy)
            .map(|l| l.name)
            .collect();
        Self {
            states: Vec::new(),
            locals,
            futures: Vec::new(),
            async_fns,
            promoted,
            unsupported: false,
        }
    }

    pub(super) fn states(&self) -> &[Option<String>] {
        &self.states
    }

    pub(super) fn locals(&self) -> &[Local] {
        &self.locals
    }

    pub(super) fn futures(&self) -> &[FutureField] {
        &self.futures
    }

    pub(super) fn unsupported(&self) -> bool {
        self.unsupported
    }

    pub(super) fn compile(&mut self, stmts: &[Stmt]) -> Result<usize> {
        let end = self.alloc();
        self.fill(end, "return Poll::Pending()\n".to_string());
        self.compile_seq(stmts, end, &[])
    }

    pub(super) fn alloc(&mut self) -> usize {
        self.states.push(None);
        self.states.len() - 1
    }

    pub(super) fn fill(&mut self, id: usize, code: String) {
        self.states[id] = Some(code);
    }

    pub(super) fn loads(&self) -> String {
        let mut s = String::new();
        for l in self.locals.iter().filter(|l| l.is_copy) {
            s.push_str(&format!(
                "{} = self.{};\n",
                l.name.as_str(),
                l.field.as_str()
            ));
        }
        s
    }

    pub(super) fn stores(&self) -> String {
        let mut s = String::new();
        for l in self.locals.iter().filter(|l| l.is_copy) {
            s.push_str(&format!(
                "self.{} = {};\n",
                l.field.as_str(),
                l.name.as_str()
            ));
        }
        s
    }

    pub(super) fn render_expr(&mut self, e: &Expr) -> String {
        if self.promoted.is_empty() {
            return format_expr(e);
        }
        match crate::hir::async_mixed_expr::render_expr(e, &self.promoted) {
            Some(s) => s,
            None => {
                self.unsupported = true;
                "0".into()
            }
        }
    }

    fn render_stmt(&mut self, st: &Stmt) -> String {
        if self.promoted.is_empty() {
            return format_stmt(st);
        }
        match crate::hir::async_mixed_expr::render_sync(st, &self.promoted) {
            Some(s) => s,
            None => {
                self.unsupported = true;
                String::new()
            }
        }
    }

    fn compile_seq(
        &mut self,
        stmts: &[Stmt],
        next: usize,
        loops: &[(usize, usize)],
    ) -> Result<usize> {
        let mut next = next;
        let mut run: Vec<&Stmt> = Vec::new();
        for st in stmts.iter().rev() {
            if simple(st) {
                run.push(st);
            } else {
                if !run.is_empty() {
                    next = self.emit_run(&run, next);
                    run.clear();
                }
                next = self.compile_stmt(st, next, loops)?;
            }
        }
        if !run.is_empty() {
            next = self.emit_run(&run, next);
        }
        Ok(next)
    }

    fn emit_run(&mut self, run: &[&Stmt], next: usize) -> usize {
        let id = self.alloc();
        let mut code = self.loads();
        for st in run.iter().rev() {
            code.push_str(&self.render_stmt(st));
            code.push('\n');
        }
        code.push_str(&self.stores());
        code.push_str(&format!("self._state = {}; continue\n", next));
        self.fill(id, code);
        id
    }

    fn compile_stmt(&mut self, st: &Stmt, next: usize, loops: &[(usize, usize)]) -> Result<usize> {
        match st {
            Stmt::Assign { name, value, .. } => {
                let (g, args) = await_parts(value).ok_or_else(|| {
                    Error::Hir("internal async CFG: assignment without await".into())
                })?;
                self.compile_await(*name, g, &args, next)
            }
            Stmt::If {
                cond,
                then_block,
                elifs,
                else_block,
                ..
            } => {
                let mut arms: Vec<(Expr, Block)> = Vec::with_capacity(elifs.len() + 1);
                arms.push((cond.clone(), then_block.clone()));
                for (c, b) in elifs {
                    arms.push((c.clone(), b.clone()));
                }
                self.compile_arms(&arms, else_block.as_ref(), next, loops)
            }
            Stmt::Block(b) => {
                if b.tail.is_some() {
                    return Err(Error::Hir(
                        "async CFG: block tail is not supported (v1)".into(),
                    ));
                }
                self.compile_seq(&b.stmts, next, loops)
            }
            Stmt::While { cond, body, .. } => self.compile_while(cond, body, next, loops),
            Stmt::Break { .. } => {
                let (_, exit) = loops
                    .last()
                    .ok_or_else(|| Error::Hir("async CFG: `break` outside loop".into()))?;
                Ok(self.jump(*exit))
            }
            Stmt::Continue { .. } => {
                let (head, _) = loops
                    .last()
                    .ok_or_else(|| Error::Hir("async CFG: `continue` outside loop".into()))?;
                Ok(self.jump(*head))
            }
            Stmt::Return { value: Some(e), .. } => {
                let rendered = self.render_expr(e);
                let id = self.alloc();
                let code = format!("{}return Poll::Ready({});\n", self.loads(), rendered);
                self.fill(id, code);
                Ok(id)
            }
            Stmt::Return { value: None, .. } => Err(Error::Hir(
                "async CFG: `return` without a value is not supported (v1)".into(),
            )),
            _ => Err(Error::Hir(
                "internal async CFG: unsupported statement".into(),
            )),
        }
    }

    fn jump(&mut self, target: usize) -> usize {
        let id = self.alloc();
        self.fill(id, format!("self._state = {}; continue\n", target));
        id
    }

    fn compile_arms(
        &mut self,
        arms: &[(Expr, Block)],
        else_block: Option<&Block>,
        next: usize,
        loops: &[(usize, usize)],
    ) -> Result<usize> {
        if arms.is_empty() {
            return match else_block {
                Some(b) => self.compile_seq(&b.stmts, next, loops),
                None => Ok(next),
            };
        }
        let (cond, then_block) = &arms[0];
        let then_entry = self.compile_seq(&then_block.stmts, next, loops)?;
        let else_entry = self.compile_arms(&arms[1..], else_block, next, loops)?;
        let rendered = self.render_expr(cond);
        let id = self.alloc();
        let mut code = self.loads();
        code.push_str(&format!(
            "if {} {{ self._state = {} }} else {{ self._state = {} }}\n",
            rendered, then_entry, else_entry
        ));
        code.push_str(&self.stores());
        code.push_str("continue\n");
        self.fill(id, code);
        Ok(id)
    }

    fn compile_while(
        &mut self,
        cond: &Expr,
        body: &Block,
        next: usize,
        loops: &[(usize, usize)],
    ) -> Result<usize> {
        let head = self.alloc();
        let exit = next;
        let mut inner = loops.to_vec();
        inner.push((head, exit));
        let body_entry = self.compile_seq(&body.stmts, head, &inner)?;
        let rendered = self.render_expr(cond);
        let mut code = self.loads();
        code.push_str(&format!(
            "if {} {{ self._state = {} }} else {{ self._state = {} }}\n",
            rendered, body_entry, exit
        ));
        code.push_str(&self.stores());
        code.push_str("continue\n");
        self.fill(head, code);
        Ok(head)
    }
}

fn simple(st: &Stmt) -> bool {
    match st {
        Stmt::Assign { value, .. } => await_parts(value).is_none(),
        Stmt::ExprStmt { .. }
        | Stmt::FieldAssign { .. }
        | Stmt::IndexAssign { .. }
        | Stmt::DerefAssign { .. } => true,
        Stmt::While { .. } => !super::stmt_has_await(st),
        Stmt::For { .. }
        | Stmt::ForIn { .. }
        | Stmt::Match { .. }
        | Stmt::Unsafe { .. }
        | Stmt::Attributed { .. } => true,
        _ => false,
    }
}
