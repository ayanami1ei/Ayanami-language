//! A5e：`#[sync(obj)] { ... }` 同步块降级（lock / 块体 / unlock，出口自动清理）。
use super::*;
use crate::hir::lower::ctx::SyncLock;
use crate::parser::ast::{Attr, AttrArg, Block, Expr, Stmt};

impl Ctx {
    /// `#[sync(obj)] { ... }`：进入前 lock、正常出口 unlock；提前出口由
    /// `Return` / `Break` / `Continue` 降级补解锁。
    pub(crate) fn lower_sync_block(&mut self, attr: &Attr, block: &Block) -> Result<HirStmt> {
        let Some(e) = attr.args.iter().find_map(|a| match a {
            AttrArg::Expr(e) => Some(e),
            _ => None,
        }) else {
            return Err(Error::Hir(format!(
                "#[sync(...)] expects an object expression (at {}:{})",
                attr.span.start_line, attr.span.start_col
            )));
        };
        // `guard = d` → 锁内 `d = obj.data()`（载荷引用绑定）
        let mut guard: Option<(Symbol, Span)> = None;
        for a in &attr.args {
            if let AttrArg::KeyValue(k, v) = a {
                if k.as_str() != "guard" {
                    return Err(Error::Hir(format!(
                        "#[sync(...)] unknown option `{}` (at {}:{})",
                        k.as_str(), attr.span.start_line, attr.span.start_col
                    )));
                }
                let AttrArg::Expr(inner) = v.as_ref() else {
                    return Err(Error::Hir(format!(
                        "#[sync(..., guard = <name>)] expects an identifier (at {}:{})",
                        attr.span.start_line, attr.span.start_col
                    )));
                };
                let Expr::Ident(gname, gspan) = inner.as_ref() else {
                    return Err(Error::Hir(format!(
                        "#[sync(..., guard = <name>)] expects an identifier (at {}:{})",
                        attr.span.start_line, attr.span.start_col
                    )));
                };
                guard = Some((*gname, *gspan));
            }
        }
        if !is_place_expr(e) {
            return Err(Error::Hir(format!(
                "#[sync(...)] object must be an identifier or field access (at {}:{})",
                attr.span.start_line, attr.span.start_col
            )));
        }
        let obj = (**e).clone();
        let lock = self.sync_call_stmt(&obj, "lock", attr.span)?;
        // guard 必须在体降级前降级，局部绑定才能被块内引用
        let guard_stmt = match guard {
            Some((gname, gspan)) => {
                let data_call = Expr::MethodCall {
                    object: Box::new(obj.clone()),
                    method: Symbol::intern("data"),
                    args: Vec::new(),
                    span: gspan,
                };
                let assign = Stmt::Assign { name: gname, is_mut: false, value: data_call, span: gspan };
                Some(self.lower_stmt(&assign)?)
            }
            None => None,
        };
        self.sync_locks.push(SyncLock { obj: obj.clone() });
        let body = self.lower_block(block);
        self.sync_locks.pop();
        let body = body?;
        let unlock = self.sync_call_stmt(&obj, "unlock", block.span)?;
        let mut stmts = Vec::with_capacity(body.stmts.len() + 3);
        stmts.push(lock);
        if let Some(g) = guard_stmt { stmts.push(g); }
        stmts.extend(body.stmts);
        stmts.push(unlock);
        Ok(HirStmt::Block { stmts, span: block.span })
    }

    /// Attributed 语句入口：`#[sync]` → 同步块（非块语句报错）；无 sync 返回 None。
    pub(crate) fn try_lower_sync(&mut self, attrs: &[Attr], inner: &Stmt) -> Option<Result<HirStmt>> {
        let attr = attrs.iter().find(|a| a.qualifier.is_empty() && a.name.as_str() == "sync")?;
        if let Stmt::Block(block) = inner {
            return Some(self.lower_sync_block(attr, block));
        }
        Some(Err(Error::Hir(format!(
            "#[sync] is only allowed on block statements (at {}:{})",
            attr.span.start_line, attr.span.start_col
        ))))
    }

    /// `return` 出口：返回值落临时局部 → 逆序 unlock → 返回（返回值在锁内求值）。
    pub(crate) fn sync_return_stmt(&mut self, hir_value: Option<HirNodeBox>, span: Span) -> Result<HirStmt> {
        let mut stmts: Vec<HirStmt> = Vec::new();
        let value = match hir_value {
            Some(v) => {
                let ty = expr_type(&v);
                let name = Symbol::intern(&format!("__sync_ret_{}", self.locals.len()));
                let (id, ty, _) = self.register_or_lookup(name, ty);
                stmts.push(HirStmt::Assign {
                    target: SVar { var: id, ty: ty.clone() }.into(),
                    value: v,
                    span,
                });
                Some(SVar { var: id, ty }.into())
            }
            None => None,
        };
        let unlocks = self.sync_unlock_stmts(0, span)?;
        stmts.extend(unlocks);
        stmts.push(HirStmt::Return { value, span });
        Ok(HirStmt::Block { stmts, span })
    }

    /// 提前出口的 unlock：逆序解锁 `sync_locks[from..]`。
    pub(crate) fn sync_unlock_stmts(&mut self, from: usize, span: Span) -> Result<Vec<HirStmt>> {
        let locks: Vec<SyncLock> = self.sync_locks[from..].to_vec();
        let mut out = Vec::with_capacity(locks.len());
        for l in locks.iter().rev() {
            out.push(self.sync_call_stmt(&l.obj, "unlock", span)?);
        }
        Ok(out)
    }

    /// `break`/`continue`：先解锁当前循环体内获取的锁（循环外持有的保持）。
    pub(crate) fn lower_loop_exit(&mut self, exit: HirStmt, span: Span) -> Result<HirStmt> {
        let Some(&mark) = self.loop_sync_marks.last() else { return Ok(exit) };
        if self.sync_locks.len() <= mark {
            return Ok(exit);
        }
        let mut stmts = self.sync_unlock_stmts(mark, span)?;
        stmts.push(exit);
        Ok(HirStmt::Block { stmts, span })
    }

    /// 循环体降级期间登记锁标记（`break`/`continue` 只解到该位置）。
    pub(crate) fn lower_loop_body(&mut self, body: &Block) -> Result<HirBlock> {
        self.loop_sync_marks.push(self.sync_locks.len());
        let r = self.lower_block(body);
        self.loop_sync_marks.pop();
        r
    }

    fn sync_call_stmt(&mut self, obj: &Expr, method: &str, span: Span) -> Result<HirStmt> {
        let call = Expr::MethodCall {
            object: Box::new(obj.clone()),
            method: Symbol::intern(method),
            args: Vec::new(),
            span,
        };
        let node = self.lower_expr(&call)?;
        Ok(HirStmt::Expr { expr: node, span })
    }
}

/// `#[sync]` 对象只允许位置表达式（标识符 / 字段访问），避免重复求值副作用。
fn is_place_expr(e: &Expr) -> bool {
    match e {
        Expr::Ident(..) => true,
        Expr::FieldAccess { object, .. } => is_place_expr(object),
        _ => false,
    }
}
