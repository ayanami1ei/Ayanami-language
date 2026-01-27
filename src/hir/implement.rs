use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    hir::{
        self, BinOperator, BlockId, FuncId, HIR, HIRInst, HirFuncSymbol, HirGenerator,
        HirVarSymbol, ObjId, ObjSlot, SlotId, StorageClass, UnaryOperation, Value, VarId,
    },
    symbol_table::{Symbol, SymbolTable},
    types::{Argc, Block, Expr, Stmt, VarType},
};

impl ObjSlot {
    pub(super) fn push(&mut self, value: Value) {
        self.set.insert(value);
    }

    pub(super) fn new(id: i32) -> ObjSlot {
        ObjSlot {
            set: HashSet::new(),
            home_level_id: id,
            cur_leve_id: id,

            last_use: 0,
            escape: false,
        }
    }

    pub(super) fn append(&mut self, value: &ObjSlot) {
        self.set.extend(value.set.iter().cloned());
    }

    pub(super) fn escape(&mut self) {
        self.escape = true;
    }

    pub(super) fn set_near_use(&mut self, index: usize) {
        self.last_use = index;
    }
}

impl SlotId {
    pub(super) fn push(self, map: &mut HashMap<SlotId, ObjSlot>, value: Value) {
        let mut obj_slot = match map.get(&self) {
            None => panic!(""),
            Some(x) => x.clone(),
        };
        obj_slot.push(value);
        map.insert(self, obj_slot);
    }

    pub(super) fn append(self, map: &mut HashMap<SlotId, ObjSlot>, value: &SlotId) {
        let mut obj_slot = match map.get(&self) {
            None => panic!(""),
            Some(x) => x.clone(),
        };

        let value_slot = match map.get(value) {
            None => panic!(""),
            Some(x) => x.clone(),
        };

        obj_slot.append(&value_slot);
        map.insert(self, obj_slot);
    }

    pub(super) fn set_near_use(self, map: &mut HashMap<SlotId, ObjSlot>, index: usize) {
        let mut slot = match map.get(&self) {
            None => panic!(""),
            Some(x) => x.clone(),
        };

        slot.last_use = index;
        map.insert(self, slot);
    }

    pub(super) fn escape(self, map: &mut HashMap<SlotId, ObjSlot>) {
        let mut slot = match map.get(&self) {
            None => panic!(""),
            Some(x) => x.clone(),
        };
        slot.escape();
        map.insert(self, slot);
    }
}

impl HirVarSymbol {
    pub(super) fn new(ast_sym: Symbol, obj_id: SlotId) -> HirVarSymbol {
        let mutability = true;
        let mut storage = StorageClass::Local;

        if ast_sym.is_argc {
            storage = StorageClass::Param;
        }

        HirVarSymbol {
            ty_set: ast_sym.its_type,
            mutability,
            storage,
            obj_id,
        }
    }
}

impl HirGenerator {
    pub(crate) fn new(stmts: Vec<Stmt>, ast_symbol_table: SymbolTable) -> HirGenerator {
        HirGenerator {
            stmts,

            next_objid: ObjId {
                id: 0,
                home_level_id: 0,
                cur_leve_id: 0,
            },
            next_blockid: BlockId {
                id: 0,
                is_merge: false,
            },
            next_slotid: SlotId { id: 0 },

            var_registry: HashMap::<VarId, HirVarSymbol>::new(),
            obj_registry: HashMap::<i32, ObjId>::new(),
            func_registry: HashMap::<FuncId, HirFuncSymbol>::new(),
            block_registry: HashMap::<BlockId, Vec<HIRInst>>::new(),
            slot_registry: HashMap::<SlotId, ObjSlot>::new(),

            ast_symbol_table,

            hir: Vec::<HIR>::new(),
        }
    }

    fn can_be_int(x: f64) -> bool {
        x.is_finite() && x.fract() == 0.0 && x.abs() <= (1_i64 << 53) as f64
    }

