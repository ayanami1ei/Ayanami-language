//! 赋值语句降级：旧值/旧字段释放顺序（drop 晚于 RHS 求值）、借用临时量提升。
use super::*;
use super::mem::{action_to_stmt, needs_drop, strategy_for};

impl Ctx {
    pub(super) fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();
        if self.alive.contains(var) && !self.moved.contains(var) && !self.result_vars.contains(var) {
            let ty = &self.var_types[var];
            let strategy = strategy_for(ty, &self.struct_defs);
            for action in strategy.on_assign_overwrite(*var, ty) {
                stmts.push(action_to_stmt(*var, ty, &action));
            }
        }
        stmts
    }

    pub(super) fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox, span: crate::span::Span) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();
        let mut post = Vec::new();

        value.record_moves(&mut self.moved);

        if let Some(tgt_var) = target.as_local() {
            // 旧值需要 drop 时，RHS 必须先求值到临时变量：否则 `s = s + x`
            // 会在 RHS 读取 s 之前释放其缓冲（use-after-free）。
            // 临时变量用目标类型：保留隐式数值转换（bool→int 等）语义。
            let cleanup = self.emit_assign_cleanup(&tgt_var);
            let tmp = if cleanup.is_empty() || matches!(value.expr_type(), HirType::Never) {
                None
            } else {
                Some(self.new_temp(self.var_types[&tgt_var].clone()))
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

            stmts.extend(cleanup);
            // 赋值后目标重新有效（循环回边重赋值场景）
            self.moved.remove(&tgt_var);

            match value.as_move() {
                Some(inner) => {
                    if let Some(src_var) = inner.as_local() {
                        let ty = self.var_types[&src_var].clone();
                        let strategy = strategy_for(&ty, &self.struct_defs);
                        for action in strategy.on_move_out(src_var, &ty) {
                            stmts.push(action_to_stmt(src_var, &ty, &action));
                        }
                        self.moved.insert(src_var);
                    }
                }
                None => {
                    if let Some(inner) = value.as_clone() {
                        if let Some(src_var) = inner.as_local() {
                            let ty = self.var_types[&src_var].clone();
                            let strategy = strategy_for(&ty, &self.struct_defs);
                            for action in strategy.on_clone(src_var, &ty) {
                                stmts.push(action_to_stmt(src_var, &ty, &action));
                            }
                        }
                    }
                }
            }

            self.mark_alive(tgt_var);

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
            stmts.push(SMirAssignStmt {
                target: target.lower_to_mir(&self.moved),
                value: mir_value,
                span,
            }.into());
        } else {
            let mut mir_value = value.lower_to_mir(&self.moved);
            let (pre, po) = self.hoist_ref_temps(&mut mir_value, span);
            stmts.extend(pre);
            stmts.push(SMirAssignStmt {
                target: target.lower_to_mir(&self.moved),
                value: mir_value,
                span,
            }.into());
            post.extend(po);
        }

        stmts.extend(post);
        stmts
    }

    pub(super) fn lower_index_assign(
        &mut self,
        object: &HirNodeBox,
        index: &HirNodeBox,
        value: &HirNodeBox,
        span: crate::span::Span,
    ) -> Vec<MirStmtBox> {
        let mut mir_value = value.lower_to_mir(&self.moved);
        let (pre, post) = self.hoist_ref_temps(&mut mir_value, span);
        let mut stmts = pre;
        // 旧元素拥有堆数据时先释放（覆盖不泄漏）；仅左值对象（求值纯）。
        // 源元素移出（SMove(Index)）已清零源槽，位移 `data[j] = data[j+1]` 不产生别名。
        // 数组分配零初始化（SLirArraySized memset），新槽 drop 为零释放，安全。
        if object.as_local().is_some() || object.as_field_access().is_some() {
            if let Some(elem) = index_elem_type(&object.expr_type()) {
                if needs_drop(&elem, &self.struct_defs) {
                    // RHS 先求值到临时量（`arr[i] = arr[i]` 等自引用先取旧值）
                    let tmp = self.new_temp(elem.clone());
                    stmts.push(SMirAssignStmt {
                        target: SMirLocal { var: tmp, ty: elem.clone(), moved: false }.into(),
                        value: mir_value,
                        span,
                    }.into());
                    let old = self.new_temp(elem.clone());
                    let idx_expr: MirNodeBox = SMirIndex {
                        object: object.lower_to_mir(&self.moved),
                        index: index.lower_to_mir(&self.moved),
                        ty: elem.clone(),
                    }.into();
                    stmts.push(SMirAssignStmt {
                        target: SMirLocal { var: old, ty: elem.clone(), moved: false }.into(),
                        value: idx_expr,
                        span,
                    }.into());
                    stmts.push(SMirDropStmt { var: old, ty: elem.clone(), span }.into());
                    mir_value = SMirLocal { var: tmp, ty: elem, moved: false }.into();
                }
            }
        }
        stmts.push(SMirIndexAssignStmt {
            object: object.lower_to_mir(&self.moved),
            index: index.lower_to_mir(&self.moved),
            value: mir_value,
            span,
        }.into());
        stmts.extend(post);
        stmts
    }

    pub(super) fn lower_deref_assign(
        &mut self,
        target: &HirNodeBox,
        value: &HirNodeBox,
        span: crate::span::Span,
    ) -> Vec<MirStmtBox> {
        let mut mir_value = value.lower_to_mir(&self.moved);
        let (pre, post) = self.hoist_ref_temps(&mut mir_value, span);
        let mut stmts = pre;
        stmts.push(SMirDerefAssignStmt {
            target: target.lower_to_mir(&self.moved),
            value: mir_value,
            span,
        }.into());
        stmts.extend(post);
        stmts
    }
}

/// 索引赋值的元素类型：`unique [T]` / `ref [T]` / `[T]` / `[T; n]` → T
fn index_elem_type(ty: &HirType) -> Option<HirType> {
    let base = match ty {
        HirType::Unique(i) => i.as_ref(),
        other => other,
    };
    let base = match base {
        HirType::Ref(i, _) => i.as_ref(),
        other => other,
    };
    match base {
        HirType::Array(e) | HirType::ArraySized(e, _) => Some((**e).clone()),
        _ => None,
    }
}
