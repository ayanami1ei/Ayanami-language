use super::*;
use super::mem::{action_to_stmt, strategy_for};

impl Ctx {
    pub(super) fn new(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Self {
        let mut var_types = HashMap::new();
        let mut mir_locals = Vec::new();
        for (i, local) in f.locals.iter().enumerate() {
            let vid = VarId(i);
            var_types.insert(vid, local.ty.clone());
            mir_locals.push(MirLocal::new(local.name, local.ty.clone(), local.mutable));
        }

        let mut alive = HashSet::new();
        for i in 0..f.params.len() {
            alive.insert(VarId(i));
        }

        Self {
            mir_locals,
            var_types,
            alive,
            moved: HashSet::new(),
            struct_defs: struct_defs.clone(),
            errors: Vec::new(),
            return_type: f.return_type.clone(),
        }
    }

    fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();
        if self.alive.contains(var) && !self.moved.contains(var) {
            let ty = &self.var_types[var];
            let strategy = strategy_for(ty, &self.struct_defs);
            for action in strategy.on_assign_overwrite(*var, ty) {
                stmts.push(action_to_stmt(*var, ty, &action));
            }
        }
        stmts
    }

    pub(super) fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmtBox> {
        self.check_use_after_move(stmt);
        self.track_stmt_moves(stmt);
        match stmt {
            HirStmt::Assign { target, value, span } => self.lower_assign(target, value, *span),
            HirStmt::FieldAssign { object, field, field_index, field_ty, value, span } => {
                vec![SMirFieldAssignStmt {
                    object: object.lower_to_mir(&self.moved),
                    field: *field,
                    field_index: *field_index,
                    field_ty: field_ty.clone(),
                    value: value.lower_to_mir(&self.moved),
                    span: *span,
                }.into()]
            }
            HirStmt::IndexAssign { object, index, value, span } => {
                vec![SMirIndexAssignStmt {
                    object: object.lower_to_mir(&self.moved),
                    index: index.lower_to_mir(&self.moved),
                    value: value.lower_to_mir(&self.moved),
                    span: *span,
                }.into()]
            }
            HirStmt::DerefAssign { target, value, span } => {
                vec![SMirDerefAssignStmt {
                    target: target.lower_to_mir(&self.moved),
                    value: value.lower_to_mir(&self.moved),
                    span: *span,
                }.into()]
            }
            HirStmt::Return { value, span } => self.lower_return(value, *span),
            HirStmt::If { cond, then_block, elifs, else_block, span } => {
                self.lower_if(cond, then_block, elifs, else_block, *span)
            }
            HirStmt::While { cond, body, span } => self.lower_while(cond, body, *span),
            HirStmt::Break { span } => vec![SMirBreakStmt { span: *span }.into()],
            HirStmt::Continue { span } => vec![SMirContinueStmt { span: *span }.into()],
            HirStmt::Expr { expr, span } => {
                expr.record_moves(&mut self.moved);
                vec![SMirExprStmt { expr: expr.lower_to_mir(&self.moved), span: *span }.into()]
            }
            HirStmt::Block { stmts, .. } => self.lower_block(stmts),
            HirStmt::Assume { cond, span } => {
                cond.record_moves(&mut self.moved);
                vec![SMirAssumeStmt { cond: cond.lower_to_mir(&self.moved), span: *span }.into()]
            }
            HirStmt::Contract { kind, cond, line, col } => {
                cond.record_moves(&mut self.moved);
                vec![SMirContractStmt {
                    kind: *kind, cond: cond.lower_to_mir(&self.moved),
                    line: *line as u64, col: *col as u64,
                    span: crate::span::Span::new(*line, *col, *line, *col, 0, 0),
                }.into()]
            }
        }
    }