    fn get_next_obj_id(&mut self) -> ObjId {
        let mut id = self.next_objid;
        self.next_objid += 1;
        id.home_level_id = self.ast_symbol_table.get_level();
        id.cur_leve_id = self.ast_symbol_table.get_level();
        self.obj_registry.insert(id.id, id);
        id
    }
    fn get_next_slot_id(&mut self) -> SlotId {
        let id = self.next_slotid;
        self.next_slotid += 1;
        self.slot_registry
            .insert(id, ObjSlot::new(self.ast_symbol_table.get_level()));
        id
    }

    fn find_var_id(&mut self, name: &String) -> VarId {
        match self.ast_symbol_table.find_symbol(name) {
            None => panic!("failed in find id of {}", name),
            Some(sys) => VarId(sys.id),
        }
    }
    fn find_func_sym(&mut self, name: &String) -> HirFuncSymbol {
        let id = FuncId(match self.ast_symbol_table.find_symbol(name) {
            None => panic!(""),
            Some(sys) => sys.id,
        });

        match self.func_registry.get(&id) {
            None => panic!(""),
            Some(sym) => sym.clone(),
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.next_blockid;
        self.next_blockid += 1;

        self.block_registry.insert(id, Vec::<HIRInst>::new());

        id
    }
    fn new_merge_block(&mut self) -> BlockId {
        let mut id = self.next_blockid;
        self.next_blockid += 1;
        id.is_merge = true;

        self.block_registry.insert(id, Vec::<HIRInst>::new());

        id
    }
    fn emit_block(&mut self, id: BlockId) {
        self.hir.push(HIR::Block(id));
        let block = match self.block_registry.get(&id) {
            None => panic!(""),
            Some(x) => x.clone(),
        };

        for i in block {
            self.hir.push(HIR::Inst(i));
        }
    }

    #[inline]
    fn bin_op_template(
        &mut self,
        left: &Rc<RefCell<Expr>>,
        right: &Rc<RefCell<Expr>>,
        op_type: BinOperator,
        irs: &mut Vec<HIRInst>,
        res_slot_id: &mut SlotId,
    ) {
        let left_slot_id = self.gen_expr_ir(left);
        let right_slot_id = self.gen_expr_ir(right);

        let dst_slot_id = self.get_next_slot_id();

        irs.push(HIRInst::BinOp {
            left: left_slot_id,
            op: op_type,
            right: right_slot_id,
            dst: dst_slot_id,
        });

        left_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());
        right_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());

