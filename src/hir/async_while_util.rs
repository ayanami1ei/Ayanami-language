//! async while 状态机辅助：break/continue 控制条件 helper。
use crate::intern::Symbol;
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Literal, Stmt, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Ctrl {
    Break,
    Continue,
}

/// 控制条件 helper：参数 + y → int（1=break，2=continue，0=继续执行）。
pub(super) fn ctrl_helper(
    stmt: &Stmt,
    name: &str,
    y: Symbol,
    y_ty: &Type,
    controls: &[(Expr, Ctrl)],
    span: crate::span::Span,
) -> Stmt {
    let mut h = stmt.clone();
    if let Stmt::FnDecl { attrs, name: hn, params, return_type, body, vis, .. } = &mut h {
        attrs.clear();
        *hn = Symbol::intern(name);
        *vis = Visibility::Private;
        params.push((y, y_ty.clone()));
        *return_type = Type::Int(span);
        let mut stmts = Vec::new();
        for (cond, ctrl) in controls {
            let code = match ctrl {
                Ctrl::Break => 1,
                Ctrl::Continue => 2,
            };
            stmts.push(Stmt::If {
                cond: cond.clone(),
                then_block: Block::new(
                    vec![Stmt::Return {
                        value: Some(Expr::Literal(Literal::Int(code, span))),
                        span,
                    }],
                    span,
                ),
                elifs: Vec::new(),
                else_block: None,
                span,
            });
        }
        stmts.push(Stmt::Return {
            value: Some(Expr::Literal(Literal::Int(0, span))),
            span,
        });
        body.stmts = stmts;
        body.tail = None;
    }
    h
}

pub(super) struct Shape {
    pub(super) y: Symbol,
    /// 连续的 (async 函数, 实参) 序列，全部赋值给 `y`。
    pub(super) awaits: Vec<(Symbol, Vec<Expr>)>,
    pub(super) cond: Expr,
    pub(super) pre: Vec<(Expr, Ctrl)>,
    pub(super) post: Vec<(Expr, Ctrl)>,
}


/// 匹配 `y = <默认>; while cond { [if c { break/continue }] (y = await g(args))+ [if c { break/continue }] }; ...`。
pub(super) fn detect(body: &Block) -> Option<Shape> {
    let [Stmt::Assign { name: pre, .. }, Stmt::While { cond, body: wbody, .. }, ..] =
        body.stmts.as_slice() else { return None };
    let mut phase = 0u8; // 0=await 前，1=await 序列，2=await 后
    let mut start = None;
    let mut end = 0;
    let mut awaits = Vec::new();
    for (i, st) in wbody.stmts.iter().enumerate() {
        if let Some((name, g, gargs)) = awaited_assignment(st) {
            if phase == 2 { return None; }
            if start.is_none() {
                if pre != &name { return None; }
                start = Some(i);
            }
            if name != *pre { return None; }
            awaits.push((g, gargs));
            end = i + 1;
            phase = 1;
        } else if phase == 1 {
            phase = 2;
        }
    }
    let start = start?;
    let mut pre_ctrl = Vec::new();
    for st in &wbody.stmts[..start] {
        pre_ctrl.push(ctrl_of(st)?);
    }
    let mut post_ctrl = Vec::new();
    for st in &wbody.stmts[end..] {
        post_ctrl.push(ctrl_of(st)?);
    }
    Some(Shape {
        y: *pre,
        awaits,
        cond: cond.clone(),
        pre: pre_ctrl,
        post: post_ctrl,
    })
}

pub(super) fn awaited_assignment(st: &Stmt) -> Option<(Symbol, Symbol, Vec<Expr>)> {
    let Stmt::Assign { name, value, .. } = st else { return None };
    let Expr::FnCall { name: an, args, .. } = value else { return None };
    if an.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    Some((*name, *g, gargs.clone()))
}

fn ctrl_of(st: &Stmt) -> Option<(Expr, Ctrl)> {
    let Stmt::If { cond, then_block, elifs, else_block: None, .. } = st else { return None };
    if !elifs.is_empty() { return None; }
    match then_block.stmts.as_slice() {
        [Stmt::Break { .. }] => Some((cond.clone(), Ctrl::Break)),
        [Stmt::Continue { .. }] => Some((cond.clone(), Ctrl::Continue)),
        _ => None,
    }
}

/// 表达式是否为 `__await(g(args))`；返回 (g, args)。
pub(super) fn await_parts(value: &Expr) -> Option<(Symbol, &[Expr])> {
    let Expr::FnCall { name, args, .. } = value else { return None };
    if name.as_str() != "__await" { return None; }
    let Expr::FnCall { name: g, args: gargs, .. } = args.first()? else { return None };
    Some((*g, gargs))
}
