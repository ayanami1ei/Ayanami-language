use crate::intern::Symbol;
use std::collections::{HashMap, HashSet};

use crate::hir::ir::*;
use crate::mir::ir::*;
use crate::mir::mem::*;

fn strategy_for(ty: &HirType, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Box<dyn MemStrategy> {
    match ty {
        HirType::Unique(_) => Box::new(UniqueStrategy),
        HirType::Shared(_) => Box::new(SharedStrategy),
        HirType::Weak(_) => Box::new(ValueStrategy),
        HirType::Named(s) => {
            // Named struct — generate cleanup for each field
            if let Some(fields) = struct_defs.get(s) {
                let actions: Vec<(usize, HirType)> = fields.iter().enumerate()
                    .filter_map(|(i, (_, ft))| match ft {
                        HirType::Shared(_) | HirType::Unique(_) => Some((i, ft.clone())),
                        _ => None,
                    })
                    .collect();
                if actions.is_empty() {
                    Box::new(ValueStrategy)
                } else {
                    Box::new(StructStrategy { fields: actions })
                }
            } else {
                Box::new(ValueStrategy)
            }
        }
        _ => Box::new(ValueStrategy),
    }
}

fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmtBox {
    match action {
        MemAction::Drop(v) => SMirDropStmt { var: *v, ty: ty.clone() }.into(),
        MemAction::Retain(v) => SMirRetainStmt { var: *v, ty: ty.clone() }.into(),
        MemAction::Release(v) => SMirReleaseStmt { var: *v, ty: ty.clone() }.into(),
    }
}

pub fn lower_program(hir: &HirProgram) -> MirProgram {
    let struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>> = hir.struct_defs.iter().map(|(name, fields)| {
        (*name, fields.iter().map(|f| (f.name, f.ty.clone())).collect())
    }).collect();
    MirProgram {
        items: hir.items.iter().flat_map(|item| lower_item(item, &struct_defs)).collect(),
        vtables: hir.vtables.clone(),
        struct_defs: struct_defs.clone(),
        generic_struct_params: hir.generic_struct_params.clone(),
        imported_fns: hir.imported_fns.iter().map(|f| crate::hir::ir::ImportedFnSig {
            fn_id: f.fn_id,
            name: f.name,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
        }).collect(),
    }
}

fn lower_item(item: &HirItem, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Vec<MirItem> {
    match item {
        HirItem::Fn(f) => vec![MirItem::Fn(lower_fn(f, struct_defs))],
        HirItem::StructDef(def) => vec![MirItem::StructDef {
            name: def.name,
            fields: def.fields.iter().map(|f| (f.name, f.ty.clone())).collect(),
        }],
        HirItem::Namespace { name, items } => {
            let inner: Vec<MirItem> = items.iter().flat_map(|item| lower_item(item, struct_defs)).collect();
            vec![MirItem::Namespace { name: *name, items: inner }]
        }
        HirItem::InterfaceDef { .. } => vec![],
    }
}

fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> MirFn {
    if f.extern_c {
        return MirFn {
            fn_id: f.fn_id,
            name: f.name,
            is_inline: f.is_inline,
            extern_c: f.extern_c,
            params: f.params.clone(),
            return_type: f.return_type.clone(),
            locals: vec![],
            body: vec![],
        };
    }

    let mut ctx = Ctx::new(f, struct_defs);

    let mut body = Vec::new();
    for stmt in &f.body.stmts {
        let mut stmts = ctx.lower_stmt(stmt);
        body.append(&mut stmts);
    }

    let mut cleanup = Vec::new();
    let alive_snapshot: Vec<VarId> = ctx.alive.iter().copied().collect();
    for var in &alive_snapshot {
        if ctx.moved.contains(var) { continue; }
        let ty = ctx.var_types[var].clone();
        let strategy = strategy_for(&ty, struct_defs);
        for action in strategy.on_scope_end(*var, &ty) {
            cleanup.push(action_to_stmt(*var, &ty, &action));
        }
    }
    body.append(&mut cleanup);

    for var in &alive_snapshot {
        if ctx.moved.contains(var) { continue; }
        let ty = &ctx.var_types[var];
        let inner = match ty {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if let HirType::Named(type_name) = inner {
            if let Some(fields) = struct_defs.get(type_name) {
                for (_, field_ty) in fields {
                    match field_ty {
                        HirType::Unique(inner_field) => {}
                        HirType::Shared(inner_field) => {}
                        _ => {}
                    }
                }
            }
        }
    }

    MirFn {
        fn_id: f.fn_id,
        name: f.name,
        is_inline: f.is_inline,
        extern_c: f.extern_c,
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        locals: ctx.mir_locals,
        body,
    }
}

struct Ctx {
    mir_locals: Vec<MirLocal>,
    var_types: HashMap<VarId, HirType>,
    alive: HashSet<VarId>,
    moved: HashSet<VarId>,
    struct_defs: HashMap<Symbol, Vec<(Symbol, HirType)>>,
}

impl Ctx {
    fn new(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> Self {
        let mut var_types = HashMap::new();
        let mut mir_locals = Vec::new();
        for (i, local) in f.locals.iter().enumerate() {
            let vid = VarId(i);
            var_types.insert(vid, local.ty.clone());
            mir_locals.push(MirLocal::new(local.name, local.ty.clone(), local.mutable));
        }

        let mut alive = HashSet::new();
        for i in 0..f.params.len() {
            // Shared parameters (including shared self) are borrowed from caller — not owned by function
            if i < f.params.len() && matches!(f.params[i].1, HirType::Shared(_)) {
                continue;
            }
            alive.insert(VarId(i));
        }

        Self { mir_locals, var_types, alive, moved: HashSet::new(), struct_defs: struct_defs.clone() }
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

    fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmtBox> {
        self.track_stmt_moves(stmt);
        match stmt {
            HirStmt::Assign { target, value } => self.lower_assign(target, value),
            HirStmt::FieldAssign { object, field, field_index, field_ty, value } => {
                vec![SMirFieldAssignStmt {
                    object: object.lower_to_mir(&self.moved),
                    field: *field,
                    field_index: *field_index,
                    field_ty: field_ty.clone(),
                    value: value.lower_to_mir(&self.moved),
                }.into()]
            }
            HirStmt::IndexAssign { object, index, value } => {
                vec![SMirIndexAssignStmt {
                    object: object.lower_to_mir(&self.moved),
                    index: index.lower_to_mir(&self.moved),
                    value: value.lower_to_mir(&self.moved),
                }.into()]
            }
            HirStmt::Return { value } => self.lower_return(value),
            HirStmt::If { cond, then_block, elifs, else_block } => {
                self.lower_if(cond, then_block, elifs, else_block)
            }
            HirStmt::While { cond, body } => self.lower_while(cond, body),
            HirStmt::Break => vec![SMirBreakStmt { }.into()],
            HirStmt::Continue => vec![SMirContinueStmt { }.into()],
            HirStmt::Expr(expr) => {
                expr.record_moves(&mut self.moved);
                vec![SMirExprStmt { expr: expr.lower_to_mir(&self.moved) }.into()]
            }
            HirStmt::Block(stmts) => self.lower_block(stmts),
        }
    }

    fn track_stmt_moves(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Assign { value, .. } => value.record_moves(&mut self.moved),
            HirStmt::FieldAssign { object, value, .. } => {
                object.record_moves(&mut self.moved);
                value.record_moves(&mut self.moved);
            }
            HirStmt::IndexAssign { object, index, value } => {
                object.record_moves(&mut self.moved);
                index.record_moves(&mut self.moved);
                value.record_moves(&mut self.moved);
            }
            HirStmt::Return { value } => {
                if let Some(v) = value { v.record_moves(&mut self.moved); }
            }
            HirStmt::If { cond, then_block, elifs, else_block } => {
                cond.record_moves(&mut self.moved);
                for s in then_block.stmts.iter()
                    .chain(elifs.iter().flat_map(|(_, b)| &b.stmts))
                    .chain(else_block.iter().flat_map(|b| &b.stmts)) {
                    self.track_stmt_moves(s);
                }
            }
            HirStmt::While { cond, body } => {
                cond.record_moves(&mut self.moved);
                for s in &body.stmts { self.track_stmt_moves(s); }
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Expr(expr) => expr.record_moves(&mut self.moved),
            HirStmt::Block(stmts) => { for s in stmts { self.track_stmt_moves(s); } }
        }
    }

    fn lower_assign(&mut self, target: &HirNodeBox, value: &HirNodeBox) -> Vec<MirStmtBox> {
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
        }.into());

        stmts
    }

    fn lower_return(&mut self, value: &Option<HirNodeBox>) -> Vec<MirStmtBox> {
        let mut stmts = Vec::new();

        let mir_value = match value {
            Some(v) => {
                v.record_moves(&mut self.moved);
                if v.as_move().is_some() {
                    if let Some(var) = v.as_local() { self.moved.insert(var); }
                }
                let mut return_vars = HashSet::new();
                v.collect_var_ids(&mut return_vars);
                for var in &return_vars { self.moved.insert(*var); }
                Some(v.lower_to_mir(&self.moved))
            }
            None => None,
        };

        let mut return_vars = HashSet::new();
        if let Some(v) = value { v.collect_var_ids(&mut return_vars); }

        let alive_snapshot: Vec<VarId> = self.alive.iter().copied().collect();
        for var in &alive_snapshot {
            if self.moved.contains(var) || return_vars.contains(var) { continue; }
            let ty = self.var_types[var].clone();
            let strategy = strategy_for(&ty, &self.struct_defs);
            for action in strategy.on_scope_end(*var, &ty) {
                stmts.push(action_to_stmt(*var, &ty, &action));
            }
        }
        self.alive.clear();

        stmts.push(SMirReturnStmt { value: mir_value }.into());

        for var in &return_vars {
            if self.moved.contains(var) { continue; }
            let ty = self.var_types[var].clone();
            let strategy = strategy_for(&ty, &self.struct_defs);
            for action in strategy.on_scope_end(*var, &ty) {
                stmts.push(action_to_stmt(*var, &ty, &action));
            }
        }

        stmts
    }

    fn lower_if(
        &mut self,
        cond: &HirNodeBox,
        then_block: &HirBlock,
        elifs: &[(HirNodeBox, HirBlock)],
        else_block: &Option<HirBlock>,
    ) -> Vec<MirStmtBox> {
        let mir_cond = cond.lower_to_mir(&self.moved);
        let mir_then = self.lower_block(&then_block.stmts);
        let mir_elifs: Vec<_> = elifs
            .iter()
            .map(|(c, b)| (c.lower_to_mir(&self.moved), self.lower_block(&b.stmts)))
            .collect();
        let mir_else = else_block
            .as_ref()
            .map(|b| self.lower_block(&b.stmts));

        vec![SMirIfStmt { cond: mir_cond, then_block: mir_then, elifs: mir_elifs, else_block: mir_else }.into()]
    }

    fn lower_while(&mut self, cond: &HirNodeBox, body: &HirBlock) -> Vec<MirStmtBox> {
        let mir_cond = cond.lower_to_mir(&self.moved);
        let mir_body = self.lower_block(&body.stmts);
        vec![SMirWhileStmt { cond: mir_cond, body: mir_body }.into()]
    }

    fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmtBox> {
        let mut mir_stmts = Vec::new();
        for stmt in stmts {
            let mut lowered = self.lower_stmt(stmt);
            mir_stmts.append(&mut lowered);
        }
        mir_stmts
    }

    fn mark_alive(&mut self, var: VarId) {
        self.alive.insert(var);
    }
}