        *res_slot_id = dst_slot_id;
    }
    fn gen_expr_ir(&mut self, expr: &Rc<RefCell<Expr>>) -> SlotId {
        let mut irs = Vec::<HIRInst>::new();
        let mut res_slot_id = SlotId::default();

        match *expr.borrow() {
            Expr::Add(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Add, &mut irs, &mut res_slot_id);
            }
            Expr::Sub(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Sub, &mut irs, &mut res_slot_id);
            }
            Expr::Mul(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Mul, &mut irs, &mut res_slot_id);
            }
            Expr::Div(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Div, &mut irs, &mut res_slot_id);
            }
            Expr::Equal(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Equal, &mut irs, &mut res_slot_id);
            }
            Expr::Greater(ref left, ref right, _) => {
                self.bin_op_template(
                    left,
                    right,
                    BinOperator::Greater,
                    &mut irs,
                    &mut res_slot_id,
                );
            }
            Expr::Less(ref left, ref right, _) => {
                self.bin_op_template(left, right, BinOperator::Less, &mut irs, &mut res_slot_id);
            }
            Expr::GreaterEqual(ref left, ref right, _) => {
                self.bin_op_template(
                    left,
                    right,
                    BinOperator::GreaterEqual,
                    &mut irs,
                    &mut res_slot_id,
                );
            }
            Expr::LessEqual(ref left, ref right, _) => {
                self.bin_op_template(
                    left,
                    right,
                    BinOperator::LessEqual,
                    &mut irs,
                    &mut res_slot_id,
                );
            }
            Expr::ConstChar(c, _) => {
                let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                s.push(Value::Const(super::Const::Char(c)));
                let ret_slot = self.get_next_slot_id();
                self.slot_registry.insert(ret_slot, s);

                irs.push(HIRInst::Store {
                    from: Value::Const(super::Const::Char(c)),
                    to: ret_slot,
                });

                res_slot_id = ret_slot;
            }
            Expr::ConstNum(x, _) => {
                if Self::can_be_int(x) {
                    let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                    s.push(Value::Const(hir::Const::Int(x as i64)));
                    let ret_slot = self.get_next_slot_id();
                    self.slot_registry.insert(ret_slot, s);

                    irs.push(HIRInst::Store {
                        from: Value::Const(super::Const::Int(x as i64)),
                        to: ret_slot,
                    });

                    res_slot_id = ret_slot;
                } else {
                    let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                    s.push(Value::Const(hir::Const::Float(hir::FloatKey(x))));
                    let ret_slot = self.get_next_slot_id();
                    self.slot_registry.insert(ret_slot, s);

                    irs.push(HIRInst::Store {
                        from: Value::Const(super::Const::Float(hir::FloatKey(x))),
                        to: ret_slot,
                    });

                    res_slot_id = ret_slot;
                }
            }
            Expr::Var(ref name, ref var_type) => {
                let id = self.find_var_id(name);
                let sym = match self.var_registry.get(&id) {
                    None => panic!(""),
                    Some(p) => p,
                };
                sym.obj_id
                    .set_near_use(&mut self.slot_registry, self.hir.len());
                res_slot_id = sym.obj_id;
            }
            Expr::FuncCall(ref name, ref argcs, _, _) => {
                let func_sym = self.find_func_sym(name);

                for i in 0..argcs.len() {
                    let arg_slot_id = self.gen_expr_ir(&argcs[i]);

                    self.hir.push(HIR::Inst(HIRInst::Bind {
                        var: func_sym.param_id[i],
                        obj: arg_slot_id,
                    }));
                }
                let new_obj_id = self.get_next_obj_id();
                irs.push(HIRInst::Call {
                    id: func_sym.id,
                    ret: func_sym.ret_obj_id,
                });

                func_sym
                    .ret_obj_id
                    .set_near_use(&mut self.slot_registry, self.hir.len());
                self.func_registry.insert(func_sym.id, func_sym.clone());
                res_slot_id = func_sym.ret_obj_id;
            }
            Expr::Not(ref expr, _) => {
                let expr_slot_id = self.gen_expr_ir(expr);
                let new_obj_id = self.get_next_obj_id();

                let dst_slot_id = self.get_next_slot_id();
                dst_slot_id.push(&mut self.slot_registry, hir::Value::Obj(new_obj_id));
                irs.push(HIRInst::Store {
                    from: hir::Value::Obj(new_obj_id),
                    to: dst_slot_id,
                });

                irs.push(HIRInst::UnaryOp {
                    op: UnaryOperation::Not,
                    expr: expr_slot_id,
                    dst: dst_slot_id,
                });

                let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                s.push(hir::Value::Obj(new_obj_id));
                let ret_slot = self.get_next_slot_id();
                self.slot_registry.insert(ret_slot, s);
                res_slot_id = ret_slot;
            }
        };

        for i in irs {
            self.hir.push(HIR::Inst(i));
        }

        res_slot_id
    }
    fn gen_block_ir(&mut self, block_id: BlockId, block: &Block, ret_obj: &mut SlotId) {
        for mut i in block.body.clone() {
            match i {
                Stmt::Assign(ref left, ref right) => {
                    let (mut insts, _obj) = self.gen_assign_inst(left, right);
                    for inst in insts.drain(..) {
                        self.hir.push(HIR::Inst(inst));
                    }
                }
                Stmt::For(ref itor, ref start, ref end, ref step, ref inner_block, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_for_hir(itor, start, end, step, inner_block, ret_obj);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::While(ref cond, ref inner_block, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_while_hir(cond, inner_block, ret_obj);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::If(ref cond, ref inner_block, ref mut elifs, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_if_hir(cond, inner_block, &mut elifs.clone(), ret_obj);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Func(ref name, ref argcs, ref var_type, ref _block, scope_id) => {
                    self.gen_func_hir(name, argcs, var_type, block);
                }
                Stmt::Call(ref name, ref argcs, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    let insts = self.gen_call_inst(name, argcs);
                    for inst in insts.into_iter() {
                        self.hir.push(HIR::Inst(inst));
                    }
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Return(ref ret_expr) => {
                    self.gen_return_hir(block_id, ret_expr, ret_obj);
                }
                Stmt::Default => {}
            }
        }

        // After finishing this block, emit its label and instructions,
        // then flush any nested blocks generated while building it.
    }

    fn gen_assign_inst(
        &mut self,
        left: &Rc<RefCell<Expr>>,
        right: &Rc<RefCell<Expr>>,
    ) -> (Vec<HIRInst>, SlotId) {
        let mut res = Vec::<HIRInst>::new();
        let value_slot_id = self.gen_expr_ir(right);

        // Increment reference for the slot that was stored
        res.push(HIRInst::IncRef { obj: value_slot_id });

        let new_sym: HirVarSymbol;
        let id: VarId;
        let binding = left.borrow();
        let name = match &*binding {
            &Expr::Var(ref n, _) => n,
            _ => panic!("Assignment left side must be a variable"),
        };
        match self.ast_symbol_table.find_symbol(name) {
            None => panic!("Assignment left side must be a variable"),
            Some(sym) => {
                id = VarId(sym.id);
                new_sym = HirVarSymbol::new(sym.clone(), value_slot_id);

                let mut slot = match self.slot_registry.get(&value_slot_id) {
                    None => panic!(""),
                    Some(x) => x.clone(),
                };
                if sym.level > slot.cur_leve_id {
                    slot.cur_leve_id = sym.level;
                    slot.escape();
                }
                slot.set_near_use(self.hir.len());
                self.slot_registry.insert(value_slot_id, slot);
            }
        }

        let old_obj_id = self.var_registry.get(&id).map(|s| s.obj_id);

        res.push(HIRInst::Bind {
            var: id,
            obj: value_slot_id,
        });

        self.var_registry.insert(id, new_sym);

        if let Some(old) = old_obj_id {
            res.push(HIRInst::DecRef { obj: old });
        }
        value_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());

        (res, value_slot_id)
    }
    fn gen_assign_hir(&mut self, left: &Rc<RefCell<Expr>>, right: &Rc<RefCell<Expr>>) -> SlotId {
        let (res, obj_id) = self.gen_assign_inst(left, right);

        for inst in res {
            self.hir.push(HIR::Inst(inst));
        }

        obj_id
    }
    fn gen_if_hir(
        &mut self,
        cond: &Rc<RefCell<Expr>>,
        block: &Block,
        elifs: &mut Vec<(Rc<RefCell<Expr>>, Block)>,
        ret_obj: &mut SlotId,
    ) {
        elifs.insert(0, (cond.clone(), block.clone()));
        self.ast_symbol_table.into_new_scope();
        let mut block_ids = Vec::<BlockId>::new();

        for _ in 0..elifs.len() {
            block_ids.push(self.new_block());
            block_ids.push(self.new_block());
        }
        block_ids.push(self.new_merge_block()); //merge
        let merge_id = block_ids[block_ids.len() - 1];

        let mut i = 0;
        let mut elif_idx = 0;
        while i < block_ids.len() - 1 {
            let br_block_id = block_ids[i];
            self.emit_block(br_block_id);
            i += 1;

            let cond_id = self.gen_expr_ir(&elifs[elif_idx].0);
            cond_id.set_near_use(&mut self.slot_registry, self.hir.len());

            let then_block_id = block_ids[i];
            let else_block_id = block_ids[i + 1];
            i += 1;

            self.hir.push(HIR::Inst(HIRInst::Br {
                cond: cond_id,
                then_block: then_block_id,
                else_block: else_block_id,
            }));

            self.emit_block(then_block_id);
            self.gen_block_ir(then_block_id, &elifs[elif_idx].1, ret_obj);
            elif_idx += 1;
            self.hir.push(HIR::Inst(HIRInst::Jmp { target: merge_id }));
        }

        self.emit_block(merge_id);

        self.ast_symbol_table.ret_to_parent_scope();
        // Defer emitting these blocks until the parent block is emitted to preserve source order.
    }
    fn gen_for_hir(
        &mut self,
        itor: &Rc<RefCell<Expr>>,
        start: &Rc<RefCell<Expr>>,
        end: &Rc<RefCell<Expr>>,
        step: &Rc<RefCell<Expr>>,
        block: &Block,
        ret_obj: &mut SlotId,
    ) {
        let init_block_id = self.new_block();
        let cond_block_id = self.new_block();
        let body_block_id = self.new_block();
        let merge_block_id = self.new_merge_block();
        // Defer emitting blocks until the parent block is emitted to preserve source order.
        self.emit_block(init_block_id);
        let itor_id = self.gen_assign_hir(itor, start);

        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        self.emit_block(cond_block_id);
        let end_id = self.gen_expr_ir(end);
        let step_id = self.gen_expr_ir(step);

        let cond_id = self.get_next_obj_id();
        let cond_slot_id = self.get_next_slot_id();
        cond_slot_id.push(&mut self.slot_registry, hir::Value::Obj(cond_id));
        let cond_ir = HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Less,
            right: end_id,
            dst: cond_slot_id,
        };
        self.hir.push(HIR::Inst(cond_ir));

        cond_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());

        self.hir.push(HIR::Inst(HIRInst::Br {
            cond: cond_slot_id,
            then_block: body_block_id,
            else_block: merge_block_id,
        }));
        // cond_block_id will be emitted later

        self.emit_block(body_block_id);
        self.gen_block_ir(body_block_id, block, ret_obj);

        self.hir.push(HIR::Inst(HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Add,
            right: step_id,
            dst: itor_id,
        }));
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        // Defer emitting these blocks until the parent block is emitted
        self.emit_block(merge_block_id);

        itor_id.set_near_use(&mut self.slot_registry, self.hir.len());
        end_id.set_near_use(&mut self.slot_registry, self.hir.len());
        step_id.set_near_use(&mut self.slot_registry, self.hir.len());
    }
    fn gen_while_hir(&mut self, cond: &Rc<RefCell<Expr>>, block: &Block, ret_obj: &mut SlotId) {
        let br_block_id = self.new_block();
        self.emit_block(br_block_id);
        let cond_id = self.gen_expr_ir(cond);
        cond_id.set_near_use(&mut self.slot_registry, self.hir.len());

        let body_block_id = self.new_block();
        let merge_block_id = self.new_merge_block();

        self.hir.push(HIR::Inst(HIRInst::Br {
            cond: cond_id,
            then_block: body_block_id,
            else_block: merge_block_id,
        }));
        // Defer emitting blocks until the parent block is emitted to preserve source order.
        self.emit_block(body_block_id);
        self.gen_block_ir(body_block_id, block, ret_obj);
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: br_block_id,
        }));
        self.emit_block(merge_block_id);
    }
    fn gen_func_hir(
        &mut self,
        name: &String,
        argcs: &Vec<Argc>,
        var_type: &HashSet<VarType>,
        block: &Block,
    ) {
        // Ensure expressions in the function body have their variable type sets
        // populated from the AST symbol table before generating HIR.
        let ast_fn_sym = match self.ast_symbol_table.find_symbol(name) {
            None => panic!(""),
            Some(sym) => sym,
        };
        self.hir.push(HIR::Func(FuncId(ast_fn_sym.id)));
        let slot_id = self.get_next_slot_id();
        let mut fn_sym = HirFuncSymbol {
            ty_set: ast_fn_sym.its_type,
            ret_obj_id: slot_id,
            id: FuncId(ast_fn_sym.id),
            param_id: Vec::new(),
        };
        self.slot_registry
            .insert(slot_id, ObjSlot::new(self.ast_symbol_table.get_level()));

        let entry_block_id = self.new_block();
        self.emit_block(entry_block_id);
        let body_block_id = self.new_block();

        for i in 0..argcs.len() {
            let Argc {
                ref is_ref,
                ref arg_type,
                ref var_name,
            } = argcs[i];

            let param_id = self.find_var_id(var_name);
            let mut ty_set = HashSet::<VarType>::new();
            ty_set.insert(arg_type.clone());

            let i_slot_id = self.get_next_slot_id();
            i_slot_id.push(
                &mut self.slot_registry,
                hir::Value::Obj(ObjId {
                    id: i as i32,
                    home_level_id: self.ast_symbol_table.get_level(),
                    cur_leve_id: self.ast_symbol_table.get_level(),
                }),
            );
            let hir_sym = HirVarSymbol {
                ty_set,
                mutability: true,
                storage: StorageClass::Param,
                obj_id: i_slot_id,
            };

            self.var_registry.insert(param_id, hir_sym);
            fn_sym.param_id.push(param_id);
        }

        self.func_registry.insert(fn_sym.id, fn_sym.clone());

        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: body_block_id,
        }));
        self.emit_block(body_block_id);

        self.gen_block_ir(body_block_id, block, &mut fn_sym.ret_obj_id);
        self.func_registry.insert(fn_sym.id, fn_sym.clone());
        // gen_block_ir will emit the body block and any nested blocks in correct order
    }
    fn gen_call_hir(&mut self, name: &String, argcs: &Vec<Rc<RefCell<Expr>>>) {
        let func_sym = self.find_func_sym(name);

        for i in 0..argcs.len() {
            let arg_slot_id = self.gen_expr_ir(&argcs[i]);
            self.hir.push(HIR::Inst(HIRInst::Bind {
                var: func_sym.param_id[i],
                obj: arg_slot_id,
            }));
        }

        self.hir.push(HIR::Inst(HIRInst::Call {
            id: func_sym.id,
            ret: func_sym.ret_obj_id,
        }));
        func_sym
            .ret_obj_id
            .set_near_use(&mut self.slot_registry, self.hir.len());
        self.func_registry.insert(func_sym.id, func_sym);
    }
    fn gen_call_inst(&mut self, name: &String, argcs: &Vec<Rc<RefCell<Expr>>>) -> Vec<HIRInst> {
        let res = Vec::<HIRInst>::new();
        let func_sym = self.find_func_sym(name);

        for i in 0..argcs.len() {
            let arg_slot_id = self.gen_expr_ir(&argcs[i]);
            self.hir.push(HIR::Inst(HIRInst::Bind {
                var: func_sym.param_id[i],
                obj: arg_slot_id,
            }));
        }

        self.hir.push(HIR::Inst(HIRInst::Call {
            id: func_sym.id,
            ret: func_sym.ret_obj_id,
        }));
        func_sym
            .ret_obj_id
            .set_near_use(&mut self.slot_registry, self.hir.len());
        self.func_registry.insert(func_sym.id, func_sym);

        res
    }
    fn gen_return_hir(
        &mut self,
        block_id: BlockId,
        ret_expr: &Rc<RefCell<Expr>>,
        ret_obj: &mut SlotId,
    ) {
        let value_slot_id = self.gen_expr_ir(ret_expr);

        self.hir.push(HIR::Inst(HIRInst::Ret {
            ret_obj: value_slot_id,
        }));
        value_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());
        value_slot_id.escape(&mut self.slot_registry);

        // push returned object id into ret_obj slot
        ret_obj.append(&mut self.slot_registry, &value_slot_id);
    }
    fn gen_stmt_hir(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign(ref left, ref right) => {
                self.gen_assign_hir(left, right);
            }
            Stmt::For(ref itor, ref start, ref end, ref step, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_for_hir(itor, start, end, step, block, &mut SlotId { id: -1 });
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::While(ref cond, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                let mut tmp = ObjSlot::new(self.ast_symbol_table.get_level());
                self.gen_while_hir(cond, block, &mut SlotId { id: -1 });
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::If(ref cond, ref block, ref elifs, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                let mut tmp = ObjSlot::new(self.ast_symbol_table.get_level());
                self.gen_if_hir(cond, block, &mut elifs.clone(), &mut SlotId { id: -1 });
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::Func(ref name, ref argcs, ref var_type, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_func_hir(name, argcs, var_type, block);
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::Call(ref name, ref argcs, scope_id) => {
                self.gen_call_hir(name, argcs);
            }
            Stmt::Return(ref ret_expr) => panic!("cannot generate return stmt hir in gen_stmt_hir"),
            Stmt::Default => panic!("unkonwn stmt"),
        }
    }

    pub(crate) fn gen_hir(&mut self) -> Vec<HIR> {
        for i in 0..self.stmts.len() {
            self.gen_stmt_hir(self.stmts[i].clone());
        }

        self.analyze_lifetime();

        self.hir.clone()
    }
}
