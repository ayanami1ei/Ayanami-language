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
            stmts.push(SMirDropStmt { var: old, ty: field_ty.clone(), span }.into());
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
