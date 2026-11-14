//! match 内 await：同步辅助函数做模式分派，绑定写入 ref mut 状态字段。
use std::collections::{HashMap, HashSet};

use crate::formatter::format_program;
use crate::hir::async_mixed_types::hir_to_ast;
use crate::intern::Symbol;
use crate::parser::ast::stmt::{MatchArm, MatchBody};
use crate::parser::ast::vis::Visibility;
use crate::parser::ast::{Block, Expr, Literal, Pattern, Program, Stmt, Type};
use crate::span::Span;

use super::anf::{Anf, expr_has_await, stmt_has_await_local};
use super::anf_guard::{guard_ok, or_consistent, pattern_bindings};
use super::scan::has_break_continue;
use super::util::{field_name, source_ty};

struct Arm {
    pattern: Pattern,
    guard: Option<Expr>,
    body: Block,
    bindings: Vec<Symbol>,
}

impl Anf<'_> {
    pub(super) fn match_stmt(
        &mut self,
        value: &Expr,
        arms: &[MatchArm],
        span: Span,
    ) -> Option<Stmt> {
        let any_await = arms.iter().any(|a| match &a.body {
            MatchBody::Block(b) => {
                b.stmts.iter().any(stmt_has_await_local)
                    || b.tail.as_ref().is_some_and(|e| expr_has_await(e))
            }
            MatchBody::Expr(e) => expr_has_await(e),
        });
        let any_jump = arms.iter().any(|a| match &a.body {
            MatchBody::Block(b) => b.stmts.iter().any(has_break_continue),
            MatchBody::Expr(_) => false,
        });
        if !any_await && !any_jump {
            return Some(Stmt::Match {
                value: Box::new(value.clone()),
                arms: arms.to_vec(),
                span,
            });
        }
        let Expr::Ident(val_name, _) = value else {
            return None;
        };
        let Some((_, val_src)) = self.probe_type(*val_name, span) else {
            return None;
        };

        let mut out: Vec<Arm> = Vec::with_capacity(arms.len());
        let mut bind_ty: HashMap<Symbol, Type> = HashMap::new();
        for arm in arms {
            if !or_consistent(&arm.pattern) {
                return None;
            }
            let bindings = pattern_bindings(&arm.pattern);
            for b in &bindings {
                let b = *b;
                if b == *val_name {
                    return None;
                }
                if self.forced.contains(&b) && !bind_ty.contains_key(&b) {
                    return None;
                }
                let Some((ty, _)) = self.probe_type(b, span) else {
                    return None;
                };
                let src = source_ty(&ty)?;
                if let Some(old) = bind_ty.get(&b) {
                    if source_ty(old)? != src {
                        return None;
                    }
                } else {
                    bind_ty.insert(b, ty);
                }
            }
            let body = match &arm.body {
                MatchBody::Block(b) => {
                    if b.tail.is_some() {
                        return None;
                    }
                    self.block(b)?
                }
                MatchBody::Expr(e) => {
                    let mut pre = Vec::new();
                    let e = self.expr(e, &mut pre)?;
                    pre.push(Stmt::ExprStmt { expr: e, span });
                    Block::new(pre, span)
                }
            };
            if let Some(g) = &arm.guard {
                let bound: HashSet<Symbol> = bindings.iter().copied().collect();
                if expr_has_await(g) || !guard_ok(g, &bound) {
                    return None;
                }
            }
            out.push(Arm {
                pattern: arm.pattern.clone(),
                guard: arm.guard.clone(),
                body,
                bindings,
            });
        }

        let mut bindings: Vec<(Symbol, Type)> = bind_ty.into_iter().collect();
        bindings.sort_by_key(|(n, _)| n.as_str().to_string());
        for (n, t) in &bindings {
            if let Some(old) = self.types.get(n) {
                if source_ty(old)? != source_ty(t)? {
                    return None;
                }
            }
            self.types.insert(*n, t.clone());
            self.forced.insert(*n);
        }

        let idx = self.match_next;
        self.match_next += 1;
        let helper = Symbol::intern(&format!("__async_match_{}_{}", self.owner.as_str(), idx));
        let sel = Symbol::intern(&format!("__async_sel_{}_{}", self.owner.as_str(), idx));
        self.types.insert(sel, Type::Int(span));
        let val_sym = Symbol::intern(&format!("__async_val{}", idx));

        let mut out_syms: Vec<(Symbol, Symbol, String)> = Vec::new();
        for (b, t) in &bindings {
            let out = Symbol::intern(&format!("__async_out_{}", b.as_str()));
            let src = source_ty(t)?;
            out_syms.push((*b, out, src));
        }

        let mut helper_params: Vec<(Symbol, Type)> = Vec::with_capacity(out_syms.len() + 1);
        helper_params.push((val_sym, src_type(&val_src, span)));
        for (_, out, src) in &out_syms {
            helper_params.push((*out, Type::Ref(Box::new(src_type(src, span)), true, span)));
        }
        let param_count = helper_params.len();

        let mut helper_arms = Vec::with_capacity(out.len() + 1);
        for (k, arm) in out.iter().enumerate() {
            let mut stmts = Vec::new();
            for (b, out_sym, src) in &out_syms {
                if arm.bindings.contains(b) {
                    let _ = src;
                    stmts.push(Stmt::Assign {
                        name: *out_sym,
                        is_mut: false,
                        value: Expr::Ident(*b, span),
                        span,
                    });
                }
            }
            stmts.push(Stmt::Return {
                value: Some(Expr::Literal(Literal::Int((k + 1) as i64, span))),
                span,
            });
            helper_arms.push(MatchArm {
                pattern: arm.pattern.clone(),
                guard: arm.guard.clone(),
                body: MatchBody::Block(Block::new(stmts, span)),
            });
        }
        helper_arms.push(MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: MatchBody::Block(Block::new(
                vec![Stmt::Return {
                    value: Some(Expr::Literal(Literal::Int(0, span))),
                    span,
                }],
                span,
            )),
        });

        let helper_fn = Stmt::FnDecl {
            attrs: Vec::new(),
            vis: Visibility::Private,
            is_inline: false,
            extern_c: false,
            is_unsafe: false,
            name: helper,
            generic_params: Vec::new(),
            params: helper_params,
            param_attrs: vec![Vec::new(); param_count],
            return_type: Type::Int(span),
            body: Block::new(
                vec![
                    Stmt::Match {
                        value: Box::new(Expr::Ident(val_sym, span)),
                        arms: helper_arms,
                        span,
                    },
                    Stmt::Return {
                        value: Some(Expr::Literal(Literal::Int(0, span))),
                        span,
                    },
                ],
                span,
            ),
            span,
        };
        let helper_src = format_program(&Program::new(vec![helper_fn]));
        self.helpers.push(helper_src);

        let mut call_args = Vec::with_capacity(out_syms.len() + 1);
        call_args.push(Expr::Ident(*val_name, span));
        for (b, _, _) in &out_syms {
            call_args.push(Expr::Ref(
                Box::new(Expr::FieldAccess {
                    object: Box::new(Expr::Ident(Symbol::intern("self"), span)),
                    field: field_name(*b),
                    span,
                }),
                true,
                span,
            ));
        }
        let call = Expr::FnCall {
            name: helper,
            args: call_args,
            generic_args: Vec::new(),
            span,
        };
        let sel_assign = Stmt::Assign {
            name: sel,
            is_mut: false,
            value: call,
            span,
        };

        let mut chain = Block::new(
            vec![Stmt::Return {
                value: Some(Expr::FnCall {
                    name: Symbol::intern("Poll.Pending"),
                    args: Vec::new(),
                    generic_args: Vec::new(),
                    span,
                }),
                span,
            }],
            span,
        );
        for (k, arm) in out.iter().enumerate().rev() {
            let cond = Expr::Binary {
                op: crate::parser::ast::BinaryOp::Eq,
                lhs: Box::new(Expr::Ident(sel, span)),
                rhs: Box::new(Expr::Literal(Literal::Int((k + 1) as i64, span))),
                span,
            };
            let if_stmt = Stmt::If {
                cond,
                then_block: arm.body.clone(),
                elifs: Vec::new(),
                else_block: Some(chain),
                span,
            };
            chain = Block::new(vec![if_stmt], span);
        }
        Some(Stmt::Block(Block::new(
            vec![sel_assign, Stmt::Block(chain)],
            span,
        )))
    }

    fn probe_type(&self, n: Symbol, span: Span) -> Option<(Type, String)> {
        let hir = self.probe?.get(&n)?;
        let ty = hir_to_ast(hir, span)?;
        let src = source_ty(&ty)?;
        Some((ty, src))
    }
}

fn src_type(src: &str, span: Span) -> Type {
    Type::Named(Symbol::intern(src), span)
}
