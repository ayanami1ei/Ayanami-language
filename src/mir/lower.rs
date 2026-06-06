use std::collections::{HashMap, HashSet};

use crate::hir::ir::*;
use crate::mir::ir::*;
use crate::mir::mem::*;

fn strategy_for(ty: &HirType) -> Box<dyn MemStrategy> {
    match ty {
        HirType::Unique(_) => Box::new(UniqueStrategy),
        HirType::Shared(_) | HirType::Weak(_) => Box::new(SharedStrategy),
        _ => Box::new(ValueStrategy),
    }
}

fn action_to_stmt(_var: VarId, ty: &HirType, action: &MemAction) -> MirStmt {
    match action {
        MemAction::Drop(v) => MirStmt::Drop(*v, ty.clone()),
        MemAction::Retain(v) => MirStmt::Retain(*v, ty.clone()),
        MemAction::Release(v) => MirStmt::Release(*v, ty.clone()),
    }
}

fn mir_expr_from_hir(expr: &HirExpr, moved: &HashSet<VarId>) -> MirExpr {
    match expr {
        HirExpr::Literal(lit, ty) => MirExpr::Literal(lit.clone(), ty.clone()),
        HirExpr::Local(id, ty) => MirExpr::Local(*id, ty.clone(), moved.contains(id)),
        HirExpr::Binary { op, lhs, rhs, ty } => MirExpr::Binary {
            op: *op,
            lhs: Box::new(mir_expr_from_hir(lhs, moved)),
            rhs: Box::new(mir_expr_from_hir(rhs, moved)),
            ty: ty.clone(),
        },
        HirExpr::Unary { op, arg, ty } => MirExpr::Unary {
            op: *op,
            arg: Box::new(mir_expr_from_hir(arg, moved)),
            ty: ty.clone(),
        },
        HirExpr::Call { fn_id, args, ty } => MirExpr::Call {
            fn_id: *fn_id,
            args: args.iter().map(|a| mir_expr_from_hir(a, moved)).collect(),
            ty: ty.clone(),
        },
        HirExpr::Move(inner, ty) => {
            MirExpr::Move(Box::new(mir_expr_from_hir(inner, moved)), ty.clone())
        }
        HirExpr::Clone(inner, ty) => {
            MirExpr::Clone(Box::new(mir_expr_from_hir(inner, moved)), ty.clone())
        }
        HirExpr::ToUnique(inner, ty) => {
            MirExpr::ToUnique(Box::new(mir_expr_from_hir(inner, moved)), ty.clone())
        }
        HirExpr::ToShared(inner, ty) => {
            MirExpr::ToShared(Box::new(mir_expr_from_hir(inner, moved)), ty.clone())
        }
        HirExpr::ToWeak(inner, ty) => {
            MirExpr::ToWeak(Box::new(mir_expr_from_hir(inner, moved)), ty.clone())
        }
        HirExpr::VirtualCall { receiver, interface, method_index, args, ty } => MirExpr::VirtualCall {
            receiver: Box::new(mir_expr_from_hir(receiver, moved)),
            interface: *interface,
            method_index: *method_index,
            args: args.iter().map(|a| mir_expr_from_hir(a, moved)).collect(),
            ty: ty.clone(),
        },
        HirExpr::MakeFatPtr { value, concrete_type, interface_name, ty } => MirExpr::MakeFatPtr {
            value: Box::new(mir_expr_from_hir(value, moved)),
            concrete_type: *concrete_type,
            interface_name: *interface_name,
            ty: ty.clone(),
        },
        HirExpr::FieldAccess { object, field, field_index, ty } => MirExpr::FieldAccess {
            object: Box::new(mir_expr_from_hir(object, moved)),
            field: *field,
            field_index: *field_index,
            ty: ty.clone(),
        },
        HirExpr::StructLiteral { type_name, fields, ty } => MirExpr::StructLiteral {
            type_name: *type_name,
            fields: fields.iter().map(|(n, e)| (*n, mir_expr_from_hir(e, moved))).collect(),
            ty: ty.clone(),
        },
        HirExpr::ArrayLiteral(elems, ty) => MirExpr::ArrayLiteral(
            elems.iter().map(|e| mir_expr_from_hir(e, moved)).collect(),
            ty.clone(),
        ),
        HirExpr::Index { object, index, ty } => MirExpr::Index {
            object: Box::new(mir_expr_from_hir(object, moved)),
            index: Box::new(mir_expr_from_hir(index, moved)),
            ty: ty.clone(),
        },
    }
}

fn extract_var_id(expr: &HirExpr) -> Option<VarId> {
    match expr {
        HirExpr::Local(id, _) => Some(*id),
        _ => None,
    }
}