    fn track_stmt_moves(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Assign { value, .. } => value.record_moves(&mut self.moved),
            HirStmt::DerefAssign { value, .. } => value.record_moves(&mut self.moved),
            HirStmt::FieldAssign { object, value, .. } => {
                object.record_moves(&mut self.moved);
                value.record_moves(&mut self.moved);
            }
            HirStmt::IndexAssign { object, index, value, .. } => {
                object.record_moves(&mut self.moved);
                index.record_moves(&mut self.moved);
                value.record_moves(&mut self.moved);
            }
            HirStmt::Return { value, .. } => {
                if let Some(v) = value { v.record_moves(&mut self.moved); }
            }
            // 复合语句不预标记：子语句在各自 lower_stmt 中按顺序跟踪移动
            HirStmt::If { .. } | HirStmt::While { .. } => {}
            HirStmt::Break { .. } | HirStmt::Continue { .. } => {}
            HirStmt::Expr { expr, .. } => expr.record_moves(&mut self.moved),
            HirStmt::Assume { cond, .. } => cond.record_moves(&mut self.moved),
            HirStmt::Contract { cond, .. } => cond.record_moves(&mut self.moved),
            HirStmt::Block { stmts, .. } => { for s in stmts { self.track_stmt_moves(s); } }
        }
    }

    fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox, span: crate::span::Span) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();

        value.record_moves(&mut self.moved);

        if let Some(tgt_var) = target.as_local() {
            let cleanup = self.emit_assign_cleanup(&tgt_var);
            stmts.extend(cleanup);

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
        }

        stmts.push(SMirAssignStmt {
            target: target.lower_to_mir(&self.moved),
            value: value.lower_to_mir(&self.moved),
            span,
        }.into());

        stmts
    }

    fn lower_return(&mut self, value: &Option<HirNodeBox>, span: crate::span::Span) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();

        // 先把返回值求值到临时变量，再做作用域清理，
        // 避免 `return up.x` 这类表达式在 drop 之后才读取变量（use-after-free）。
        // M1.9：发散返回（`return <!>` / `-> !` 函数）直接求值，不建临时变量（void 无 alloca）
        let mut direct_value: Option<MirNodeBox> = None;
        let mut ret_var: Option<VarId> = None;
        if let Some(v) = value {
            let is_never = matches!(v.expr_type(), HirType::Never)
                || matches!(self.return_type, HirType::Never);
            if is_never {
                direct_value = Some(v.lower_to_mir(&self.moved));
            } else {
                // 临时变量使用函数返回类型（比较运算等表达式类型可能与返回类型不同）
                let ty = self.return_type.clone();
                let tmp = self.new_temp(ty);
                let mir_value = v.lower_to_mir(&self.moved);
                stmts.push(SMirAssignStmt {
                    target: SMirLocal { var: tmp, ty: self.var_types[&tmp].clone(), moved: false }.into(),
                    value: mir_value,
                    span,
                }.into());
                ret_var = Some(tmp);
            }
        }

        let alive_snapshot: Vec<VarId> = self.alive.iter().copied().collect();
        for var in &alive_snapshot {
            if self.moved.contains(var) { continue; }
            let ty = self.var_types[var].clone();
            let strategy = strategy_for(&ty, &self.struct_defs);
            for action in strategy.on_scope_end(*var, &ty) {
                stmts.push(action_to_stmt(*var, &ty, &action));
            }
        }
        self.alive.clear();

        let mir_value = if let Some(dv) = direct_value {
            Some(dv)
        } else {
            ret_var.map(|var| SMirLocal { var, ty: self.var_types[&var].clone(), moved: false }.into())
        };
        stmts.push(SMirReturnStmt { value: mir_value, span }.into());

        stmts
    }

    /// 申请一个仅供编译器内部使用的临时局部变量（返回语句求值用）。
    fn new_temp(&mut self, ty: HirType) -> VarId {
        let id = VarId(self.mir_locals.len());
        self.mir_locals.push(MirLocal::new(Symbol::intern("__ret"), ty.clone(), false));
        self.var_types.insert(id, ty);
        id
    }
}
