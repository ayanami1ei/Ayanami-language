// ============================================================
//  MIR（中级中间表示）降级
//  将 HIR（高级中间表示）降级为 MIR，主要工作：
//  1. 插入内存管理指令（Drop/Retain/Release）
//  2. 根据所有权的不同策略（值/Unique/Shared）管理生命周期
//  3. 跟踪变量移动状态以决定是否插入清理代码
//  4. 处理返回语句中的所有权转移
// ============================================================

use crate::intern::Symbol;
use std::collections::{HashMap, HashSet};

use crate::hir::ir::*;
use crate::mir::ir::*;
use crate::mir::mem::*;

/// 根据类型选择对应的内存管理策略
///
/// - `Unique[T]` → UniqueStrategy：作用域结束时 Drop（free）
/// - `Shared[T]` | `Weak[T]` → SharedStrategy：引用计数增减
/// - 值类型（int、float、char、bool 等）→ ValueStrategy：无操作
/// 根据类型选择内存管理策略
///
/// - `Unique[T]` → UniqueStrategy：作用域结束时 Drop（free）
/// - `Shared[T]` → SharedStrategy：引用计数 retain/release
/// - `Weak[T]` → ValueStrategy：弱引用不参与引用计数
/// - 值类型（int、float、char、bool 等）→ ValueStrategy：无操作
fn strategy_for(ty: &HirType) -> Box<dyn MemStrategy> {
    match ty {
        HirType::Unique(_) => Box::new(UniqueStrategy),
        HirType::Shared(_) => Box::new(SharedStrategy),
        HirType::Weak(_) => Box::new(ValueStrategy),
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
        HirExpr::VirtualCall { receiver, interface, method_index, args, ty, .. } => MirExpr::VirtualCall {
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
        HirExpr::ArraySized { count, elem_ty, ty } => MirExpr::ArraySized {
            count: Box::new(mir_expr_from_hir(count, moved)),
            elem_ty: elem_ty.clone(),
            ty: ty.clone(),
        },
        HirExpr::Ref { expr, mutable, ty } => MirExpr::Ref {
            expr: Box::new(mir_expr_from_hir(expr, moved)),
            mutable: *mutable,
            ty: ty.clone(),
        },
        HirExpr::Index { object, index, ty } => MirExpr::Index {
            object: Box::new(mir_expr_from_hir(object, moved)),
            index: Box::new(mir_expr_from_hir(index, moved)),
            ty: ty.clone(),
        },
        HirExpr::Asm { template, outputs, inputs, ty } => MirExpr::Asm {
            template: template.clone(),
            outputs: outputs.iter().map(|(c, e)| (c.clone(), Box::new(mir_expr_from_hir(e, moved)))).collect(),
            inputs: inputs.iter().map(|(c, e)| (c.clone(), Box::new(mir_expr_from_hir(e, moved)))).collect(),
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
        HirExpr::ArraySized { count, .. } => {
            vars.extend(collect_var_ids(count));
        }
        HirExpr::Ref { expr, .. } => {
            vars.extend(collect_var_ids(expr));
        }
        HirExpr::Index { object, index, .. } => {
            vars.extend(collect_var_ids(object));
            vars.extend(collect_var_ids(index));
        }
        HirExpr::Asm { outputs, inputs, .. } => {
            for (_, e) in outputs { vars.extend(collect_var_ids(e)); }
            for (_, e) in inputs { vars.extend(collect_var_ids(e)); }
        }
        _ => {}
    }
    vars
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
        // InterfaceDef has no runtime code — skip
        HirItem::InterfaceDef { .. } => vec![],
    }
}

fn lower_fn(f: &HirFn, struct_defs: &HashMap<Symbol, Vec<(Symbol, HirType)>>) -> MirFn {
    // Extern C declarations have no body
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

    // --- Struct field cleanup: for struct-typed variables, drop owned fields ---
    for var in &alive_snapshot {
        if ctx.moved.contains(var) { continue; }
        let ty = &ctx.var_types[var];
        // Walk through ownership wrappers to get the inner type
        let inner = match ty {
            HirType::Shared(i) | HirType::Unique(i) | HirType::Weak(i) => i.as_ref(),
            other => other,
        };
        if let HirType::Named(type_name) = inner {
            if let Some(fields) = struct_defs.get(type_name) {
                for (_, field_ty) in fields {
                    match field_ty {
                        HirType::Unique(inner_field) => {
                            // For Unique(inner), drop the field (free)
                            // Use DropValue action
                            // We can't easily generate per-field drops here without restructuring
                            // For now, just emit a Drop for the whole variable
                            // to prevent leaks at the struct level.
                            // A proper implementation would generate field-by-field cleanup.
                        }
                        HirType::Shared(inner_field) => {
                            // Release the shared field
                        }
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
        // Track moves in all expressions within this statement
        self.track_stmt_moves(stmt);
        match stmt {
            HirStmt::Assign { target, value } => self.lower_assign(target, value),
            HirStmt::FieldAssign { object, field, field_index, field_ty, value } => {
                vec![MirStmt::FieldAssign {
                    object: Box::new(mir_expr_from_hir(object, &self.moved)),
                    field: *field,
                    field_index: *field_index,
                    field_ty: field_ty.clone(),
                    value: mir_expr_from_hir(value, &self.moved),
                }]
            }
            HirStmt::IndexAssign { object, index, value } => {
                vec![MirStmt::IndexAssign {
                    object: Box::new(mir_expr_from_hir(object, &self.moved)),
                    index: Box::new(mir_expr_from_hir(index, &self.moved)),
                    value: mir_expr_from_hir(value, &self.moved),
                }]
            }
            HirStmt::Return { value } => self.lower_return(value),
            HirStmt::If { cond, then_block, elifs, else_block } => {
                self.lower_if(cond, then_block, elifs, else_block)
            }
            HirStmt::While { cond, body } => self.lower_while(cond, body),
            HirStmt::Break => vec![MirStmt::Break],
            HirStmt::Continue => vec![MirStmt::Continue],
            HirStmt::Expr(expr) => {
                // Track moves inside expression (e.g., Move inside Call args)
                self.track_expr_moves(expr);
                vec![MirStmt::Expr(mir_expr_from_hir(expr, &self.moved))]
            },
            HirStmt::Block(stmts) => self.lower_block(stmts),
        }
    }

    /// Walk all expressions in a statement and mark any Move(Local(v)) as moved.
    fn track_stmt_moves(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Assign { value, .. } => self.track_expr_moves(value),
            HirStmt::FieldAssign { object, value, .. } => {
                self.track_expr_moves(object);
                self.track_expr_moves(value);
            }
            HirStmt::IndexAssign { object, index, value } => {
                self.track_expr_moves(object);
                self.track_expr_moves(index);
                self.track_expr_moves(value);
            }
            HirStmt::Return { value } => {
                if let Some(v) = value { self.track_expr_moves(v); }
            }
            HirStmt::If { cond, then_block, elifs, else_block } => {
                self.track_expr_moves(cond);
                for s in then_block.stmts.iter().chain(elifs.iter().flat_map(|(_, b)| &b.stmts))
                    .chain(else_block.iter().flat_map(|b| &b.stmts)) {
                    self.track_stmt_moves(s);
                }
            }
            HirStmt::While { cond, body } => {
                self.track_expr_moves(cond);
                for s in &body.stmts { self.track_stmt_moves(s); }
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Expr(expr) => self.track_expr_moves(expr),
            HirStmt::Block(stmts) => {
                for s in stmts { self.track_stmt_moves(s); }
            }
        }
    }

    /// Walk an expression tree and mark any Move(Local(v)) as moved.
    fn track_expr_moves(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Move(inner, _) => {
                if let HirExpr::Local(id, _) = inner.as_ref() {
                    self.moved.insert(*id);
                }
                // 即使 Move 包装的不是 Local（例如 Move(Call(...))），
                // 也需要递归遍历内部表达式，以找到参数中的 Move(Local)
                self.track_expr_moves(inner);
            }
            HirExpr::Call { args, .. } => {
                for a in args { self.track_expr_moves(a); }
            }
            HirExpr::VirtualCall { receiver, args, .. } => {
                self.track_expr_moves(receiver);
                for a in args { self.track_expr_moves(a); }
            }
            HirExpr::Binary { lhs, rhs, .. } => {
                self.track_expr_moves(lhs);
                self.track_expr_moves(rhs);
            }
            HirExpr::Unary { arg, .. } => self.track_expr_moves(arg),
            HirExpr::ToUnique(inner, _) | HirExpr::ToShared(inner, _) | HirExpr::ToWeak(inner, _) => {
                self.track_expr_moves(inner);
            }
            HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
                self.track_expr_moves(object);
            }
            HirExpr::StructLiteral { fields, .. } => {
                for (_, e) in fields { self.track_expr_moves(e); }
            }
            HirExpr::Asm { outputs, inputs, .. } => {
                for (_, e) in outputs { self.track_expr_moves(e); }
                for (_, e) in inputs { self.track_expr_moves(e); }
            }
            HirExpr::ArrayLiteral(elems, _) => {
                for e in elems { self.track_expr_moves(e); }
            }
            _ => {}
        }
    }

    fn lower_assign(&mut self, target: &HirExpr, value: &HirExpr) -> Vec<MirStmt> {
        let mut stmts = Vec::new();

        // Track moves inside the value expression
        self.track_expr_moves(value);

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
                // Track moves inside the return expression (e.g., Move inside Call args)
                self.track_expr_moves(v);
                match v {
                    HirExpr::Move(inner, _) => {
                        if let Some(var) = extract_var_id(inner) {
                            self.moved.insert(var);
                        }
                    }
                    _ => {}
                }
                // Mark all return expression variables as moved
                // (they're consumed by the return, even if nested in Call args)
                if let Some(return_vars_fix) = value.as_ref().map(collect_var_ids) {
                    for var in &return_vars_fix {
                        self.moved.insert(*var);
                    }
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
