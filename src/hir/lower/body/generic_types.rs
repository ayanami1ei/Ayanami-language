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

impl crate::hir::lower::Ctx {
    /// AST 类型 → 浅层 `GType`。
    pub(crate) fn ast_gtype(&self, ty: &Type, gp: &[(Symbol, Option<Symbol>)]) -> GType {
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
        gp: &[(Symbol, Option<Symbol>)],
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

    /// C1：泛型参数 `T` 上的方法调用按约束检查。
    pub(crate) fn check_param_method(
        &self,
        p: &Symbol,
        method: &Symbol,
        argc: usize,
        span: &Span,
        gp: &[(Symbol, Option<Symbol>)],
    ) -> Result<()> {
        let bound = gp.iter().find(|(n, _)| n == p).and_then(|(_, c)| *c);
        match bound {
            Some(iface) => {
                let reg = self.interfaces.get(&iface).or_else(|| {
                    let base = crate::hir::lower::strip_generic_name(&iface);
                    self.interfaces.get(&base)
                });
                let ok = reg.map(|r| {
                    r.methods.iter().any(|m| m.name == *method && m.params.len() == argc)
                }).unwrap_or(false);
                if !ok {
                    return Err(Error::Hir(format!(
                        "generic parameter `{}` (bound `{}`) has no method `{}` for {} argument(s) (at {}:{})",
                        p.as_str(), iface.as_str(), method.as_str(), argc, span.start_line, span.start_col
                    )));
                }
                Ok(())
            }
            None => Err(Error::Hir(format!(
                "method `{}` on generic parameter `{}` requires a bound (e.g. `[{}: Iface]`) (at {}:{})",
                method.as_str(), p.as_str(), p.as_str(), span.start_line, span.start_col
            ))),
        }
    }
}
