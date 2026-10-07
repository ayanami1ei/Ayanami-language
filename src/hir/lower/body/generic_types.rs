//! C1：泛型体浅层类型推断（用于声明期 T 上方法调用检查）。
//!
//! 只回答一个问题：某表达式是否**确定为泛型参数 `T`**（或结构体基名）。
//! 类型不确定时一律 `Other`（保守，不误报）。

use super::*;

/// 浅层类型：只区分泛型参数 / 结构体基名 / 其他。
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum GType {
    /// 类型是泛型参数 `T`
    Param(Symbol),
    /// 类型是结构体/枚举（基名），用于字段与方法返回推断
    Struct(Symbol),
    Other,
}

/// C3a：AST 类型是否为具体基元（int/float/char/bool/定宽整数/f32）。
pub(super) fn ast_is_concrete_primitive(ty: &Type) -> bool {
    match ty {
        Type::Int(_) | Type::Float(_) | Type::Char(_) | Type::Bool(_) => true,
        Type::Named(n, _) => fixed_width_type(&n.as_str()).is_some(),
        _ => false,
    }
}

/// C3a：HIR 类型是否为具体基元。
pub(super) fn hir_is_concrete_primitive(ty: &HirType) -> bool {
    matches!(
        ty,
        HirType::Int | HirType::Float | HirType::F32 | HirType::Char | HirType::Bool | HirType::IntN { .. }
    )
}

impl crate::hir::lower::Ctx {
    /// AST 类型 → 浅层 `GType`。
    pub(crate) fn ast_gtype(&self, ty: &Type, gp: &[(Symbol, Vec<Symbol>)]) -> GType {
        match ty {
            Type::Named(n, _) => {
                if gp.iter().any(|(g, _)| g == n) {
                    GType::Param(*n)
                } else if self.struct_defs.contains_key(n) {
                    GType::Struct(*n)
                } else {
                    GType::Other
                }
            }
            Type::Generic(n, _, _) => {
                if self.struct_defs.contains_key(n) { GType::Struct(*n) } else { GType::Other }
            }
            Type::Ref(inner, _, _) | Type::Unique(inner, _) => self.ast_gtype(inner, gp),
            _ => GType::Other,
        }
    }

    /// 表达式浅层类型推断（仅 Ident / 字段 / 泛型 impl 方法返回）。
    pub(crate) fn infer_gtype(
        &self,
        expr: &Expr,
        gp: &[(Symbol, Vec<Symbol>)],
        env: &HashMap<Symbol, GType>,
    ) -> GType {
        match expr {
            Expr::Ident(v, _) => env.get(v).cloned().unwrap_or(GType::Other),
            Expr::Move(i, _) | Expr::Clone(i, _) | Expr::ToUnique(i, _)
            | Expr::Ref(i, _, _) | Expr::TryOp(i, _) => self.infer_gtype(i, gp, env),
            Expr::FieldAccess { object, field, .. } => {
                if let GType::Struct(base) = self.infer_gtype(object, gp, env) {
                    if let Some(f) = self.struct_defs.get(&base)
                        .and_then(|fs| fs.iter().find(|f| f.name == *field))
                    {
                        if let HirType::Named(p) = strip_ownership_ref(&f.ty) {
                            if gp.iter().any(|(g, _)| g == p) {
                                return GType::Param(*p);
                            }
                        }
                    }
                }
                GType::Other
            }
            Expr::MethodCall { object, method, .. } => {
                if let GType::Struct(base) = self.infer_gtype(object, gp, env) {
                    for (m, _, stmt) in &self.generic_fns {
                        if m != method { continue; }
                        if let Stmt::FnDecl { params, return_type, .. } = stmt {
                            if let Some((_, rt)) = params.first() {
                                if matches!(self.ast_gtype(rt, gp), GType::Struct(b) if b == base) {
                                    return self.ast_gtype(return_type, gp);
                                }
                            }
                        }
                    }
                }
                GType::Other
            }
            _ => GType::Other,
        }
    }

    /// C2：泛型参数 `T` 参与的运算符按约束检查（映射到 add/sub/.../eq/... 方法）。
    pub(crate) fn check_param_operator(
        &self,
        p: &Symbol,
        method: Option<&str>,
        argc: usize,
        span: &Span,
        gp: &[(Symbol, Vec<Symbol>)],
        op_display: &str,
    ) -> Result<()> {
        let Some(m) = method else {
            return Err(Error::Hir(format!(
                "operator `{}` cannot be applied to generic parameter `{}` (not overloadable) (at {}:{})",
                op_display, p.as_str(), span.start_line, span.start_col
            )));
        };
        let bounds: Vec<Symbol> = gp.iter().find(|(n, _)| n == p).map(|(_, c)| c.clone()).unwrap_or_default();
        if bounds.is_empty() {
            return Err(Error::Hir(format!(
                "operator `{}` on generic parameter `{}` requires a bound providing `{}` (e.g. `[{}: Iface]`) (at {}:{})",
                op_display, p.as_str(), m, p.as_str(), span.start_line, span.start_col
            )));
        }
        let ok = bounds.iter().any(|iface| {
            self.interfaces.get(iface).or_else(|| {
                self.interfaces.get(&crate::hir::lower::strip_generic_name(iface))
            }).map(|r| {
                r.methods.iter().any(|mm| mm.name.as_str() == m && mm.params.len() == argc)
            }).unwrap_or(false)
        });
        if !ok {
            let names: Vec<String> = bounds.iter().map(|b| b.as_str()).collect();
            let bound_desc = if names.len() == 1 {
                format!("bound `{}` has", names[0])
            } else {
                format!("bounds `{}` have", names.join(" + "))
            };
            return Err(Error::Hir(format!(
                "operator `{}` on generic parameter `{}`: {} no `{}` method (at {}:{})",
                op_display, p.as_str(), bound_desc, m, span.start_line, span.start_col
            )));
        }
        Ok(())
    }

    /// C1：泛型参数 `T` 上的方法调用按约束检查。
    pub(crate) fn check_param_method(
        &self,
        p: &Symbol,
        method: &Symbol,
        argc: usize,
        span: &Span,
        gp: &[(Symbol, Vec<Symbol>)],
    ) -> Result<()> {
        let bounds: Vec<Symbol> = gp.iter().find(|(n, _)| n == p).map(|(_, c)| c.clone()).unwrap_or_default();
        if bounds.is_empty() {
            return Err(Error::Hir(format!(
                "method `{}` on generic parameter `{}` requires a bound (e.g. `[{}: Iface]`) (at {}:{})",
                method.as_str(), p.as_str(), p.as_str(), span.start_line, span.start_col
            )));
        }
        let ok = bounds.iter().any(|iface| {
            self.interfaces.get(iface).or_else(|| {
                self.interfaces.get(&crate::hir::lower::strip_generic_name(iface))
            }).map(|r| {
                r.methods.iter().any(|m| m.name == *method && m.params.len() == argc)
            }).unwrap_or(false)
        });
        if !ok {
            let names: Vec<String> = bounds.iter().map(|b| b.as_str()).collect();
            let bound_desc = if names.len() == 1 {
                format!("bound `{}`", names[0])
            } else {
                format!("bounds `{}`", names.join(" + "))
            };
            return Err(Error::Hir(format!(
                "generic parameter `{}` ({}) has no method `{}` for {} argument(s) (at {}:{})",
                p.as_str(), bound_desc, method.as_str(), argc, span.start_line, span.start_col
            )));
        }
        Ok(())
    }
}
