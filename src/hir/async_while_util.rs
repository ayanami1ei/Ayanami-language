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
