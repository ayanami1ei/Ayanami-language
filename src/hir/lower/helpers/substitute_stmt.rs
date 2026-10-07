//! 语句级泛型类型替换（`substitute_type_in_stmt` / match 臂）。
use super::*;
use crate::parser::ast::stmt::MatchArm;

pub(crate) fn substitute_type_in_stmt(stmt: &Stmt, subst: &HashMap<Symbol, Type>) -> Stmt {
    match stmt {
        Stmt::Block(block) => Stmt::Block(substitute_type_in_block(block, subst)),
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
        Stmt::StaticDecl { attrs, vis, is_mut, name, ty, value, span } => Stmt::StaticDecl {
            attrs: attrs.clone(),
            vis: *vis,
            is_mut: *is_mut,
            name: *name,
            ty: ty.as_ref().map(|t| substitute_type_in_type(t, subst)),
            value: Box::new(substitute_type_in_expr(value, subst)),
            span: *span,
        },
        Stmt::ConstDecl { attrs, vis, name, ty, value, span } => Stmt::ConstDecl {
            attrs: attrs.clone(),
            vis: *vis,
            name: *name,
            ty: ty.as_ref().map(|t| substitute_type_in_type(t, subst)),
            value: Box::new(substitute_type_in_expr(value, subst)),
            span: *span,
        },
        Stmt::Assign { name, is_mut, value, span } => Stmt::Assign { name: *name, is_mut: *is_mut, value: substitute_type_in_expr(value, subst), span: *span },
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
        Stmt::Return { value, span } => Stmt::Return { value: value.as_ref().map(|v| substitute_type_in_expr(v, subst)), span: *span },
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
        Stmt::ForIn { iterator, iterable, body, span } => Stmt::ForIn { iterator: *iterator, iterable: substitute_type_in_expr(iterable, subst), body: substitute_type_in_block(body, subst), span: *span },
        Stmt::Match { value, arms, span } => Stmt::Match {
            value: Box::new(substitute_type_in_expr(value, subst)),
            arms: arms.iter().map(|a| substitute_match_arm(a, subst)).collect(),
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

/// atb.3：match 臂（含块臂体）的类型替换
pub(crate) fn substitute_match_arm(a: &MatchArm, subst: &HashMap<Symbol, Type>) -> MatchArm {
    MatchArm {
        pattern: a.pattern.clone(),
        guard: a.guard.as_ref().map(|g| substitute_type_in_expr(g, subst)),
        body: match &a.body {
            crate::parser::ast::stmt::MatchBody::Expr(e) =>
                crate::parser::ast::stmt::MatchBody::Expr(substitute_type_in_expr(e, subst)),
            crate::parser::ast::stmt::MatchBody::Block(b) =>
                crate::parser::ast::stmt::MatchBody::Block(substitute_type_in_block(b, subst)),
        },
    }
}
