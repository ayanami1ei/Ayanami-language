//! Phase 1.3：模式匹配降级（atb.1 字面量/`_`/`|`/guard；atb.2 嵌套/解构/range）。
//! 统一递归：`pattern_match(pat, expr, ty)` → (条件, 绑定赋值)；条件 None = 不可反驳。
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

    /// 统一模式匹配：返回 (条件, 绑定赋值)；条件 None = 不可反驳。
    pub(super) fn pattern_match(
        &mut self,
        pat: &Pattern,
        expr: &HirNodeBox,
        ty: &HirType,
        span: &Span,
    ) -> Result<(Option<HirNodeBox>, Vec<HirStmt>)> {
        self.pattern_match_in(pat, expr, ty, span, false)
    }

    /// `nested`：位于枚举载荷/结构体字段内（允许绑定拥有类型字段，随 scrutinee 移动）
    fn pattern_match_in(
        &mut self,
        pat: &Pattern,
        expr: &HirNodeBox,
        ty: &HirType,
        span: &Span,
        nested: bool,
    ) -> Result<(Option<HirNodeBox>, Vec<HirStmt>)> {
        let base = strip_ownership(ty.clone());
        // 裸标识符若匹配枚举变体名 → 变体模式（无载荷），支持嵌套 `Some(None)`
        if let Pattern::Binding(n) = pat {
            if let HirType::Named(en) = &base {
                if self.variant_tag(en, n).is_some() {
                    return self.pattern_match_in(&Pattern::Enum { name: *n, args: Vec::new() }, expr, ty, span, nested);
                }
            }
        }
        match pat {
            Pattern::Wildcard => Ok((None, Vec::new())),
            Pattern::Binding(n) => {
                // #150 后续：枚举 scrutinee 下大写裸标识符多半是变体拼错，给出明确诊断
                if let HirType::Named(en) = &base {
                    let looks_variant = n.as_str().chars().next().map_or(false, |c| c.is_uppercase());
                    if looks_variant && self.variant_tag(en, n).is_none() {
                        return Err(Error::Hir(format!(
                            "unknown variant `{}` of enum `{}`; use `_` for the catch-all arm (at {}:{})",
                            n.as_str(), en.as_str(), span.start_line, span.start_col
                        )));
                    }
                }
                if !nested && !base.is_copy() {
                    return Err(Error::Hir(format!(
                        "binding the whole `{}` value is not supported yet; use `_` or destructure (at {}:{})",
                        hir_type_display(&base), span.start_line, span.start_col
                    )));
                }
                let (bid, _, _) = self.register_or_lookup(*n, base.clone());
                Ok((None, vec![HirStmt::Assign {
                    target: SVar { var: bid, ty: base }.into(),
                    value: expr.clone(),
                    span: *span,
                }]))
            }
            Pattern::Literal(lit) => {
                super::match_util::check_scalar_pattern(&base, lit, false, span)?;
                Ok((Some(self.cmp_const(expr, lit, &base, crate::parser::ast::BinaryOp::Eq)), Vec::new()))
            }
            Pattern::Range { lo, hi, inclusive } => {
                super::match_util::check_scalar_pattern(&base, lo, true, span)?;
                super::match_util::check_scalar_pattern(&base, hi, true, span)?;
                let c1 = self.cmp_const(expr, lo, &base, crate::parser::ast::BinaryOp::Ge);
                let c2 = self.cmp_const(expr, hi, &base,
                    if *inclusive { crate::parser::ast::BinaryOp::Le } else { crate::parser::ast::BinaryOp::Lt });
                Ok((Some(and_cond(c1, c2)), Vec::new()))
            }
            Pattern::Enum { name, args } => {
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
                let mut cond = Self::tag_cond(expr, tag as usize);
                let mut binds = Vec::new();
                let var_struct = crate::hir::lower::variant_struct_name(en, name);
                // #150 后续：变体载荷个数必须与子模式个数一致（`A =>` 不能忽略载荷）
                let payload_count = self.struct_defs.get(&var_struct).map(|f| f.len()).unwrap_or(0);
                if args.len() != payload_count {
                    return Err(Error::Hir(format!(
                        "variant `{}::{}` expects {} sub-pattern(s), found {} (at {}:{})",
                        en.as_str(), name.as_str(), payload_count, args.len(),
                        span.start_line, span.start_col
                    )));
                }
                let data_field = Symbol::intern(&format!("_data_{}", name));
                for (j, sub) in args.iter().enumerate() {
                    if matches!(sub, Pattern::Wildcard) { continue; }
                    let payload_field = Symbol::intern(&format!("_{}", j));
                    let fty = self.find_field_type(&HirType::Named(var_struct), &payload_field, span)?;
                    let inner_acc: HirNodeBox = SField {
                        object: expr.clone(),
                        field: data_field,
                        field_index: tag as usize + 1,
                        ty: HirType::Named(var_struct),
                    }.into();
                    let fval: HirNodeBox = SField {
                        object: inner_acc,
                        field: payload_field,
                        field_index: j,
                        ty: fty.clone(),
                    }.into();
                    let (sc, sb) = self.pattern_match_in(sub, &fval, &fty, span, true)?;
                    if let Some(c) = sc { cond = and_cond(cond, c); }
                    binds.extend(sb);
                }
                Ok((Some(cond), binds))
            }
            Pattern::Struct { name, fields } => {
                let HirType::Named(sn) = &base else {
                    return Err(Error::Hir(format!(
                        "struct pattern `{}` cannot match `{}` (at {}:{})",
                        name.as_str(), hir_type_display(&base), span.start_line, span.start_col
                    )));
                };
                if sn != name {
                    return Err(Error::Hir(format!(
                        "struct pattern `{}` does not match `{}` (at {}:{})",
                        name.as_str(), sn.as_str(), span.start_line, span.start_col
                    )));
                }
                let def = self.struct_defs.get(sn).cloned().ok_or_else(|| Error::Hir(format!(
                    "unknown struct `{}` in pattern (at {}:{})",
                    name.as_str(), span.start_line, span.start_col
                )))?;
                let mut cond: Option<HirNodeBox> = None;
                let mut binds = Vec::new();
                for (fname, sub) in fields {
                    let idx = def.iter().position(|f| f.name == *fname).ok_or_else(|| Error::Hir(format!(
                        "struct `{}` has no field `{}` (at {}:{})",
                        name.as_str(), fname.as_str(), span.start_line, span.start_col
                    )))?;
                    let fty = def[idx].ty.clone();
                    let fval: HirNodeBox = SField {
                        object: expr.clone(),
                        field: *fname,
                        field_index: idx,
                        ty: fty.clone(),
                    }.into();
                    let (sc, sb) = self.pattern_match_in(sub, &fval, &fty, span, true)?;
                    if let Some(c) = sc {
                        cond = Some(match cond { Some(prev) => and_cond(prev, c), None => c });
                    }
                    binds.extend(sb);
                }
                Ok((cond, binds))
            }
            Pattern::Or(ps) => {
                // atb.3：逐分支保留条件与绑定；绑定按实际匹配的分支执行
                let mut alts: Vec<(HirNodeBox, Vec<HirStmt>)> = Vec::new();
                for p in ps {
                    let (c, b) = self.pattern_match_in(p, expr, ty, span, nested)?;
                    match c {
                        Some(c) => alts.push((c, b)),
                        // 含不可反驳分支 → 整体不可反驳（其后的分支不可达）
                        None => return Ok((None, b)),
                    }
                }
                if alts.is_empty() {
                    return Err(Error::Hir(format!(
                        "empty or-pattern (at {}:{})", span.start_line, span.start_col
                    )));
                }
                // 绑定一致性：各分支绑定的名字集合与类型必须一致（顺序无关）
                let bind_map = |binds: &Vec<HirStmt>| -> Vec<(Symbol, HirType)> {
                    binds.iter().filter_map(|s| match s {
                        HirStmt::Assign { target, .. } => target
                            .as_local()
                            .map(|v| (self.locals[v.0].name, target.expr_type())),
                        _ => None,
                    }).collect()
                };
                let first_map = bind_map(&alts[0].1);
                for (i, (_, b)) in alts.iter().enumerate().skip(1) {
                    let m = bind_map(b);
                    super::match_coverage::check_or_bindings(&first_map, &m, i + 1, span)?;
                }
                let mut it = alts.iter().map(|(c, _)| c.clone());
                let mut acc = it.next().unwrap();
                for c in it { acc = or_cond(acc, c); }
                if alts.iter().all(|(_, b)| b.is_empty()) {
                    return Ok((Some(acc), Vec::new()));
                }
                // 绑定：if c0 {b0} elif c1 {b1} ... else {b_last}
                let n = alts.len();
                let binds = if n == 1 {
                    alts[0].1.clone()
                } else {
                    let elifs: Vec<(HirNodeBox, HirBlock)> = alts[1..n - 1].iter()
                        .map(|(c, b)| (c.clone(), HirBlock::new(b.clone())))
                        .collect();
                    vec![HirStmt::If {
                        cond: alts[0].0.clone(),
                        then_block: HirBlock::new(alts[0].1.clone()),
                        elifs,
                        else_block: Some(HirBlock::new(alts[n - 1].1.clone())),
                        span: *span,
                    }]
                };
                Ok((Some(acc), binds))
            }
        }
    }

    /// `expr OP lit`（字面量按 scrutinee 类型重定型）
    fn cmp_const(
        &self,
        expr: &HirNodeBox,
        lit: &crate::parser::ast::Literal,
        ty: &HirType,
        op: crate::parser::ast::BinaryOp,
    ) -> HirNodeBox {
        SBin {
            op,
            lhs: expr.clone(),
            rhs: SConst { val: super::match_util::hir_literal(lit), ty: ty.clone() }.into(),
            ty: ty.clone(),
        }.into()
    }
}

fn and_cond(a: HirNodeBox, b: HirNodeBox) -> HirNodeBox {
    SBin { op: crate::parser::ast::BinaryOp::And, lhs: a, rhs: b, ty: HirType::Bool }.into()
}

fn or_cond(a: HirNodeBox, b: HirNodeBox) -> HirNodeBox {
    SBin { op: crate::parser::ast::BinaryOp::Or, lhs: a, rhs: b, ty: HirType::Bool }.into()
}