fn collect_var_ids(expr: &HirExpr) -> HashSet<VarId> {
    let mut vars = HashSet::new();
    match expr {
        HirExpr::Local(id, _) => { vars.insert(*id); }
        HirExpr::Binary { lhs, rhs, .. } => {
            vars.extend(collect_var_ids(lhs));
            vars.extend(collect_var_ids(rhs));
        }
        HirExpr::Unary { arg, .. } => vars.extend(collect_var_ids(arg)),
        HirExpr::Call { args, .. } => {
            for a in args { vars.extend(collect_var_ids(a)); }
        }
        HirExpr::Move(inner, _) | HirExpr::Clone(inner, _) => vars.extend(collect_var_ids(inner)),
        HirExpr::ToUnique(inner, _) | HirExpr::ToShared(inner, _) | HirExpr::ToWeak(inner, _) => vars.extend(collect_var_ids(inner)),
        HirExpr::VirtualCall { receiver, args, .. } => {
            vars.extend(collect_var_ids(receiver));
            for a in args { vars.extend(collect_var_ids(a)); }
        }
        HirExpr::MakeFatPtr { value, .. } => vars.extend(collect_var_ids(value)),
        HirExpr::FieldAccess { object, .. } => vars.extend(collect_var_ids(object)),
        HirExpr::StructLiteral { fields, .. } => {
            for (_, e) in fields { vars.extend(collect_var_ids(e)); }
        }
        HirExpr::ArrayLiteral(elems, _) => {
            for e in elems { vars.extend(collect_var_ids(e)); }
        }
        HirExpr::Index { object, index, .. } => {
            vars.extend(collect_var_ids(object));
            vars.extend(collect_var_ids(index));
        }
        _ => {}
    }
    vars
}

pub fn lower_program(hir: &HirProgram) -> MirProgram {
    MirProgram {
        items: hir.items.iter().flat_map(lower_item).collect(),
        vtables: hir.vtables.clone(),
        struct_defs: hir.struct_defs.iter().map(|(name, fields)| {
            (*name, fields.iter().map(|f| (f.name, f.ty.clone())).collect())
        }).collect(),
    }
}

fn lower_item(item: &HirItem) -> Vec<MirItem> {
    match item {
        HirItem::Fn(f) => vec![MirItem::Fn(lower_fn(f))],
        HirItem::StructDef(def) => vec![MirItem::StructDef {
            name: def.name,
            fields: def.fields.iter().map(|f| (f.name, f.ty.clone())).collect(),
        }],
        HirItem::Namespace { name, items } => {
            let inner: Vec<MirItem> = items.iter().flat_map(lower_item).collect();
            vec![MirItem::Namespace { name: *name, items: inner }]
        }
        // InterfaceDef has no runtime code — skip
        HirItem::InterfaceDef { .. } => vec![],
    }
}

