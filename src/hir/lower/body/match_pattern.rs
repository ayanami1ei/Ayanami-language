//! Phase 1.3：模式匹配降级（atb.1 字面量/`_`/`|`/guard；atb.2 嵌套/解构/range）。
//! 统一递归：`pattern_match(pat, expr, ty)` → (条件, 绑定赋值)；条件 None = 不可反驳。
use super::*;
use crate::parser::ast::pattern::Pattern;

impl crate::hir::lower::Ctx {
    /// 统一模式匹配：返回 (条件, 绑定赋值)；条件 None = 不可反驳。
    pub(super) fn pattern_match(
        &mut self,
        pat: &Pattern,
        expr: &HirNodeBox,
        ty: &HirType,
        span: &Span,
    ) -> Result<(Option<HirNodeBox>, Vec<HirStmt>)> {
        let mode = match strip_ownership(ty.clone()) {
            HirType::Ref(_, mutable) => Some(mutable),
            _ => None,
        };
        let saved = self.pattern_by_ref;
        self.pattern_by_ref = mode;
        let r = self.pattern_match_in(pat, expr, ty, span, false);
        self.pattern_by_ref = saved;
        r
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
        // 裸标识符/限定名若匹配枚举变体名 → 变体模式（无载荷），支持嵌套 `Some(None)`、`Option::None`
        if let Pattern::Binding(n) = pat {
            if let HirType::Named(en) = &base {
                let bare = self.bare_variant_name(&base, n);
                if self.variant_tag(en, &bare).is_some() {
                    return self.pattern_match_in(&Pattern::Enum { name: bare, args: Vec::new() }, expr, ty, span, nested);
                }
            }
        }
        match pat {
            Pattern::Wildcard => Ok((None, Vec::new())),
            Pattern::Binding(n) => {
                // 点号名未能归一化为变体（前缀不匹配/变体不存在）→ 明确诊断
                if n.as_str().contains('.') {
                    return Err(Error::Hir(format!(
                        "unknown variant `{}` of enum `{}` (at {}:{})",
                        n.as_str().replace('.', "::"), hir_type_display(&base),
                        span.start_line, span.start_col
                    )));
                }
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
                // match ergonomics：引用 scrutinee 下按引用绑定（Rust 默认绑定模式）；
                // scrutinee 可能已被 auto-deref，按位置的根局部是否为 ref 兜底判断。
                let via_root = super::match_util::place_root_hir(&**expr).and_then(|id| {
                    match self.locals.get(id.0).map(|l| l.ty.clone()) {
                        Some(HirType::Ref(_, m)) => Some(m),
                        _ => None,
                    }
                });
                if let Some(mutable) = self.pattern_by_ref.or(via_root) {
                    let (bind_ty, value): (HirType, HirNodeBox) = if nested {
                        // Rust match ergonomics：绑定 `ref Ptr[T]`（不能剥掉 Unique 层）
                        let rt = HirType::Ref(Box::new(ty.clone()), mutable);
                        (rt.clone(), SRef { expr: expr.clone(), mutable, ty: rt }.into())
                    } else {
                        (ty.clone(), expr.clone())
                    };
                    let (bid, _, _) = self.register_or_lookup(*n, bind_ty.clone());
                    return Ok((None, vec![HirStmt::Assign {
                        target: SVar { var: bid, ty: bind_ty }.into(),
                        value,
                        span: *span,
                    }]));
                }
                // 绑定变量保留源类型：`Ptr[T]`（`Unique`）是拥有指针值，
                // 不能剥成 pointee——否则 `Some(p)` / `Struct { p }` 的载荷绑定类型错误
                let bind_ty = ty.clone();
                if !nested && !bind_ty.is_copy() {
                    return Err(Error::Hir(format!(
                        "binding the whole `{}` value is not supported yet; use `_` or destructure (at {}:{})",
                        hir_type_display(&bind_ty), span.start_line, span.start_col
                    )));
                }
                let (bid, _, _) = self.register_or_lookup(*n, bind_ty.clone());
                // 拥有类型载荷/字段绑定按移动处理：源字段清零（SLirFieldTake），
                // 源枚举/结构体 drop 不再重复释放
                Ok((None, vec![HirStmt::Assign {
                    target: SVar { var: bid, ty: bind_ty }.into(),
                    value: implicit_move(expr.clone()),
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
                // 限定名变体模式（`Option::Some(x)`）归一化为裸变体名
                let name = &self.bare_variant_name(&base, name);
                let HirType::Named(en) = &base else {
                    return Err(Error::Hir(format!(
                        "enum pattern `{}` cannot match `{}` (at {}:{})",
                        name.as_str(), hir_type_display(&base), span.start_line, span.start_col
                    )));
                };
                let tag = self.variant_tag(en, name).ok_or_else(|| Error::Hir(format!(
                    "unknown variant `{}` of enum `{}` (at {}:{})",
                    name.as_str().replace('.', "::"), en.as_str(), span.start_line, span.start_col
                )))?;
                // N1：niche 形态 → 空值判定（tag 表达式 == tag）
                let mut cond = if self.is_niche_enum(en) {
                    let tag_expr = self.build_niche_tag_expr(
                        expr.clone(), &HirType::Named(*en), span)?;
                    SBin {
                        op: crate::parser::ast::BinaryOp::Eq,
                        lhs: tag_expr,
                        rhs: SConst { val: HirLiteral::Int(tag), ty: HirType::Int }.into(),
                        ty: HirType::Int,
                    }.into()
                } else {
                    Self::tag_cond(expr, tag as usize)
                };
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
                    // N1：niche 形态无 `_tag`，下标按实际字段表取（tag 形态 = tag + 1）
                    let data_index = self.find_field_index(&HirType::Named(*en), &data_field, span)?;
                    // N2：boxed 变体的字段类型为 Unique(E_V)（后续 `_0` 访问经指针）
                    let data_ty = self.find_field_type(&HirType::Named(*en), &data_field, span)?;
                    let inner_acc: HirNodeBox = SField {
                        object: expr.clone(),
                        field: data_field,
                        field_index: data_index,
                        ty: data_ty,
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
