use super::*;

pub(crate) fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type {
    let s = Span::default();
    match ty {
        Type::Never(sp) => Type::Never(*sp),
        Type::Named(name, _) => {
            if let Some(concrete) = subst.get(name) {
                concrete.clone()
            } else {
                Type::Named(*name, s)
            }
        }
        Type::Unique(inner, _) => Type::Unique(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Array(inner, _) => Type::Array(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::ArraySized(inner, n, _) => Type::ArraySized(Box::new(substitute_type_in_type(inner, subst)), *n, s),
        Type::Ref(inner, mutable, _) => Type::Ref(Box::new(substitute_type_in_type(inner, subst)), *mutable, s),
        Type::Int(_) => Type::Int(s),
        Type::Float(_) => Type::Float(s),
        Type::Char(_) => Type::Char(s),
        Type::Bool(_) => Type::Bool(s),
        Type::Void(_) => Type::Void(s),
        Type::Generic(name, args, _) => Type::Generic(*name, args.iter().map(|a| substitute_type_in_type(a, subst)).collect(), s),
        Type::Default => ty.clone(),
        Type::FnPtr(params, ret, _) => Type::FnPtr(params.iter().map(|p| substitute_type_in_type(p, subst)).collect(), Box::new(substitute_type_in_type(ret, subst)), Span::default()),
        Type::Closure(params, ret, _, once) => Type::Closure(params.iter().map(|p| substitute_type_in_type(p, subst)).collect(), Box::new(substitute_type_in_type(ret, subst)), Span::default(), *once),
        Type::Self_(_) => ty.clone(),
    }
}

/// Substitute generic type parameters in an AST Expr.
pub(crate) fn substitute_type_in_expr(expr: &Expr, subst: &HashMap<Symbol, Type>) -> Expr {
    match expr {
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary {
            op: *op,
            lhs: Box::new(substitute_type_in_expr(lhs, subst)),
            rhs: Box::new(substitute_type_in_expr(rhs, subst)),
            span: *span,
        },
        Expr::Unary { op, arg, span } => Expr::Unary {
            op: *op,
            arg: Box::new(substitute_type_in_expr(arg, subst)),
            span: *span,
        },
        Expr::Cast { expr, ty, span } => Expr::Cast {
            expr: Box::new(substitute_type_in_expr(expr, subst)),
            ty: substitute_type_in_type(ty, subst),
            span: *span,
        },
        Expr::Literal(_) => expr.clone(),
        Expr::Suffixed { .. } => expr.clone(),
        Expr::If { cond, then_block, elifs, else_block, span } => Expr::If {
            cond: Box::new(substitute_type_in_expr(cond, subst)),
            then_block: substitute_type_in_block(then_block, subst),
            elifs: elifs.iter().map(|(c, b)| (substitute_type_in_expr(c, subst), substitute_type_in_block(b, subst))).collect(),
            else_block: else_block.as_ref().map(|b| substitute_type_in_block(b, subst)),
            span: *span,
        },
        Expr::Ident(_, _) => expr.clone(),
        Expr::MacroCall { name, args, span } => Expr::MacroCall {
            name: *name,
            args: args.iter().map(|a| match a {
                crate::parser::ast::expr::MacroArg::Expr(e) =>
                    crate::parser::ast::expr::MacroArg::Expr(substitute_type_in_expr(e, subst)),
                crate::parser::ast::expr::MacroArg::Block(b) =>
                    crate::parser::ast::expr::MacroArg::Block(substitute_type_in_block(b, subst)),
            }).collect(),
            span: *span,
        },
        Expr::FnCall { name, args, generic_args, span } => Expr::FnCall {
            name: *name,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
            generic_args: generic_args.iter().map(|t| substitute_type_in_type(t, subst)).collect(),
            span: *span,
        },
        Expr::MethodCall { object, method, args, span } => Expr::MethodCall {
            object: Box::new(substitute_type_in_expr(object, subst)),
            method: *method,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
            span: *span,
        },
        Expr::StructLiteral { type_name, generic_args, fields, span } => Expr::StructLiteral {
            type_name: *type_name,
            generic_args: generic_args.iter().map(|a| substitute_type_in_type(a, subst)).collect(),
            fields: fields.iter().map(|(n, e)| (*n, substitute_type_in_expr(e, subst))).collect(),
            span: *span,
        },
        Expr::ArrayLiteral(elems, span) => Expr::ArrayLiteral(elems.iter().map(|e| substitute_type_in_expr(e, subst)).collect(), *span),
        Expr::ArrayRepeat { value, count, span } => Expr::ArrayRepeat {
            value: Box::new(substitute_type_in_expr(value, subst)), count: Box::new(substitute_type_in_expr(count, subst)), span: *span,
        },
        Expr::ArraySized { elem_type, count, span } => Expr::ArraySized {
            elem_type: substitute_type_in_type(elem_type, subst),
            count: Box::new(substitute_type_in_expr(count, subst)),
            span: *span,
        },
        Expr::Move(inner, span) => Expr::Move(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::Clone(inner, span) => Expr::Clone(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::ToUnique(inner, span) => Expr::ToUnique(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::FieldAccess { object, field, span } => Expr::FieldAccess {
            object: Box::new(substitute_type_in_expr(object, subst)),
            field: *field,
            span: *span,
        },
        Expr::Null(span) => Expr::Null(*span),
        Expr::Ref(inner, mutable, span) => Expr::Ref(
            Box::new(substitute_type_in_expr(inner, subst)),
            *mutable,
            *span,
        ),
        Expr::Asm { template, outputs, inputs, span } => Expr::Asm {
            template: template.clone(),
            outputs: outputs.iter().map(|(c, e)| (c.clone(), Box::new(substitute_type_in_expr(e, subst)))).collect(),
            inputs: inputs.iter().map(|(c, e)| (c.clone(), Box::new(substitute_type_in_expr(e, subst)))).collect(),
            span: *span,
        },
        Expr::Index { object, index, span } => Expr::Index {
            object: Box::new(substitute_type_in_expr(object, subst)),
            index: Box::new(substitute_type_in_expr(index, subst)),
            span: *span,
        },
        Expr::CallExpr { target, args, span } => Expr::CallExpr {
            target: Box::new(substitute_type_in_expr(target, subst)),
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
            span: *span,
        },
        Expr::TryOp(inner, span) => Expr::TryOp(
            Box::new(substitute_type_in_expr(inner, subst)),
            *span,
        ),
        Expr::Match { value, arms, span } => Expr::Match {
            value: Box::new(substitute_type_in_expr(value, subst)),
            arms: arms.iter().map(|a| substitute_match_arm(a, subst)).collect(),
            span: *span,
        },
        Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, span } => {
            Expr::EnumConstruct {
                enum_name: *enum_name,
                variant_name: *variant_name,
                tuple_args: tuple_args.iter().map(|e| substitute_type_in_expr(e, subst)).collect(),
                named_args: named_args.iter().map(|(n, e)| (*n, substitute_type_in_expr(e, subst))).collect(),
                span: *span,
            }
        }
        Expr::Lambda { params, return_type, body, span } => Expr::Lambda {
            params: params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
            return_type: substitute_type_in_type(return_type, subst),
            body: substitute_type_in_block(body, subst),
            span: *span,
        },
    }
}

/// Substitute generic type parameters in an AST Block.
pub(crate) fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block {
    Block {
        stmts: block.stmts.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
        tail: block.tail.as_ref().map(|e| Box::new(substitute_type_in_expr(e, subst))),
        span: block.span,
    }
}
