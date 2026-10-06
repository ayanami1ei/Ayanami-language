//! Phase 1.3（atb.1）：模式条件与绑定降级。
use super::*;
use crate::parser::ast::pattern::Pattern;

impl crate::hir::lower::Ctx {
    /// 变体在枚举中的 tag 序号（`_data_<name>` 字段位置 - 1）
    pub(super) fn variant_tag(&self, enum_name: &Symbol, variant: &Symbol) -> Option<i64> {
        let target = format!("_data_{}", variant);
        self.struct_defs.get(enum_name).and_then(|fields| {
            fields.iter().position(|f| f.name.as_str() == target).map(|i| i as i64 - 1)
        })
    }

    /// 模式条件：不可反驳返回 None；或模式为各分支 OR
    pub(super) fn pattern_cond(
        &mut self,
        pat: &Pattern,
        val: &HirNodeBox,
        val_ty: &HirType,
        span: &Span,
    ) -> Result<Option<HirNodeBox>> {
        let base = strip_ownership(val_ty.clone());
        match pat {
            Pattern::Wildcard | Pattern::Binding(_) => Ok(None),
            Pattern::Literal(lit) => {
                let lit = match (&base, lit) {
                    (HirType::Float | HirType::F32, crate::parser::ast::Literal::Int(i, _)) =>
                        crate::parser::ast::Literal::Float(*i as f64, span_of(lit)),
                    _ => lit.clone(),
                };
                if !matches!(base, HirType::Int | HirType::IntN { .. } | HirType::Char
                    | HirType::Bool | HirType::Float | HirType::F32)
                {
                    return Err(Error::Hir(format!(
                        "literal pattern cannot match `{}` (at {}:{})",
                        hir_type_display(&base), span.start_line, span.start_col
                    )));
                }
                let c = SConst { val: hir_literal(&lit), ty: base.clone() }.into();
                Ok(Some(SBin {
                    op: crate::parser::ast::BinaryOp::Eq,
                    lhs: val.clone(),
                    rhs: c,
                    ty: base,
                }.into()))
            }
            Pattern::Enum { name, .. } => {
                let HirType::Named(en) = &base else {
                    return Err(Error::Hir(format!(
                        "enum pattern `{}` cannot match `{}` (at {}:{})",
                        name.as_str(), hir_type_display(&base), span.start_line, span.start_col
                    )));
                };
                let tag = self.variant_tag(en, name).ok_or_else(|| Error::Hir(format!(
                    "unknown variant `{}` of enum `{}` (at {}:{})",
                    name.as_str(), en.as_str(), span.start_line, span.start_col
                )))?;
                Ok(Some(Self::tag_cond(val, tag as usize)))
            }
            Pattern::Or(ps) => {
                let mut conds = Vec::new();
                for p in ps {
                    match self.pattern_cond(p, val, val_ty, span)? {
                        Some(c) => conds.push(c),
                        None => return Ok(None), // 含不可反驳分支 → 整体不可反驳
                    }
                }
                let mut it = conds.into_iter();
                let mut acc = it.next().ok_or_else(|| Error::Hir(format!(
                    "empty or-pattern (at {}:{})", span.start_line, span.start_col
                )))?;
                for c in it {
                    acc = SBin {
                        op: crate::parser::ast::BinaryOp::Or,
                        lhs: acc,
                        rhs: c,
                        ty: HirType::Bool,
                    }.into();
                }
                Ok(Some(acc))
            }
        }
    }

