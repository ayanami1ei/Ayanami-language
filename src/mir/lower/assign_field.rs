//! 字段赋值降级：旧字段释放（含动态数组按兄弟字段计数逐元素释放）。
use super::*;
use super::mem::needs_drop;

impl Ctx {
    pub(super) fn lower_field_assign(
        &mut self,
        object: &HirNodeBox,
        field: Symbol,
        field_index: usize,
        field_ty: &HirType,
        value: &HirNodeBox,
        span: crate::span::Span,
    ) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();
        let mut post = Vec::new();
        // 旧字段需要释放时，RHS 先求值到临时变量（drop 必须晚于 RHS 求值）
        let drop_old = object.as_local()
            .map(|v| needs_drop(field_ty, &self.struct_defs) && !self.moved.contains(&v))
            .unwrap_or(false);
        let tmp = if !drop_old || matches!(value.expr_type(), HirType::Never) {
            None
        } else {
            Some(self.new_temp(field_ty.clone()))
        };
        if let Some(tmp_var) = tmp {
            let mut mir_value = value.lower_to_mir(&self.moved);
            let (pre, po) = self.hoist_ref_temps(&mut mir_value, span);
            stmts.extend(pre);
            stmts.push(SMirAssignStmt {
                target: SMirLocal { var: tmp_var, ty: self.var_types[&tmp_var].clone(), moved: false }.into(),
                value: mir_value,
                span,
            }.into());
            post.extend(po);
        }
        // 旧字段拥有堆数据时移出并释放（覆盖不泄漏）；仅局部对象（避免重复求值）
        if drop_old {
            let old = self.new_temp(field_ty.clone());
            let field_expr: MirNodeBox = SMirFieldAccess {
                object: object.lower_to_mir(&self.moved),
                field,
                field_index,
                ty: field_ty.clone(),
            }.into();
            stmts.push(SMirAssignStmt {
                target: SMirLocal { var: old, ty: field_ty.clone(), moved: false }.into(),
                value: SMirMove { expr: field_expr, ty: field_ty.clone() }.into(),
                span,
            }.into());
            // 动态 [T] 旧字段：从结构体兄弟字段取计数，逐元素释放（否则元素泄漏）
            let counted = match field_ty {
                HirType::Unique(inner) if matches!(inner.as_ref(), HirType::Array(_)) => {
                    let elem_ty = match inner.as_ref() {
                        HirType::Array(e) => (**e).clone(),
                        _ => HirType::Void,
                    };
                    if needs_drop(&elem_ty, &self.struct_defs) {
                        object_struct_name(&object.expr_type())
                            .and_then(|sn| self.struct_defs.get(&sn).cloned())
                            .and_then(|fields| count_field_idx(&fields, field_index))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some((ci, cty, fields)) = counted {
                let count_tmp = self.new_temp(cty.clone());
                let cname = fields[ci].0;
                let cexpr: MirNodeBox = SMirFieldAccess {
                    object: object.lower_to_mir(&self.moved),
                    field: cname,
                    field_index: ci,
                    ty: cty,
                }.into();
                stmts.push(SMirAssignStmt {
                    target: SMirLocal { var: count_tmp, ty: self.var_types[&count_tmp].clone(), moved: false }.into(),
                    value: cexpr,
                    span,
                }.into());
                let elem_ty = match field_ty {
                    HirType::Unique(inner) => match inner.as_ref() {
                        HirType::Array(e) => (**e).clone(),
                        _ => HirType::Void,
                    },
                    _ => HirType::Void,
                };
                stmts.push(SMirDropCounted { var: old, elem_ty, count_var: count_tmp, span }.into());
            } else {
                stmts.push(SMirDropStmt { var: old, ty: field_ty.clone(), span }.into());
            }
        }
        let mir_value: MirNodeBox = match tmp {
            Some(tmp_var) => SMirLocal { var: tmp_var, ty: self.var_types[&tmp_var].clone(), moved: false }.into(),
            None => {
                let mut mir_value = value.lower_to_mir(&self.moved);
                let (pre, po) = self.hoist_ref_temps(&mut mir_value, span);
                stmts.extend(pre);
                post.extend(po);
                mir_value
            }
        };
        stmts.push(SMirFieldAssignStmt {
            object: object.lower_to_mir(&self.moved),
            field,
            field_index,
            field_ty: field_ty.clone(),
            value: mir_value,
            span,
        }.into());
        stmts.extend(post);
        stmts
    }
}

/// 取拥有/引用包装内的命名结构体名。
fn object_struct_name(ty: &HirType) -> Option<Symbol> {
    let base = match ty {
        HirType::Ref(i, _) | HirType::Unique(i) => i.as_ref(),
        other => other,
    };
    match base {
        HirType::Named(n) => Some(*n),
        _ => None,
    }
}

/// 动态数组字段的计数兄弟字段：容量优先，其次长度；返回 (下标, 类型, 字段表)。
fn count_field_idx(
    fields: &[(Symbol, HirType)],
    array_idx: usize,
) -> Option<(usize, HirType, Vec<(Symbol, HirType)>)> {
    let is_int = |t: &HirType| matches!(t, HirType::Int | HirType::IntN { bits: 64, .. });
    let by_names = |names: &[&str]| -> Option<usize> {
        fields.iter().enumerate().find(|(i, (n, t))| {
            *i != array_idx && is_int(t) && names.iter().any(|x| n.as_str() == *x)
        }).map(|(i, _)| i)
    };
    let idx = by_names(&["capability", "capacity", "cap"])
        .or_else(|| by_names(&["len", "length", "size", "count"]))
        .or_else(|| {
            let after = array_idx + 1;
            (after < fields.len() && is_int(&fields[after].1)).then_some(after)
        })?;
    Some((idx, fields[idx].1.clone(), fields.to_vec()))
}
