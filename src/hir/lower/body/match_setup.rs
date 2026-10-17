//! `match` 目标求值：可寻址位置就地匹配，否则溢出到临时局部。
use super::*;

/// 可寻址位置：变量 / 字段 / 索引 / 解引用（ref 与 Ptr 均算）
fn is_match_place(node: &HirNodeBox) -> bool {
    node.as_local().is_some()
        || node.as_field_access().is_some()
        || node.as_index().is_some()
        || node.as_deref().is_some()
}

impl crate::hir::lower::Ctx {
    /// 求值 match 目标 → 位置或临时局部；返回 (值节点, 值类型)
    pub(super) fn setup_match_value(&mut self, value: &Expr, span: &Span) -> Result<(HirNodeBox, HirType)> {
        let lowered = self.lower_expr(value)?;
        let hir_value = auto_deref(lowered);
        let value_ty = expr_type(&hir_value);
        let base = strip_ownership(value_ty.clone());
        if !matches!(base,
            HirType::Named(_) | HirType::Int | HirType::IntN { .. } | HirType::Char
            | HirType::Bool | HirType::Float | HirType::F32)
        {
            return Err(Error::Hir(format!(
                "match on unsupported type `{}` at {}:{}",
                hir_type_display(&value_ty), span.start_line, span.start_col
            )));
        }
        // 可寻址位置（变量/字段/索引/解引用，含 ref 与 Ptr）：就地匹配——
        // 载荷移出直接作用于原位置（无 clone glue 时字段清零），避免浅拷贝
        // 临时副本（含 Ptr 复制）与原值各自 drop 造成双重释放
        if is_match_place(&hir_value) {
            return Ok((hir_value, value_ty));
        }
        let val_var = VarId(self.locals.len());
        self.locals.push(HirLocal::new(Symbol::intern("__match_val"), value_ty.clone(), false));
        let val_node: HirNodeBox = SVar { var: val_var, ty: value_ty.clone() }.into();
        self.pending_stmts.push(HirStmt::Assign {
            target: val_node.clone(),
            value: hir_value,
            span: *span,
        });
        Ok((val_node, value_ty))
    }
}
