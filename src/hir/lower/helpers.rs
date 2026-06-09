// ============================================================
//  辅助函数集 —— 供 HIR 降级过程中使用的工具函数
//  包括：表达式类型推断、所有权转换、泛型参数推断、
//  类型替换、类型格式化显示、运算符名映射等
// ============================================================


use std::collections::HashMap;
use crate::intern::Symbol;
use crate::parser::ast::*;
use crate::span::Span;
use crate::hir::ir::*;
use super::strip_generic_name;
use super::InterfaceReg;

    /// 自动插入 Move 包装：如果表达式是 unique 类型且尚未包装，则包装为 Move
pub(crate) fn implicit_move(expr: HirExpr) -> HirExpr {
    let ty = expr_type(&expr);
    if matches!(ty, HirType::Unique(_))
        && !matches!(expr, HirExpr::Move(_, _) | HirExpr::Clone(_, _))
    {
        HirExpr::Move(Box::new(expr), ty)
    } else {
        expr
    }
}

/// 包装参数以匹配期望的参数类型（处理所有权转换）
///
/// 支持以下转换：
/// - `T` → `Unique(T)`：ToUnique（堆分配）
/// - `T` → `Shared(T)`：ToShared（堆分配 + retain）
/// - `T` → `Weak(T)`：ToWeak（创建弱引用）
/// - `Unique(T)` → `Shared(T)`：ToShared（仅 retain，不拷贝）
/// - `Unique(T)` → `Weak(T)`：ToWeak（仅复制指针）
/// - `Shared(T)` → `Unique(T)`：ToUnique（拷贝 + 堆分配）
/// - `Shared(T)` → `Weak(T)`：ToWeak（仅复制指针）
pub(crate) fn wrap_arg_for_param(arg: HirExpr, param_ty: &HirType) -> HirExpr {
    let arg_ty = expr_type(&arg);
    match param_ty {
        HirType::Unique(pt) | HirType::Shared(pt) | HirType::Weak(pt) => {
            if arg_ty == *pt.as_ref() {
                // 裸类型 → 所有权类型：自动包装
                match param_ty {
                    HirType::Unique(_) => {
                        HirExpr::ToUnique(Box::new(arg), param_ty.clone())
                    }
                    HirType::Shared(_) => {
                        HirExpr::ToShared(Box::new(arg), param_ty.clone())
                    }
                    HirType::Weak(_) => {
                        HirExpr::ToWeak(Box::new(arg), param_ty.clone())
                    }
                    _ => arg,
                }
            } else if let HirType::Unique(inner) = &arg_ty {
                // arg 是 Unique(T)，param 想要所有权类型
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Unique(_) => {
                            // Unique(T) → Unique(T)：需要 Move 包装以转移所有权
                            wrap_for_unique_param(arg, param_ty)
                        }
                        HirType::Shared(_) => {
                            HirExpr::ToShared(Box::new(arg), param_ty.clone())
                        }
                        HirType::Weak(_) => {
                            HirExpr::ToWeak(Box::new(arg), param_ty.clone())
                        }
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if let HirType::Shared(inner) = &arg_ty {
                // arg 是 Shared(T)，param 想要其他所有权类型
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Shared(_) => {
                            arg
                        }
                        HirType::Unique(_) => {
                            HirExpr::ToUnique(Box::new(arg), param_ty.clone())
                        }
                        HirType::Weak(_) => {
                            HirExpr::ToWeak(Box::new(arg), param_ty.clone())
                        }
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if let HirType::Weak(inner) = &arg_ty {
                // arg 是 Weak(T)，param 想要其他所有权类型
                if **inner == *pt.as_ref() {
                    match param_ty {
                        HirType::Weak(_) => {
                            arg
                        }
                        HirType::Shared(_) => {
                            HirExpr::ToShared(Box::new(arg), param_ty.clone())
                        }
                        HirType::Unique(_) => {
                            HirExpr::ToUnique(Box::new(arg), param_ty.clone())
                        }
                        _ => arg,
                    }
                } else {
                    arg
                }
            } else if matches!(param_ty, HirType::Unique(_)) {
                wrap_for_unique_param(arg, param_ty)
            } else {
                arg
            }
        }
        _ => arg,
    }
}

/// Like implicit_move, but also wraps plain values when the param expects Unique.
pub(crate) fn wrap_for_unique_param(expr: HirExpr, param_ty: &HirType) -> HirExpr {
    let ty = expr_type(&expr);
    if matches!(param_ty, HirType::Unique(_))
        && !matches!(expr, HirExpr::Move(_, _) | HirExpr::Clone(_, _))
    {
        HirExpr::Move(Box::new(expr), ty)
    } else {
        expr
    }
}

// ====================================================================
//  Generic substitution: AST-level type replacement
// ====================================================================

/// Convert a HirType back to an AST Type (for substitution into generic body).
/// Uses Span::default() for synthetic nodes.
pub(crate) fn hir_type_to_ast_type(ty: &HirType) -> Type {
    let s = Span::default();
    match ty {
        HirType::Int => Type::Int(s),
        HirType::Float => Type::Float(s),
        HirType::Char => Type::Char(s),
        HirType::Bool => Type::Bool(s),
        HirType::Void => Type::Void(s),
        HirType::Named(n) => Type::Named(*n, s),
        HirType::Unique(inner) => Type::Unique(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Shared(inner) => Type::Shared(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Weak(inner) => Type::Weak(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::Array(inner) => Type::Array(Box::new(hir_type_to_ast_type(inner)), s),
        HirType::FatPtr { name, kind } => {
            let inner = Type::Named(*name, s);
            match kind.as_ref() {
                HirType::Shared(_) => Type::Shared(Box::new(inner), s),
                HirType::Unique(_) => Type::Unique(Box::new(inner), s),
                _ => inner,
            }
        }
        HirType::Ref(inner, mutable) => Type::Ref(Box::new(hir_type_to_ast_type(inner)), *mutable, s),
    }
}

/// Given an AST param type and the corresponding HirType from the lowered arg,
/// extract the binding for a generic parameter name (if the param type uses it).
pub(crate) fn infer_generic_from_param<'a>(param_ty: &'a Type, arg_ty: &'a HirType) -> Option<(Symbol, HirType)> {
    match (param_ty, arg_ty) {
        (Type::Named(name, _), _) => Some((*name, arg_ty.clone())),
        // Generic("LinkedList", [Named("T")]) vs Named("LinkedList<int>") → infer T = int
        (Type::Generic(name, params, _), _) => {
            let base = name.as_str();
            // 剥离所有所有权包装，获取底层的 Named 类型名
            let stripped = strip_ownership_ref(arg_ty);
            let arg_name = match stripped {
                HirType::Named(n) => n.as_str(),
                _ => return None,
            };
            // 如果 arg 本身就是泛型参数（如 Named("T")），直接映射
            for gp in params.iter() {
                if let Type::Named(gp_name, _) = gp {
                    if arg_name == gp_name.as_str() {
                        return Some((*gp_name, arg_ty.clone()));
                    }
                }
            }
            // Check if arg_name is "Name<...>"
            if let Some(start) = arg_name.find('<') {
                if &arg_name[..start] == base {
                    let inner = arg_name[start..].trim_start_matches('<').trim_end_matches('>');
                    let inner_parts: Vec<&str> = inner.split(',').collect();
                    // Decode each inner part: "int" → Int, "String" → Named("String")
                    for (gp, inner_str) in params.iter().zip(inner_parts.iter()) {
                        if let Type::Named(gp_name, _) = gp {
                            let hir_inner = sig_str_to_hir(inner_str.trim());
                            return Some((*gp_name, hir_inner));
                        }
                    }
                }
            }
            None
        }
        (Type::Unique(inner, _), HirType::Unique(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        (Type::Shared(inner, _), HirType::Shared(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        (Type::Weak(inner, _), HirType::Weak(hir_inner)) => infer_generic_from_param(inner, hir_inner),
        // Param expects wrapper but arg is unwrapped (auto-wrap will handle)
        (Type::Unique(inner, _) | Type::Shared(inner, _) | Type::Weak(inner, _), _) => {
            infer_generic_from_param(inner, arg_ty)
        }
        _ => None,
    }
}

/// Substitute generic type parameters in an AST Type node.
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
        Type::Shared(inner, _) => Type::Shared(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Weak(inner, _) => Type::Weak(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Array(inner, _) => Type::Array(Box::new(substitute_type_in_type(inner, subst)), s),
        Type::Ref(inner, mutable, _) => Type::Ref(Box::new(substitute_type_in_type(inner, subst)), *mutable, s),
        Type::Int(_) => Type::Int(s),
        Type::Float(_) => Type::Float(s),
        Type::Char(_) => Type::Char(s),
        Type::Bool(_) => Type::Bool(s),
        Type::Void(_) => Type::Void(s),
        Type::Generic(name, args, _) => Type::Generic(*name, args.iter().map(|a| substitute_type_in_type(a, subst)).collect(), s),
        Type::Default => ty.clone(),
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
        Expr::Literal(_) => expr.clone(),
        Expr::Ident(_, _) => expr.clone(),
        Expr::FnCall { name, args, span } => Expr::FnCall {
            name: *name,
            args: args.iter().map(|a| substitute_type_in_expr(a, subst)).collect(),
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
        Expr::ToShared(inner, span) => Expr::ToShared(Box::new(substitute_type_in_expr(inner, subst)), *span),
        Expr::ToWeak(inner, span) => Expr::ToWeak(Box::new(substitute_type_in_expr(inner, subst)), *span),
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
        Stmt::FnDecl { vis, is_inline, extern_c, name, generic_params: _, params, return_type, body, span } => {
            Stmt::FnDecl {
                vis: *vis,
                is_inline: *is_inline,
                extern_c: *extern_c,
                name: *name,
                generic_params: Vec::new(), // cleared: all generics are now concrete
                params: params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
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
        Stmt::StructDef { vis, name, fields, span, .. } => Stmt::StructDef {
            vis: *vis,
            name: *name,
            generic_params: Vec::new(),
            fields: fields.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
            span: *span,
        },
        Stmt::InterfaceDef { name, methods, generic_params, span } => Stmt::InterfaceDef {
            name: *name,
            generic_params: generic_params.clone(),
            methods: methods.iter().map(|m| {
                InterfaceMethod {
                    name: m.name,
                    self_keyword: m.self_keyword,
                    params: m.params.iter().map(|(n, t)| (*n, substitute_type_in_type(t, subst))).collect(),
                    return_type: substitute_type_in_type(&m.return_type, subst),
                }
            }).collect(),
            span: *span,
        },
        Stmt::ImplBlock { type_name, generic_params, methods, span } => Stmt::ImplBlock {
            type_name: *type_name,
            generic_params: generic_params.clone(),
            methods: methods.iter().map(|s| substitute_type_in_stmt(s, subst)).collect(),
            span: *span,
        },
        Stmt::Import { path, span } => Stmt::Import { path: path.clone(), span: *span },
        Stmt::Break { span } => Stmt::Break { span: *span },
        Stmt::Continue { span } => Stmt::Continue { span: *span },
    }
}

// ====================================================================
//  Type helpers
// ====================================================================

/// Parse a type from a package signature string like "int", "shared Point", "[int]", etc.
/// Convert an AST Type to a string for generic encoding.
pub(crate) fn type_to_string_generic(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> String {
    match ty {
        Type::Default => "?".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Generic(name, args, _) => {
            let a: Vec<String> = args.iter().map(|a| type_to_string_generic(a, interfaces)).collect();
            format!("{}<{}>", name, a.join(","))
        }
        Type::Array(inner, _) => format!("[{}]", type_to_string_generic(inner, interfaces)),
        Type::Ref(inner, mutable, _) => format!("ref{}{}",
            if *mutable { " mut" } else { "" },
            type_to_string_generic(inner, interfaces)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string_generic(inner, interfaces)),
        Type::Shared(inner, _) => format!("shared {}", type_to_string_generic(inner, interfaces)),
        Type::Weak(inner, _) => format!("weak {}", type_to_string_generic(inner, interfaces)),
        Type::Self_(_) => "Self".into(),
    }
}

/// Substitute generic type parameters in a HirType.
pub(crate) fn substitute_hir_type(ty: &HirType, subst: &HashMap<Symbol, HirType>) -> HirType {
    match ty {
        HirType::Named(s) => {
            // 直接匹配泛型参数名（如 T → int）
            if let Some(replacement) = subst.get(s) {
                return replacement.clone();
            }
            // 处理编码的泛型类型名（如 "LinkedListNode<T>" 或 "LinkedListNode[T]"）
            let name_str = s.as_str();
            let open = name_str.find('<').or_else(|| name_str.find('['));
            if let Some(start) = open {
                let base = &name_str[..start];
                let inner = name_str[start..].trim_start_matches('<').trim_start_matches('[').trim_end_matches('>').trim_end_matches(']');
                let parts: Vec<&str> = inner.split(',').collect();
                let mut changed = false;
                let new_parts: Vec<String> = parts.iter().map(|p| {
                    let trimmed = p.trim();
                    let sym = Symbol::intern(trimmed);
                    if let Some(replacement) = subst.get(&sym) {
                        changed = true;
                        hir_type_display(replacement)
                    } else {
                        trimmed.to_string()
                    }
                }).collect();
                if changed {
                    let new_name = format!("{}<{}>", base, new_parts.join(","));
                    return HirType::Named(Symbol::intern(&new_name));
                }
            }
            ty.clone()
        }
        HirType::Unique(inner) => HirType::Unique(Box::new(substitute_hir_type(inner, subst))),
        HirType::Shared(inner) => HirType::Shared(Box::new(substitute_hir_type(inner, subst))),
        HirType::Weak(inner) => HirType::Weak(Box::new(substitute_hir_type(inner, subst))),
        HirType::Array(inner) => HirType::Array(Box::new(substitute_hir_type(inner, subst))),
        HirType::Ref(inner, mutable) => HirType::Ref(Box::new(substitute_hir_type(inner, subst)), *mutable),
        HirType::FatPtr { name, kind } => HirType::FatPtr {
            name: *name,
            kind: Box::new(substitute_hir_type(kind, subst)),
        },
        _ => ty.clone(),
    }
}

    /// 将字符串形式签名（如 "unique String"）解析为 HirType
pub(crate) fn sig_str_to_hir(s: &str) -> HirType {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("shared ") {
        HirType::Shared(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("unique ") {
        HirType::Unique(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix("weak ") {
        HirType::Weak(Box::new(sig_str_to_hir(inner)))
    } else if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        HirType::Array(Box::new(sig_str_to_hir(inner)))
    } else {
        match s {
            "int" => HirType::Int,
            "float" => HirType::Float,
            "char" => HirType::Char,
            "bool" => HirType::Bool,
            "void" => HirType::Void,
            other => HirType::Named(Symbol::intern(other)),
        }
    }
}

    /// 将 AST 类型节点转换为 HIR 类型（含接口信息）
fn is_iface_type(inner_hir: &HirType, interfaces: &HashMap<Symbol, super::InterfaceReg>) -> bool {
    match inner_hir {
        HirType::Named(n) if interfaces.contains_key(n) => true,
        HirType::Named(n) => {
            let base = strip_generic_name(n);
            base != *n && interfaces.contains_key(&base)
        }
        _ => false,
    }
}

pub(crate) fn ast_type_to_hir(ty: &Type, interfaces: &HashMap<Symbol, InterfaceReg>) -> HirType {
    match ty {
        Type::Default | Type::Int(_) => HirType::Int,
        Type::Float(_) => HirType::Float,
        Type::Char(_) => HirType::Char,
        Type::Bool(_) => HirType::Bool,
        Type::Void(_) => HirType::Void,
        Type::Array(inner, _) => HirType::Array(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Generic(name, args, _) => {
            // Encode generic instantiation as a unique named type
            let args_str: Vec<String> = args.iter()
                .map(|a| type_to_string_generic(a, interfaces))
                .collect();
            HirType::Named(Symbol::intern(&format!("{}<{}>", name, args_str.join(","))))
        }
        Type::Named(s, _) => {
            let name = s.as_str();
            if name == "int" { HirType::Int }
            else if name == "float" { HirType::Float }
            else if name == "char" { HirType::Char }
            else if name == "void" { HirType::Void }
            else if name == "bool" { HirType::Bool }
            else { HirType::Named(*s) }
        }
        Type::Unique(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if is_iface_type(&inner_hir, interfaces) {
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Unique(Box::new(HirType::Void))) }
            } else {
                HirType::Unique(Box::new(inner_hir))
            }
        }
        Type::Shared(inner, _) => {
            let inner_hir = ast_type_to_hir(inner, interfaces);
            if is_iface_type(&inner_hir, interfaces) {
                HirType::FatPtr { name: *extract_named(&inner_hir).unwrap(), kind: Box::new(HirType::Shared(Box::new(HirType::Void))) }
            } else {
                HirType::Shared(Box::new(inner_hir))
            }
        }
        Type::Weak(inner, _) => HirType::Weak(Box::new(ast_type_to_hir(inner, interfaces))),
        Type::Ref(inner, mutable, _) => HirType::Ref(Box::new(ast_type_to_hir(inner, interfaces)), *mutable),
        Type::Self_(_) => {
            // Self_ should not appear outside impl blocks since the parser
            // already fills in the concrete type
            HirType::Void
        }
    }
}

    /// 从 HirType 中提取命名类型的名称（剥去所有权包装）
pub(crate) fn extract_named(ty: &HirType) -> Option<&Symbol> {
    match ty {
        HirType::Named(s) => Some(s),
        _ => None,
    }
}

    /// 将 HirType 格式化为可读字符串（用于错误消息和调试输出）
pub(crate) fn hir_type_display(ty: &HirType) -> String {
    match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::Char => "char".into(),
        HirType::Void => "void".into(),
        HirType::Bool => "bool".into(),
        HirType::Named(s) => s.as_str().to_string(),
        HirType::Unique(inner) => format!("unique {}", hir_type_display(inner)),
        HirType::Shared(inner) => format!("shared {}", hir_type_display(inner)),
        HirType::Weak(inner) => format!("weak {}", hir_type_display(inner)),
        HirType::FatPtr { name, kind } => format!("{} {}", hir_type_display(kind), name.as_str()),
        HirType::Array(inner) => format!("[{}]", hir_type_display(inner)),
        HirType::Ref(inner, mutable) => {
            if *mutable {
                format!("ref mut {}", hir_type_display(inner))
            } else {
                format!("ref {}", hir_type_display(inner))
            }
        }
    }
}

/// Check if a type needs deep copy (heap-allocated data).
pub(crate) fn needs_deep_copy(ty: &HirType) -> bool {
    matches!(ty, HirType::Named(_) | HirType::Array(_) | HirType::FatPtr { .. })
}

/// Map binary operators to function names for operator overloading.
pub(crate) fn binary_op_to_fn_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("add"),
        BinaryOp::Sub => Some("sub"),
        BinaryOp::Mul => Some("mul"),
        BinaryOp::Div => Some("div"),
        BinaryOp::Mod => Some("rem"),
        BinaryOp::Eq => Some("eq"),
        BinaryOp::Neq => Some("ne"),
        BinaryOp::Lt => Some("lt"),
        BinaryOp::Gt => Some("gt"),
        BinaryOp::Le => Some("le"),
        BinaryOp::Ge => Some("ge"),
        BinaryOp::And | BinaryOp::Or => None, // logical ops not overloadable
    }
}

    /// 例如：`-`（负号）→ "neg"
    /// 将一元运算符映射到对应的方法名（用于运算符重载查找）
pub(crate) fn unary_op_to_fn_name(op: &UnaryOp) -> Option<&'static str> {
    match op {
        UnaryOp::Neg => Some("neg"),
        UnaryOp::Not => Some("not"),
    }
}

    /// 剥去所有权包装（Unique/Shared/Weak），返回内部类型
pub(crate) fn strip_ownership(ty: HirType) -> HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => *inner,
        other => other,
    }
}

    /// 剥去所有权包装的引用版本（不消耗所有权）
pub(crate) fn strip_ownership_ref(ty: &HirType) -> &HirType {
    match ty {
        HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
        other => other,
    }
}

    /// 推断 HIR 表达式的类型
pub(crate) fn expr_type(expr: &HirExpr) -> HirType {
    match expr {
        HirExpr::Literal(_, ty)
        | HirExpr::Local(_, ty)
        | HirExpr::Binary { ty, .. }
        | HirExpr::Unary { ty, .. }
        | HirExpr::Call { ty, .. }
        | HirExpr::Move(_, ty)
        | HirExpr::Clone(_, ty)
        | HirExpr::ToUnique(_, ty)
        | HirExpr::ToShared(_, ty)
        | HirExpr::ToWeak(_, ty)
        | HirExpr::VirtualCall { ty, .. }
        | HirExpr::MakeFatPtr { ty, .. }
        | HirExpr::FieldAccess { ty, .. }
        | HirExpr::StructLiteral { ty, .. }
        | HirExpr::ArraySized { ty, .. }
        | HirExpr::ArrayLiteral(_, ty)
        | HirExpr::Index { ty, .. }
        | HirExpr::Ref { ty, .. }
        | HirExpr::Asm { ty, .. } => ty.clone(),
    }
}

/// Check if a HirExpr is a null literal (lowered to Int(0)).
pub(crate) fn is_null_literal(expr: &HirExpr) -> bool {
    matches!(expr, HirExpr::Literal(HirLiteral::Int(0), HirType::Int))
}

/// Check if a type is a pointer-like type for null comparison purposes.
pub(crate) fn is_pointer_type_for_cmp(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::Shared(_) | HirType::Unique(_)
        | HirType::Weak(_) | HirType::FatPtr { .. } | HirType::Array(_)
    )
}