fn lower_fn(f: &HirFn) -> MirFn {
    let mut ctx = Ctx::new(f);

    let mut body = Vec::new();
    for stmt in &f.body.stmts {
        let mut stmts = ctx.lower_stmt(stmt);
        body.append(&mut stmts);
    }

    // --- Function scope end: cleanup remaining alive variables ---
    let mut cleanup = Vec::new();
    let alive_snapshot: Vec<VarId> = ctx.alive.iter().copied().collect();
    for var in &alive_snapshot {
        if ctx.moved.contains(var) {
            continue;
        }
        let ty = ctx.var_types[var].clone();
        let strategy = strategy_for(&ty);
        for action in strategy.on_scope_end(*var, &ty) {
            cleanup.push(action_to_stmt(*var, &ty, &action));
        }
    }
    body.append(&mut cleanup);

    MirFn {
        fn_id: f.fn_id,
        name: f.name,
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
}

impl Ctx {
    fn new(f: &HirFn) -> Self {
        let mut var_types = HashMap::new();
        let mut mir_locals = Vec::new();
        for (i, local) in f.locals.iter().enumerate() {
            let vid = VarId(i);
            var_types.insert(vid, local.ty.clone());
            mir_locals.push(MirLocal::new(local.name, local.ty.clone(), local.mutable));
        }

        // Params (indices 0..params.len()) are alive from the start
        let mut alive = HashSet::new();
        for i in 0..f.params.len() {
            alive.insert(VarId(i));
        }

        Self { mir_locals, var_types, alive, moved: HashSet::new() }
    }

    fn emit_assign_cleanup(&self, var: &VarId) -> Vec<MirStmt> {
        let mut stmts = Vec::new();
        if self.alive.contains(var) && !self.moved.contains(var) {
            let ty = &self.var_types[var];
            let strategy = strategy_for(ty);
            for action in strategy.on_assign_overwrite(*var, ty) {
                stmts.push(action_to_stmt(*var, ty, &action));
            }
        }
        stmts
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<MirStmt> {
        match stmt {
            HirStmt::Assign { target, value } => self.lower_assign(target, value),
            HirStmt::Return { value } => self.lower_return(value),
            HirStmt::If { cond, then_block, elifs, else_block } => {
                self.lower_if(cond, then_block, elifs, else_block)
            }
            HirStmt::While { cond, body } => self.lower_while(cond, body),
            HirStmt::Expr(expr) => vec![MirStmt::Expr(mir_expr_from_hir(expr, &self.moved))],
            HirStmt::Block(stmts) => self.lower_block(stmts),
        }
    }

    fn lower_assign(&mut self, target: &HirExpr, value: &HirExpr) -> Vec<MirStmt> {
        let mut stmts = Vec::new();

        // If target is a local, handle cleanup of old value
        if let Some(tgt_var) = extract_var_id(target) {
            let cleanup = self.emit_assign_cleanup(&tgt_var);
            stmts.extend(cleanup);

            // Handle move/clone side-effects on source
            match value {
                HirExpr::Move(inner, _) => {
                    if let Some(src_var) = extract_var_id(inner) {
                        let ty = self.var_types[&src_var].clone();
                        let strategy = strategy_for(&ty);
                        for action in strategy.on_move_out(src_var, &ty) {
                            stmts.push(action_to_stmt(src_var, &ty, &action));
                        }
                        self.moved.insert(src_var);
                    }
                }
                HirExpr::Clone(inner, _) => {
                    if let Some(src_var) = extract_var_id(inner) {
                        let ty = self.var_types[&src_var].clone();
                        let strategy = strategy_for(&ty);
                        for action in strategy.on_clone(src_var, &ty) {
                            stmts.push(action_to_stmt(src_var, &ty, &action));
                        }
                    }
                }
                _ => {}
            }

            self.mark_alive(tgt_var);
        }

        stmts.push(MirStmt::Assign {
            target: mir_expr_from_hir(target, &self.moved),
            value: mir_expr_from_hir(value, &self.moved),
        });

        stmts
    }

    fn lower_return(&mut self, value: &Option<HirExpr>) -> Vec<MirStmt> {
        let mut stmts = Vec::new();

        let mir_value = match value {
            Some(v) => {
                match v {
                    HirExpr::Move(inner, _) => {
                        if let Some(var) = extract_var_id(inner) {
                            self.moved.insert(var);
                        }
                    }
                    _ => {}
                }
                Some(mir_expr_from_hir(v, &self.moved))
            }
            None => None,
        };

        // Collect variables used in the return expression
        let return_vars: HashSet<VarId> = value.as_ref().map(collect_var_ids).unwrap_or_default();

        // Cleanup variables NOT used in the return expression (safe to clean before return)
        let alive_snapshot: Vec<VarId> = self.alive.iter().copied().collect();
        for var in &alive_snapshot {
            if self.moved.contains(var) || return_vars.contains(var) {
                continue;
            }
            let ty = self.var_types[var].clone();
            let strategy = strategy_for(&ty);
            for action in strategy.on_scope_end(*var, &ty) {
                stmts.push(action_to_stmt(*var, &ty, &action));
            }
        }
        self.alive.clear();

        stmts.push(MirStmt::Return { value: mir_value });

        // Emit cleanup for return variables AFTER return (LIR will process them before Ret)
        for var in &return_vars {
            if self.moved.contains(var) {
                continue;
            }
            let ty = self.var_types[var].clone();
            let strategy = strategy_for(&ty);
            for action in strategy.on_scope_end(*var, &ty) {
                stmts.push(action_to_stmt(*var, &ty, &action));
            }
        }

        stmts
    }

    fn lower_if(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        elifs: &[(HirExpr, HirBlock)],
        else_block: &Option<HirBlock>,
    ) -> Vec<MirStmt> {
        let mir_cond = mir_expr_from_hir(cond, &self.moved);
        let mir_then = self.lower_block(&then_block.stmts);
        let mir_elifs: Vec<_> = elifs
            .iter()
            .map(|(c, b)| (mir_expr_from_hir(c, &self.moved), self.lower_block(&b.stmts)))
            .collect();
        let mir_else = else_block
            .as_ref()
            .map(|b| self.lower_block(&b.stmts));

        vec![MirStmt::If {
            cond: mir_cond,
            then_block: mir_then,
            elifs: mir_elifs,
            else_block: mir_else,
        }]
    }

    fn lower_while(&mut self, cond: &HirExpr, body: &HirBlock) -> Vec<MirStmt> {
        let mir_cond = mir_expr_from_hir(cond, &self.moved);
        let mir_body = self.lower_block(&body.stmts);
        vec![MirStmt::While {
            cond: mir_cond,
            body: mir_body,
        }]
    }

    fn lower_block(&mut self, stmts: &[HirStmt]) -> Vec<MirStmt> {
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