    /// 模式绑定赋值（在臂作用域内调用；`Or` 各分支绑定名须一致）
    pub(super) fn pattern_bindings(
        &mut self,
        pat: &Pattern,
        val: &HirNodeBox,
        val_ty: &HirType,
        span: &Span,
    ) -> Result<Vec<HirStmt>> {
        let base = strip_ownership(val_ty.clone());
        match pat {
            Pattern::Wildcard | Pattern::Literal(_) => Ok(Vec::new()),
            Pattern::Binding(n) => {
                // 整值绑定：仅 Copy 类型（拥有类型会与 scrutinee 双重释放）
                if !base.is_copy() {
                    return Err(Error::Hir(format!(
                        "binding the whole `{}` value is not supported yet; use `_` or destructure (at {}:{})",
                        hir_type_display(&base), span.start_line, span.start_col
                    )));
                }
                let (bid, _, _) = self.register_or_lookup(*n, base.clone());
                Ok(vec![HirStmt::Assign {
                    target: SVar { var: bid, ty: base }.into(),
                    value: val.clone(),
                    span: *span,
                }])
            }
            Pattern::Enum { name, bindings } => {
                let HirType::Named(en) = &base else { return Ok(Vec::new()) };
                let tag = self.variant_tag(en, name).ok_or_else(|| Error::Hir(format!(
                    "unknown variant `{}` (at {}:{})", name.as_str(), span.start_line, span.start_col
                )))? as usize;
                let data_field = Symbol::intern(&format!("_data_{}", name));
                let var_struct = crate::hir::lower::variant_struct_name(en, name);
                let mut stmts = Vec::new();
                for (j, bind_name) in bindings.iter().enumerate() {
                    if bind_name.as_str() == "_" { continue; }
                    let inner_acc: HirNodeBox = SField {
                        object: val.clone(),
                        field: data_field,
                        field_index: tag + 1,
                        ty: HirType::Named(var_struct),
                    }.into();
                    let payload_field = Symbol::intern(&format!("_{}", j));
                    let fty = self.find_field_type(&HirType::Named(var_struct), &payload_field, span)?;
                    let fval: HirNodeBox = SField {
                        object: inner_acc,
                        field: payload_field,
                        field_index: j,
                        ty: fty.clone(),
                    }.into();
                    let (bid, _, _) = self.register_or_lookup(*bind_name, fty.clone());
                    stmts.push(HirStmt::Assign {
                        target: SVar { var: bid, ty: fty }.into(),
                        value: fval,
                        span: *span,
                    });
                }
                Ok(stmts)
            }
            Pattern::Or(ps) => {
                // 各分支绑定名须一致
                let first: Vec<Symbol> = ps.first().map(|p| p.bindings()).unwrap_or_default();
                for p in ps.iter().skip(1) {
                    if p.bindings() != first {
                        return Err(Error::Hir(format!(
                            "or-pattern branches must bind the same names (at {}:{})",
                            span.start_line, span.start_col
                        )));
                    }
                }
                for p in ps {
                    let s = self.pattern_bindings(p, val, val_ty, span)?;
                    if !s.is_empty() {
                        return Ok(s);
                    }
                }
                Ok(Vec::new())
            }
        }
    }

    pub(super) fn tag_cond(val_node: &HirNodeBox, tag: usize) -> HirNodeBox {
        SBin {
            op: crate::parser::ast::BinaryOp::Eq,
            lhs: SField {
                object: val_node.clone(),
                field: Symbol::intern("_tag"),
                field_index: 0,
                ty: HirType::Int,
            }.into(),
            rhs: SConst { val: HirLiteral::Int(tag as i64), ty: HirType::Int }.into(),
            ty: HirType::Int,
        }.into()
    }
}

fn span_of(l: &crate::parser::ast::Literal) -> Span {
    use crate::parser::ast::Literal;
    match l {
        Literal::Int(_, s) | Literal::Float(_, s) | Literal::Char(_, s)
        | Literal::String(_, s) | Literal::Bool(_, s) => *s,
    }
}

/// AST 字面量 → HIR 字面量
fn hir_literal(l: &crate::parser::ast::Literal) -> HirLiteral {
    use crate::parser::ast::Literal;
    match l {
        Literal::Int(i, _) => HirLiteral::Int(*i),
        Literal::Float(f, _) => HirLiteral::Float(*f),
        Literal::Char(c, _) => HirLiteral::Char(*c),
        Literal::Bool(b, _) => HirLiteral::Bool(*b),
        Literal::String(s, _) => HirLiteral::String(s.clone()),
    }
}

/// match/if 分支公共结果类型（数值提升；其余取首个分支类型）
pub(super) fn match_result_type(a: &HirType, b: &HirType) -> HirType {
    let sa = strip_ownership(a.clone());
    let sb = strip_ownership(b.clone());
    // M1.9：`!` 是单位元（发散分支不影响公共类型）
    if sa == HirType::Never { return sb; }
    if sb == HirType::Never { return sa; }
    if sa == sb { return sa; }
    match (&sa, &sb) {
        (HirType::Float, HirType::Int) | (HirType::Int, HirType::Float)
        | (HirType::Float, HirType::Char) | (HirType::Char, HirType::Float) => HirType::Float,
        (HirType::Char, HirType::Int) | (HirType::Int, HirType::Char) => HirType::Int,
        _ => sa,
    }
}

/// M1.9：块是否发散（末尾为 `return` 或 `!` 类型表达式语句）
pub(super) fn block_diverges(b: &HirBlock) -> bool {
    match b.stmts.last() {
        Some(HirStmt::Return { .. }) => true,
        Some(HirStmt::Expr { expr, .. }) => matches!(strip_ownership(expr_type(expr)), HirType::Never),
        _ => false,
    }
}

