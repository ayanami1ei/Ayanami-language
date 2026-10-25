//! Phase 1.3：变体 tag / 限定名归一 / `_tag` 条件（从 match_pattern.rs 拆出，保持文件 ≤300 行）。
use super::*;

impl crate::hir::lower::Ctx {
    /// 变体在枚举中的 tag 序号（`_data_<name>` 字段位置 - 1）
    pub(super) fn variant_tag(&self, enum_name: &Symbol, variant: &Symbol) -> Option<i64> {
        // N1：niche 形态无 `_tag`，tag 从元信息取（= `_data_*` 字段序）
        if let Some(info) = self.niche_info(enum_name) {
            let target = Symbol::intern(&format!("_data_{}", variant));
            if target == info.payload_field { return Some(info.payload_tag); }
            if target == info.empty_field { return Some(info.empty_tag); }
            return None;
        }
        let target = format!("_data_{}", variant);
        self.struct_defs.get(enum_name).and_then(|fields| {
            fields.iter().position(|f| f.name.as_str() == target).map(|i| i as i64 - 1)
        })
    }

    /// 限定名变体（`Option::Some`/`Option.Some`）→ 裸变体名；
    /// 仅当前缀等于 scrutinee 枚举基名且该变体存在时归一化（否则原样返回，走未知变体诊断）。
    pub(super) fn bare_variant_name(&self, scrutinee: &HirType, name: &Symbol) -> Symbol {
        let HirType::Named(en) = strip_ownership_ref(scrutinee) else { return *name };
        let base_name = crate::hir::lower::strip_generic_name(en);
        let Some(pos) = name.as_str().rfind('.') else { return *name };
        let (prefix, bare) = (&name.as_str()[..pos], &name.as_str()[pos + 1..]);
        let bare_sym = Symbol::intern(bare);
        if prefix == base_name.as_str() && self.variant_tag(&base_name, &bare_sym).is_some() {
            return bare_sym;
        }
        *name
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
