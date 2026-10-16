//! Ctx 的字段/泛型访问器（从 ctx.rs 拆出，保持文件 ≤300 行）。

use super::*;

impl Ctx {
    /// 查找结构体中某字段的索引位置
    pub fn find_field_index(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<usize> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Unique(inner) | HirType::Ref(inner, _) => {
                return self.find_field_index(inner, field, span);
            }
            _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col))),
        };
        self.find_field_index_by_name(&type_name, field, span)
    }

    /// 按类型名查找字段索引（含泛型回退与替换）
    fn find_field_index_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<usize> {
        if let Some(fields) = self.struct_defs.get(type_name) {
            return fields.iter().position(|f| f.name == *field)
                .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
        }
        let base = strip_generic_name(type_name);
        if base != *type_name {
            if let Some(fields) = self.struct_defs.get(&base) {
                // 从类型名中提取泛型替换 e.g. LinkedListNode<int> → T=int
                return fields.iter().position(|f| f.name == *field)
                    .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
            }
        }
        Err(Error::Hir(format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col)))
    }

    /// 查找结构体中某字段的类型
    pub fn find_field_type(&self, struct_ty: &HirType, field: &Symbol, span: &Span) -> Result<HirType> {
        let type_name = match struct_ty {
            HirType::Named(n) => *n,
            HirType::Unique(inner) | HirType::Ref(inner, _) => {
                let inner_name = match inner.as_ref() {
                    HirType::Named(n) => *n,
                    _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                        hir_type_display(struct_ty), field, span.start_line, span.start_col))),
                };
                inner_name
            }
            _ => return Err(Error::Hir(format!("类型 {} 没有字段 `{}` (位置 {}:{})",
                hir_type_display(struct_ty), field, span.start_line, span.start_col))),
        };
        self.find_field_type_by_name(&type_name, field, span)
    }

    /// 按类型名查找字段类型（含泛型回退与替换）
    fn find_field_type_by_name(&self, type_name: &Symbol, field: &Symbol, span: &Span) -> Result<HirType> {
        if let Some(fields) = self.struct_defs.get(type_name) {
            return fields.iter().find(|f| f.name == *field)
                .map(|f| f.ty.clone())
                .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
        }
        let base = strip_generic_name(type_name);
        if base != *type_name {
            if let Some(fields) = self.struct_defs.get(&base) {
                let subst = self.build_generic_subst(type_name, &base);
                return fields.iter().find(|f| f.name == *field)
                    .map(|f| substitute_hir_type(&f.ty, &subst))
                    .ok_or_else(|| Error::Hir(format!("结构体 `{}` 没有字段 `{}` (位置 {}:{})", type_name, field, span.start_line, span.start_col)));
            }
        }
        Err(Error::Hir(format!("未知结构体 `{}` (位置 {}:{})", type_name, span.start_line, span.start_col)))
    }

    /// A3d：取枚举 `_data_X` 变体结构体的首个载荷字段类型（处理泛型替换）。
    pub fn variant_payload_type(&self, enum_ty: &HirType, data_field: &Symbol, span: &Span) -> Result<HirType> {
        let var_ty = self.find_field_type(enum_ty, data_field, span)?;
        let var_name = match &var_ty {
            HirType::Named(n) => *n,
            _ => return Ok(var_ty),
        };
        let fields = self.struct_defs.get(&var_name)
            .ok_or_else(|| Error::Hir(format!("未知变体结构体 `{}` (位置 {}:{})", var_name, span.start_line, span.start_col)))?;
        let first = match fields.first() {
            Some(f) => f.ty.clone(),
            None => return Ok(HirType::Void),
        };
        let enum_name = match enum_ty {
            HirType::Named(n) => *n,
            _ => return Ok(first),
        };
        let base = strip_generic_name(&enum_name);
        let subst = self.build_generic_subst(&enum_name, &base);
        Ok(substitute_hir_type(&first, &subst))
    }

    /// 从完整类型名（含泛型参数）构建替换映射
    pub(crate) fn build_generic_subst(&self, type_name: &Symbol, base: &Symbol) -> HashMap<Symbol, HirType> {
        let mut subst = HashMap::new();
        let s = type_name.as_str();
        let b = base.as_str();
        if let Some(start) = s.find(['<', '[']) {
            if &s[..start] == b {
                if let Some(inner) = generic_inner(&s) {
                    let gp = self.collected_generic_params(base);
                    for ((gp_name, _), val_str) in gp.iter().zip(split_generic_args(inner).iter()) {
                        subst.insert(*gp_name, sig_str_to_hir(val_str.trim()));
                    }
                }
            }
        }
        subst
    }

    /// 获取结构体的泛型参数列表（如有）
    pub fn collected_generic_params(&self, type_name: &Symbol) -> Vec<(Symbol, Vec<Symbol>)> {
        self.generic_struct_params.get(type_name).cloned().unwrap_or_default()
    }

}
