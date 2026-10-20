//! N1（docs/enum-layout.md §3.1）：niche 枚举的 Ctx 侧判定、改写与 `_tag` 特殊化。
use super::*;

impl Ctx {
    /// niche 元信息（仅“已 niche 形态”：无 `_tag` 的单指针槽表示）。
    pub(crate) fn niche_info(&self, name: &Symbol) -> Option<crate::hir::niche::NicheInfo> {
        let fields = self.struct_defs.get(name)?;
        crate::hir::niche::niche_info_of(fields, &self.struct_defs)
    }

    /// 枚举是否 niche 形态。
    pub(crate) fn is_niche_enum(&self, name: &Symbol) -> bool {
        self.niche_info(name).is_some()
    }

    /// N2（docs/enum-layout.md §3.2）：载荷结构 > 16 字节的变体 → 字段改为 `Unique(E_V)`
    /// （堆装箱；构造分配 / 匹配经指针字段访问 / drop 自动释放）。
    pub(crate) fn apply_boxing_rewrite(
        &mut self,
        fields: Vec<HirStructField>,
        generic: bool,
    ) -> Vec<HirStructField> {
        // 声明期泛型枚举：载荷为未实例化类型，布局无意义（具体实例化时再装箱）
        if generic || fields.first().map(|f| f.name.as_str() != "_tag").unwrap_or(true) {
            return fields;
        }
        let defs: crate::hir::layout::StructDefs = self.struct_defs.iter()
            .map(|(k, fs)| (*k, fs.iter().map(|f| (f.name, f.ty.clone())).collect()))
            .collect();
        fields.into_iter().map(|f| {
            if !f.name.as_str().starts_with("_data_") {
                return f;
            }
            let HirType::Named(vn) = &f.ty else { return f; };
            if !self.struct_defs.contains_key(vn) {
                return f;
            }
            // 保守：变体载荷引用“未实例化的泛型基类型”（推断期临时实例化，如 Opt<Opt>）
            // 时跳过装箱——其布局与真实实例化不一致，装箱会破坏 ABI 一致性。
            if let Some(vfields) = self.struct_defs.get(vn) {
                let spurious = vfields.iter().any(|vf| match &vf.ty {
                    HirType::Named(n) => self.generic_struct_params.get(n)
                        .map(|g| !g.is_empty()).unwrap_or(false),
                    _ => false,
                });
                if spurious { return f; }
            }
            let (size, _) = crate::hir::layout::type_layout(&f.ty, &defs);
            if size > 16 {
                HirStructField { name: f.name, ty: HirType::Unique(Box::new(HirType::Named(*vn))) }
            } else {
                f
            }
        }).collect()
    }

    /// 声明/实例化时应用 niche 改写：tag 形态且载荷可 niche → 去掉 `_tag` 字段
    /// （字段序保持：`_data_*` 的下标即 tag；空变体 = 载荷槽零值 = null）。
    pub(crate) fn apply_niche_rewrite(&mut self, fields: Vec<HirStructField>) -> Vec<HirStructField> {
        if crate::hir::niche::niche_shape(&fields, &self.struct_defs).is_some() {
            return fields.into_iter()
                .filter(|f| f.name.as_str() != "_tag")
                .collect();
        }
        fields
    }

    /// N1：niche 枚举 `_tag` 读取 → 空值判定（非空 → payload_tag；空 → empty_tag）。
    /// 表达式形态：`(reinterpret<ptr→int>(v._data_P._0) == 0) as int`（payload_tag == 0 时；
    /// 否则 `!= 0`）——沿用比较表达式保留操作数类型 + `SBoolToNum` 的既有转换口径。
    pub(crate) fn build_niche_tag_expr(
        &mut self,
        object: HirNodeBox,
        enum_ty: &HirType,
        span: &Span,
    ) -> Result<HirNodeBox> {
        let HirType::Named(name) = strip_ownership_ref(enum_ty) else {
            return Err(Error::Hir(format!(
                "niche `_tag`: 非枚举类型 `{}` (位置 {}:{})",
                hir_type_display(enum_ty), span.start_line, span.start_col
            )));
        };
        let info = self.niche_info(&name).ok_or_else(|| Error::Hir(format!(
            "niche `_tag`: `{}` 非 niche 枚举 (位置 {}:{})",
            name.as_str(), span.start_line, span.start_col
        )))?;
        let data_ty = self.find_field_type(enum_ty, &info.payload_field, span)?;
        let data_index = self.find_field_index(enum_ty, &info.payload_field, span)?;
        let ptr_ty = self.variant_payload_type(enum_ty, &info.payload_field, span)?;
        let inner: HirNodeBox = SField {
            object,
            field: info.payload_field,
            field_index: data_index,
            ty: data_ty.clone(),
        }.into();
        // N1：变体结构体内的 `_0` 即指针；N2：boxed 变体的字段本身即指针
        let ptr_expr: HirNodeBox = if matches!(&data_ty, HirType::Named(_)) {
            SField {
                object: inner,
                field: Symbol::intern("_0"),
                field_index: 0,
                ty: ptr_ty,
            }.into()
        } else {
            inner
        };
        let bits: HirNodeBox = SReinterpret { expr: ptr_expr, ty: HirType::Int }.into();
        let op = if info.payload_tag == 0 {
            crate::parser::ast::BinaryOp::Eq
        } else {
            crate::parser::ast::BinaryOp::Neq
        };
        let cmp: HirNodeBox = SBin {
            op,
            lhs: bits,
            rhs: SConst { val: HirLiteral::Int(0), ty: HirType::Int }.into(),
            ty: HirType::Int,
        }.into();
        Ok(SBoolToNum { expr: cmp, ty: HirType::Int }.into())
    }
}
