//! ANF 表达式重写：await 子表达式 → 临时赋值。
use crate::hir::async_mixed_types::type_default;
use crate::intern::Symbol;
use crate::parser::ast::{Expr, Stmt};

use super::anf::Anf;

impl Anf<'_> {
    pub(super) fn expr(&mut self, e: &Expr, pre: &mut Vec<Stmt>) -> Option<Expr> {
        match e {
            Expr::FnCall {
                name,
                args,
                generic_args,
                span,
            } => {
                if name.as_str() == "__await" {
                    if args.len() != 1 {
                        return None;
                    }
                    let inner = self.expr(&args[0], pre)?;
                    let Expr::FnCall {
                        name: g,
                        args: gargs,
                        generic_args: ggenerics,
                        span: gspan,
                    } = inner
                    else {
                        return None;
                    };
                    let ret = self.returns.get(&g)?.clone();
                    if type_default(&ret).is_none() {
                        return None;
                    }
                    let tmp = Symbol::intern(&format!("__async_t{}", self.next));
                    self.next += 1;
                    self.types.insert(tmp, ret);
                    pre.push(Stmt::Assign {
                        name: tmp,
                        is_mut: false,
                        value: Expr::FnCall {
                            name: Symbol::intern("__await"),
                            args: vec![Expr::FnCall {
                                name: g,
                                args: gargs,
                                generic_args: ggenerics,
                                span: gspan,
                            }],
                            generic_args: Vec::new(),
                            span: *span,
                        },
                        span: *span,
                    });
                    Some(Expr::Ident(tmp, *span))
                } else {
                    let mut out = Vec::with_capacity(args.len());
                    for a in args {
                        out.push(self.expr(a, pre)?);
                    }
                    Some(Expr::FnCall {
                        name: *name,
                        args: out,
                        generic_args: generic_args.clone(),
                        span: *span,
                    })
                }
            }
            Expr::Binary { op, lhs, rhs, span } => Some(Expr::Binary {
                op: *op,
                lhs: Box::new(self.expr(lhs, pre)?),
                rhs: Box::new(self.expr(rhs, pre)?),
                span: *span,
            }),
            Expr::Unary { op, arg, span } => Some(Expr::Unary {
                op: *op,
                arg: Box::new(self.expr(arg, pre)?),
                span: *span,
            }),
            Expr::Cast { expr, ty, span } => Some(Expr::Cast {
                expr: Box::new(self.expr(expr, pre)?),
                ty: ty.clone(),
                span: *span,
            }),
            Expr::Move(e, span) => Some(Expr::Move(Box::new(self.expr(e, pre)?), *span)),
            Expr::Clone(e, span) => Some(Expr::Clone(Box::new(self.expr(e, pre)?), *span)),
            Expr::ToUnique(e, span) => Some(Expr::ToUnique(Box::new(self.expr(e, pre)?), *span)),
            Expr::Ref(e, m, span) => Some(Expr::Ref(Box::new(self.expr(e, pre)?), *m, *span)),
            Expr::TryOp(e, span) => Some(Expr::TryOp(Box::new(self.expr(e, pre)?), *span)),
            Expr::MethodCall {
                object,
                method,
                args,
                span,
            } => {
                let object = self.expr(object, pre)?;
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.expr(a, pre)?);
                }
                Some(Expr::MethodCall {
                    object: Box::new(object),
                    method: *method,
                    args: out,
                    span: *span,
                })
            }
            Expr::FieldAccess {
                object,
                field,
                span,
            } => Some(Expr::FieldAccess {
                object: Box::new(self.expr(object, pre)?),
                field: *field,
                span: *span,
            }),
            Expr::Index {
                object,
                index,
                span,
            } => Some(Expr::Index {
                object: Box::new(self.expr(object, pre)?),
                index: Box::new(self.expr(index, pre)?),
                span: *span,
            }),
            Expr::StructLiteral {
                type_name,
                generic_args,
                fields,
                span,
            } => {
                let mut out = Vec::with_capacity(fields.len());
                for (n, e) in fields {
                    out.push((*n, self.expr(e, pre)?));
                }
                Some(Expr::StructLiteral {
                    type_name: *type_name,
                    generic_args: generic_args.clone(),
                    fields: out,
                    span: *span,
                })
            }
            Expr::ArrayLiteral(elems, span) => {
                let mut out = Vec::with_capacity(elems.len());
                for e in elems {
                    out.push(self.expr(e, pre)?);
                }
                Some(Expr::ArrayLiteral(out, *span))
            }
            Expr::ArraySized {
                elem_type,
                count,
                span,
            } => Some(Expr::ArraySized {
                elem_type: elem_type.clone(),
                count: Box::new(self.expr(count, pre)?),
                span: *span,
            }),
            Expr::ArrayRepeat { value, count, span } => Some(Expr::ArrayRepeat {
                value: Box::new(self.expr(value, pre)?),
                count: Box::new(self.expr(count, pre)?),
                span: *span,
            }),
            Expr::CallExpr { target, args, span } => {
                let target = self.expr(target, pre)?;
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.expr(a, pre)?);
                }
                Some(Expr::CallExpr {
                    target: Box::new(target),
                    args: out,
                    span: *span,
                })
            }
            Expr::EnumConstruct {
                enum_name,
                variant_name,
                tuple_args,
                named_args,
                span,
            } => {
                let mut targs = Vec::with_capacity(tuple_args.len());
                for a in tuple_args {
                    targs.push(self.expr(a, pre)?);
                }
                let mut nargs = Vec::with_capacity(named_args.len());
                for (n, e) in named_args {
                    nargs.push((*n, self.expr(e, pre)?));
                }
                Some(Expr::EnumConstruct {
                    enum_name: *enum_name,
                    variant_name: *variant_name,
                    tuple_args: targs,
                    named_args: nargs,
                    span: *span,
                })
            }
            // 控制流/宏/汇编表达式内暂不支持 await；无 await 时原样保留。
            _ => {
                if crate::hir::async_cfg::expr_has_await_pub(e) {
                    None
                } else {
                    Some(e.clone())
                }
            }
        }
    }
}
