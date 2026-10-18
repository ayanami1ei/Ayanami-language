//! 结构体字面量降级：显式/推断泛型实参 + 单态化 + 字段类型转换。
use super::*;

impl crate::hir::lower::Ctx {
    pub(crate) fn lower_struct_literal(&mut self, type_name: &Symbol, generic_args: &Vec<Type>, fields: &Vec<(Symbol, Expr)>, span: &Span) -> Result<HirNodeBox> {
        // 字段先降级（原始值）；字段类型转换在具体名（concrete_name）确定后做。
        // #153：字面量字段必须存在于结构体定义（否则 codegen GEP 越界）
        let base_fields = self.struct_defs.get(type_name).cloned();
        let mut seen: std::collections::HashSet<Symbol> = std::collections::HashSet::new();
        let mut raw_fields: Vec<(Symbol, HirNodeBox, Span)> = Vec::new();
        for (name, expr) in fields {
            let sp = expr.span();
            if let Some(defs) = &base_fields {
                if !defs.iter().any(|f| f.name == *name) {
                    return Err(Error::Hir(format!(
                        "struct `{}` has no field `{}` (at {}:{})",
                        type_name.as_str(), name.as_str(), sp.start_line, sp.start_col
                    )));
                }
            }
            if !seen.insert(*name) {
                return Err(Error::Hir(format!(
                    "duplicate field `{}` in struct literal (at {}:{})",
                    name.as_str(), sp.start_line, sp.start_col
                )));
            }
            raw_fields.push((*name, self.lower_expr(expr)?, sp));
        }
        // 无显式实参的泛型结构体字面量：从字段值推断泛型实参
        // （`Box { v = 42 }` → `Box<int>`；与枚举构造 `Option::Some(1)` 一致）。
        // 无法全部推断时回退基名，由后续使用点细化。
        let inferred_args: Vec<Type> = if generic_args.is_empty() {
            self.infer_struct_literal_args(type_name, &raw_fields)
        } else {
            Vec::new()
        };
        let effective_args: &Vec<Type> = if generic_args.is_empty() { &inferred_args } else { generic_args };
        // Handle generic struct instantiation
        let concrete_name = if !effective_args.is_empty() {
            let args_str: Vec<String> = effective_args.iter()
                .map(|a| type_to_string_generic(a, &self.interfaces))
                .collect();
            let encoded = format!("{}<{}>", type_name, args_str.join(","));
            let name_sym = Symbol::intern(&encoded);
            // Monomorphize: create concrete struct def if not exists
            if !self.struct_defs.contains_key(&name_sym) {
                if let Some(generic_fields) = self.struct_defs.get(type_name) {
                    // Build substitution map: T → concrete type
                    let mut generic_params = self.collected_generic_params(type_name);
                    // 若 generic_struct_params 未从 .lcl 合并，则从字段类型推断 GP 名称
                    if generic_params.is_empty() && !effective_args.is_empty() {
                        generic_params = effective_args.iter().enumerate()
                            .map(|(i, _)| (Symbol::intern(&format!("_G{}", i)), Vec::new()))
                            .collect();
                        // 尝试从字段类型中提取实际 GP 名称（单字母大写名）
                        if let Some(fields) = self.struct_defs.get(type_name) {
                            for field in fields {
                                if let HirType::Named(n) = strip_ownership_ref(&field.ty) {
                                    let s = n.as_str();
                                    if s.len() == 1 && s.chars().all(|c| c.is_uppercase()) {
                                        generic_params = vec![(*n, Vec::new())];
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    let mut subst: HashMap<Symbol, HirType> = HashMap::new();
                    for ((gp_name, _), concrete_ty) in generic_params.iter().zip(effective_args.iter()) {
                        let hir_ty = ast_type_to_hir(concrete_ty, &self.interfaces);
                        subst.insert(*gp_name, hir_ty);
                    }
                    // Substitute field types
                    let concrete_fields: Vec<HirStructField> = generic_fields.iter()
                        .map(|f| {
                            let new_ty = substitute_hir_type(&f.ty, &subst);
                            HirStructField { name: f.name, ty: new_ty }
                        })
                        .collect();
                    self.struct_defs.insert(name_sym, concrete_fields);
                }
            }
            name_sym
        } else {
            *type_name
        };
        let struct_ty = HirType::Named(concrete_name);
        let struct_fields = self.struct_defs.get(&concrete_name).cloned();
        let field_tys: HashMap<Symbol, HirType> = struct_fields.as_ref()
            .map(|fields| fields.iter().map(|f| (f.name, f.ty.clone())).collect())
            .unwrap_or_default();
        let mut hir_fields = Vec::new();
        for (name, mut hir_val, sp) in raw_fields {
            if struct_fields.is_none() {
                return Err(Error::Hir(format!(
                    "unknown struct `{}` in struct literal (at {}:{})",
                    concrete_name.as_str(), sp.start_line, sp.start_col
                )));
            }
            // Coerce null literal to the correct pointer type
            if let Some(HirLiteral::Int(0)) = hir_val.as_const() {
                if let Some(field_ty) = field_tys.get(&name) {
                    if matches!(field_ty, HirType::Unique(_)) {
                        hir_val = SConst { val: HirLiteral::Int(0), ty: field_ty.clone() }.into();
                    }
                }
            }
            // 字段类型转换：接口字段装箱（#161）+ 隐式数值转换
            if let Some(field_ty) = field_tys.get(&name) {
                hir_val = match self.coerce_iface_value(hir_val.clone(), field_ty, &sp)? {
                    Some(converted) => converted,
                    None => coerce_expr(hir_val, field_ty, &sp)?,
                };
            }
            hir_fields.push((name, implicit_move(hir_val)));
        }
        if let Some(defs) = &struct_fields {
            let is_union = defs.first().map(|f| f.name.as_str() == "__union").unwrap_or(false);
            if is_union {
                // P3b：union 字面量恰好初始化一个字段（其余字段是同一存储的别名）
                if hir_fields.len() != 1 {
                    return Err(Error::Hir(format!(
                        "union `{}` literal must initialize exactly one field (at {}:{})",
                        concrete_name.as_str(), span.start_line, span.start_col
                    )));
                }
            } else {
                // #153 后续：缺少字段同样静默（未初始化）；要求字面量覆盖全部字段
                let missing: Vec<String> = defs.iter()
                    .filter(|f| !seen.contains(&f.name))
                    .map(|f| format!("`{}`", f.name.as_str()))
                    .collect();
                if !missing.is_empty() {
                    return Err(Error::Hir(format!(
                        "missing field(s) {} in struct literal `{}` (at {}:{})",
                        missing.join(", "), concrete_name.as_str(), span.start_line, span.start_col
                    )));
                }
                // 字段按声明顺序排序（LIR 按索引存字段；字面量顺序不得影响布局）
                hir_fields.sort_by_key(|(n, _)| {
                    defs.iter().position(|f| f.name == *n).unwrap_or(usize::MAX)
                });
            }
        }
        Ok(SStruct {
            type_name: concrete_name,
            fields: hir_fields,
            ty: struct_ty,
        }.into())
    }

    /// 无显式实参的结构体字面量：从字段值推断泛型实参。
    /// 仅绑定「字段声明类型恰为泛型参数、且值类型具体」的字段；
    /// 无法全部推断时返回空（回退基名，由后续使用点细化）。
    fn infer_struct_literal_args(&self, type_name: &Symbol, raw_fields: &[(Symbol, HirNodeBox, Span)]) -> Vec<Type> {
        let params = self.collected_generic_params(type_name);
        if params.is_empty() {
            return Vec::new();
        }
        let Some(defs) = self.struct_defs.get(type_name) else { return Vec::new() };
        let mut subst: HashMap<Symbol, HirType> = HashMap::new();
        for (name, node, _) in raw_fields {
            let Some(field) = defs.iter().find(|f| f.name == *name) else { continue };
            let val_ty = node.expr_type();
            if !self.is_concrete_literal_type(&val_ty) {
                continue;
            }
            // R1/P4c：统一结构匹配——`T` 直接绑定；`Ptr[T]`/`Box[T]` 等复合字段同样绑定
            // （此前复合字段留给使用点细化，字段访问路径无细化点 → 裸基名）
            let mut local: HashMap<Symbol, HirType> = HashMap::new();
            let _ = unify(&field.ty, &val_ty, &mut local);
            for (gp, ty) in local {
                if params.iter().any(|(p, _)| *p == gp) {
                    subst.entry(gp).or_insert(ty);
                }
            }
        }
        if subst.len() != params.len() {
            return Vec::new();
        }
        params.iter().filter_map(|(gp, _)| subst.get(gp).map(hir_type_to_ast_type)).collect()
    }

    /// 字面量字段值是否可作为泛型实参：具体类型（非裸泛型基名/未解参数）。
    fn is_concrete_literal_type(&self, ty: &HirType) -> bool {
        match strip_ownership_ref(ty) {
            HirType::Named(n) => {
                let s = n.as_str();
                if s.contains('<') {
                    return true;
                }
                // 裸泛型基名（`Box`）与单字母大写参数（`T`）不算具体
                !self.generic_struct_params.contains_key(n)
                    && !(s.len() == 1 && s.chars().all(|c| c.is_uppercase()))
            }
            HirType::Int | HirType::IntN { .. } | HirType::Char | HirType::Bool
            | HirType::Float | HirType::F32 | HirType::RawPtr(_) | HirType::FnPtr(..) => true,
            _ => false,
        }
    }
}
