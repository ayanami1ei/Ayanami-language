//! 基于「后续用法」的泛型实参推断。
//! 例：`res = ArrayList::new(); res.push(TokenType::Number(1));`
//! 构造调用本身推不出 T，但后续 `push/set` 的实参类型可以确定元素类型。

use super::*;
use crate::parser::ast::Literal;

impl crate::hir::lower::Ctx {
    /// 收集本块内「构造函数 + 后续 push/set」可推断出的泛型实参
    pub(crate) fn collect_usage_hints(&self, stmts: &[Stmt]) -> HashMap<Symbol, Vec<Type>> {
        let mut hints = HashMap::new();
        for (i, stmt) in stmts.iter().enumerate() {
            let Stmt::Assign { name, value, .. } = stmt else { continue };
            let Expr::FnCall { name: fn_name, generic_args, .. } = value else { continue };
            if !generic_args.is_empty() { continue; }
            // 只处理泛型函数（命名空间构造函数等）
            if !self.generic_fns.iter().any(|(n, _, _)| n == fn_name) { continue; }
            if let Some(ty) = self.find_elem_usage_stmts(name, &stmts[i + 1..]) {
                hints.insert(*name, vec![ty]);
            }
        }
        hints
    }

    /// 递归（含嵌套块）查找 `var.push(x)` / `var.set(i, x)`，返回 x 的类型
    fn find_elem_usage_stmts(&self, var: &Symbol, stmts: &[Stmt]) -> Option<Type> {
        for s in stmts {
            if let Some(t) = self.find_elem_usage(var, s) {
                return Some(t);
            }
            let nested: Option<Type> = match s {
                Stmt::If { then_block, elifs, else_block, .. } => {
                    self.find_elem_usage_stmts(var, &then_block.stmts)
                        .or_else(|| elifs.iter().find_map(|(_, b)| self.find_elem_usage_stmts(var, &b.stmts)))
                        .or_else(|| else_block.as_ref().and_then(|b| self.find_elem_usage_stmts(var, &b.stmts)))
                }
                Stmt::While { body, .. } | Stmt::For { body, .. } => {
                    self.find_elem_usage_stmts(var, &body.stmts)
                }
                Stmt::Attributed { stmt, .. } => self.find_elem_usage_stmts(var, std::slice::from_ref(stmt)),
                _ => None,
            };
            if nested.is_some() {
                return nested;
            }
        }
        None
    }

    /// 在语句里找 `var.push(x)` / `var.set(i, x)`，返回 x 的类型
    fn find_elem_usage(&self, var: &Symbol, stmt: &Stmt) -> Option<Type> {
        let expr = match stmt {
            Stmt::ExprStmt { expr, .. } => expr,
            Stmt::Assign { value, .. } => value,
            _ => return None,
        };
        let Expr::MethodCall { object, method, args, .. } = expr else { return None };
        let Expr::Ident(obj, _) = object.as_ref() else { return None };
        if *obj != *var { return None; }
        let arg = match method.as_str().as_str() {
            "push" => args.first(),
            "set" => args.get(1),
            _ => None,
        }?;
        self.infer_expr_type_ast(arg)
    }

    /// 轻量 AST 类型推断（够用即可：字面量/枚举/结构体/单签名函数/已注册变量）
    fn infer_expr_type_ast(&self, e: &Expr) -> Option<Type> {
        match e {
            Expr::Literal(Literal::Int(..)) => Some(Type::Int(Span::default())),
            Expr::Literal(Literal::Float(..)) => Some(Type::Float(Span::default())),
            Expr::Literal(Literal::Char(..)) => Some(Type::Char(Span::default())),
            Expr::Literal(Literal::Bool(..)) => Some(Type::Bool(Span::default())),
            Expr::Literal(Literal::String(..)) => Some(Type::Named(Symbol::intern("String"), Span::default())),
            Expr::EnumConstruct { enum_name, .. } => Some(Type::Named(*enum_name, Span::default())),
            Expr::StructLiteral { type_name, generic_args, .. } => {
                if generic_args.is_empty() {
                    Some(Type::Named(*type_name, Span::default()))
                } else {
                    Some(Type::Generic(*type_name, generic_args.clone(), Span::default()))
                }
            }
            Expr::FnCall { name, .. } => {
                // 枚举构造被解析器合并为 `Enum.Variant(...)`
                if let Some((prefix, _)) = name.as_str().split_once('.') {
                    let pfx = Symbol::intern(prefix);
                    if self.is_enum_type(&pfx) {
                        return Some(Type::Named(pfx, Span::default()));
                    }
                }
                let ids = self.fn_map.get(name)?;
                if ids.len() == 1 {
                    let ret = self.fns[ids[0].0].return_type.clone();
                    Some(hir_type_to_ast_type(&ret))
                } else {
                    None
                }
            }
            Expr::Ident(v, _) => {
                // 无参枚举变体 `Enum.Variant`
                if let Some((prefix, _)) = v.as_str().split_once('.') {
                    let pfx = Symbol::intern(prefix);
                    if self.is_enum_type(&pfx) {
                        return Some(Type::Named(pfx, Span::default()));
                    }
                }
                self.lookup_var(v).map(|(_, ty, _)| hir_type_to_ast_type(&ty))
            }
            _ => None,
        }
    }
}
