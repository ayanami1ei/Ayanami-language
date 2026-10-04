use crate::parser::ast::stmt::MatchArm;
use super::*;

pub(crate) fn substitute_type_in_type(ty: &Type, subst: &HashMap<Symbol, Type>) -> Type {
    let s = Span::default();
    match ty {
        Type::Named(name, _) => {
            if let Some(concrete) = subst.get(name) {
                concrete.clone()
            } else {
                Type::Named(*name, s)
            }
        }
        Type::Unique(inner, _) => Type::Unique(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Array(inner, _) => Type::Array(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Ref(inner, mutable, _) => Type::Ref(Box::new(substitute_type_in_type(inner, subst)), *mutable, s),
        Type::Int(_) => Type::Int(s),
        Type::Float(_) => Type::Float(s),
        Type::Char(_) => Type::Char(s),
        Type::Bool(_) => Type::Bool(s),
        Type::Void(_) => Type::Void(s),
        Type::Generic(name, args, _) => Type::Generic(*name, args.iter().map(|a| substitute_type_in_type(a, subst)).collect(), s),
        Type::Default => ty.clone(),
        Type::FnPtr(params, ret, _) => Type::FnPtr(params.iter().map(|p| substitute_type_in_type(p, subst)).collect(), Box::new(substitute_type_in_type(ret, subst)), Span::default()),
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
        Expr::Ident(_, _) => expr.clone(),
        Expr::MacroCall { name, args, span } => Expr::MacroCall {
            name: *name,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
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
        Expr::ArrayLiteral(elems, span) => Expr::ArrayLiteral(
            elems.iter().map(|e| substitute_type_in_expr(e, subst)).collect(),
            *span,
        ),
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
        Expr::Match { .. } => todo!(),
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
            params: params.clone(),
            return_type: return_type.clone(),
            body: body.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
    }
}

/// Substitute generic type parameters in an AST Block.
pub(crate) fn substitute_type_in_block(block: &Block, subst: &HashMap<Symbol, Type>) -> Block {
    Block {
        stmts: block.stmts.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
        span: block.span,
    }
}

/// Substitute generic type parameters in an AST Stmt.
pub(crate) fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt {
    match stmt {
        Stmt::FnDecl { attrs, vis, is_inline, extern_c, name, generic_params: _, params, param_attrs, return_type, body, span } => {
            Stmt::FnDecl {
                attrs: attrs.clone(),
                vis: *vis,
                is_inline: *is_inline,
                extern_c: *extern_c,
                name: *name,
                generic_params: Vec::new(), // cleared: all generics are now concrete
                params: params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
                param_attrs: param_attrs.clone(),
                return_type: substitute_type_in_type(return_type, subst),
                body: substitute_type_in_block(body, subst),
                span: *span,
            }
        }
        Stmt::Assign { name, is_mut, value, span } => Stmt::Assign {
            name: *name,
            is_mut: *is_mut,
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::FieldAssign { object, field, value, span } => Stmt::FieldAssign {
            object: Box::new(substitute_type_in_expr(object, subst)),
            field: *field,
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::IndexAssign { object, index, value, span } => Stmt::IndexAssign {
            object: Box::new(substitute_type_in_expr(object, subst)),
            index: Box::new(substitute_type_in_expr(index, subst)),
            value: substitute_type_in_expr(value, subst),
            span: *span,
        },
        Stmt::Return { value, span } => Stmt::Return {
            value: value.as_ref().map(|v| substitute_type_in_expr(v, subst)),
            span: *span,
        },
        Stmt::If { cond, then_block, elifs, else_block, span } => Stmt::If {
            cond: substitute_type_in_expr(cond, subst),
            then_block: substitute_type_in_block(then_block, subst),
            elifs: elifs.iter().map(|(c, b)| (substitute_type_in_expr(c, subst), substitute_type_in_block(b, subst))).collect(),
            else_block: else_block.as_ref().map(|b| substitute_type_in_block(b, subst)),
            span: *span,
        },
        Stmt::For { iterator, start, end, step, body, span } => Stmt::For {
            iterator: *iterator,
            start: substitute_type_in_expr(start, subst),
            end: substitute_type_in_expr(end, subst),
            step: step.as_ref().map(|s| substitute_type_in_expr(s, subst)),
            body: substitute_type_in_block(body, subst),
            span: *span,
        },
        Stmt::While { cond, body, span } => Stmt::While {
            cond: substitute_type_in_expr(cond, subst),
            body: substitute_type_in_block(body, subst),
            span: *span,
        },
        Stmt::Match { value, arms, span } => Stmt::Match {
            value: Box::new(substitute_type_in_expr(value, subst)),
            arms: arms.iter().map(|a| MatchArm {
                variant_name: a.variant_name,
                bindings: a.bindings.clone(),
                body: substitute_type_in_expr(&a.body, subst),
            }).collect(),
            span: *span,
        },
        Stmt::ExprStmt { expr, span } => Stmt::ExprStmt {
            expr: substitute_type_in_expr(expr, subst),
            span: *span,
        },
        Stmt::Namespace { vis, name, items, span } => Stmt::Namespace {
            vis: *vis,
            name: *name,
            items: items.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
        Stmt::StructDef { attrs, vis, name, fields, field_attrs, span, .. } => Stmt::StructDef {
            attrs: attrs.clone(),
            vis: *vis,
            name: *name,
            generic_params: Vec::new(),
            fields: fields.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
            field_attrs: field_attrs.clone(),
            span: *span,
        },
        Stmt::InterfaceDef { attrs, name, methods, generic_params, span } => Stmt::InterfaceDef {
            attrs: attrs.clone(),
            name: *name,
            generic_params: generic_params.clone(),
            methods: methods.iter().map(|m| {
                InterfaceMethod {
                    attrs: m.attrs.clone(),
                    name: m.name,
                    self_keyword: m.self_keyword,
                    params: m.params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
                    return_type: substitute_type_in_type(&m.return_type, subst),
                }
            }).collect(),
            span: *span,
        },
        Stmt::ImplBlock { attrs, type_name, generic_params, methods, span } => Stmt::ImplBlock {
            attrs: attrs.clone(),
            type_name: *type_name,
            generic_params: generic_params.clone(),
            methods: methods.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
        Stmt::EnumDef { .. } => todo!(),
        Stmt::Import { path, macros, span } => Stmt::Import { path: path.clone(), macros: macros.clone(), span: *span },
        Stmt::Attributed { attrs, stmt, span } => Stmt::Attributed {
            attrs: attrs.clone(),
            stmt: Box::new(substitute_type_in_stmt(stmt, subst)),
            span: *span,
        },
        Stmt::Break { span } => Stmt::Break { span: *span },
        Stmt::Continue { span } => Stmt::Continue { span: *span },
    }
}
