use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
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
    pub(crate) fn raw_id(self) -> i32 {
        self.id
    }

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
        let mut res = HirGenerator {
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
            next_slotid: SlotId { id: 0, temp: false },

            var_registry: HashMap::<VarId, HirVarSymbol>::new(),
            obj_registry: HashMap::<i32, ObjId>::new(),
            func_registry: HashMap::<FuncId, HirFuncSymbol>::new(),
            block_registry: HashMap::<BlockId, Vec<HIRInst>>::new(),
            slot_registry: HashMap::<SlotId, ObjSlot>::new(),
            var_to_slot: HashMap::new(),

            ast_symbol_table,

            hir: Vec::<HIR>::new(),
        };

        let ast_sym = res
            .ast_symbol_table
            .find_symbol(&"write".to_string())
            .expect("");
        let param_id = ast_sym.args[0].id;
        res.func_registry.insert(
            FuncId(ast_sym.id),
            HirFuncSymbol {
                ty_set: HashSet::new(),
                id: hir::FuncId(ast_sym.id),
                param_id: [VarId(param_id)].to_vec(),
                is_main: false,
                name: "write".to_string(),
            },
        );

        res
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

    fn get_next_temp_slot_id(&mut self) -> SlotId {
        let id = self.next_slotid;
        self.next_slotid += 1;
        let temp_id = SlotId {
            id: id.id,
            temp: true,
        };
        self.slot_registry
            .insert(temp_id, ObjSlot::new(self.ast_symbol_table.get_level()));
        temp_id
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
        if !self.hir.is_empty() {
            if let HIR::Inst(x) = &self.hir[self.hir.len() - 1] {
                match x {
                    HIRInst::Br { .. } | HIRInst::Jmp { .. } | HIRInst::Ret { .. } => {
                        // 已经是分支/跳转/返回：不做任何事
                    }
                    _ => {
                        self.hir.push(HIR::Inst(HIRInst::Jmp { target: id }));
                    }
                }
            } else if let HIR::FuncLabel(_) = &self.hir[self.hir.len() - 1] {
            } else {
                self.hir.push(HIR::Inst(HIRInst::Jmp { target: id }));
            }
        } else {
            // 若当前没有任何指令，直接插入跳转到新块的指令
            self.hir.push(HIR::Inst(HIRInst::Jmp { target: id }));
        }

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

                irs.push(HIRInst::New {
                    obj_type: VarType::Char,
                    val: c.to_string(),
                    dst: ret_slot,
                });

                res_slot_id = ret_slot;
            }
            Expr::ConstNum(x, _) => {
                if Self::can_be_int(x) {
                    let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                    s.push(Value::Const(hir::Const::Int(x as i64)));
                    let ret_slot = self.get_next_slot_id();
                    self.slot_registry.insert(ret_slot, s);

                    irs.push(HIRInst::New {
                        obj_type: VarType::Int,
                        val: x.to_string(),
                        dst: ret_slot,
                    });

                    res_slot_id = ret_slot;
                } else {
                    let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                    s.push(Value::Const(hir::Const::Float(hir::FloatKey(x))));
                    let ret_slot = self.get_next_slot_id();
                    self.slot_registry.insert(ret_slot, s);

                    irs.push(HIRInst::New {
                        obj_type: VarType::Float,
                        val: x.to_string(),
                        dst: ret_slot,
                    });

                    res_slot_id = ret_slot;
                }
            }
            Expr::ConstStr(ref str, _) => {
                let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                s.push(Value::Const(super::Const::String(str.clone())));
                let ret_slot = self.get_next_slot_id();
                self.slot_registry.insert(ret_slot, s);

                irs.push(HIRInst::New {
                    obj_type: VarType::String,
                    val: str.clone(),
                    dst: ret_slot,
                });

                res_slot_id = ret_slot;
            }
            Expr::Var(ref name, _) => {
                let id = self.find_var_id(name);
                let dst_slot = self.get_next_temp_slot_id();

                irs.push(HIRInst::Load {
                    var: id,
                    obj: dst_slot,
                });

                if let Some(sym) = self.ast_symbol_table.find_symbol(name)
                    && sym.is_argc
                {
                    dst_slot.escape(&mut self.slot_registry);
                }
                if let Some(sym) = self.var_registry.get(&id) {
                    sym.obj_id
                        .set_near_use(&mut self.slot_registry, self.hir.len());
                }

                dst_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                res_slot_id = dst_slot;
            }
            Expr::FuncCall(ref name, ref argcs, _, _) => {
                let func_sym = self.find_func_sym(name);

                let mut call_args: Vec<SlotId> = Vec::new();
                for i in 0..argcs.len() {
                    let arg_slot_id = self.gen_expr_ir(&argcs[i]);
                    // 保持引用计数语义：调用前增加引用
                    self.hir
                        .push(HIR::Inst(HIRInst::IncRef { obj: arg_slot_id }));
                    call_args.push(arg_slot_id);
                }

                let ret_slot = self.get_next_slot_id();
                irs.push(HIRInst::Call {
                    id: func_sym.id,
                    args: call_args,
                    dst: ret_slot,
                });

                ret_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                self.func_registry.insert(func_sym.id, func_sym.clone());
                res_slot_id = ret_slot;
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
            Expr::Array(ref elems, _, _) => {
                let mut elem_slots = Vec::new();
                for e in elems {
                    let elem_slot = self.gen_expr_ir(e);
                    elem_slot.escape(&mut self.slot_registry);
                    elem_slots.push(elem_slot);
                }

                let new_obj_id = self.get_next_obj_id();
                let dst_slot_id = self.get_next_slot_id();
                dst_slot_id.push(&mut self.slot_registry, hir::Value::Obj(new_obj_id));

                irs.push(HIRInst::ArrayNew {
                    elems: elem_slots.clone(),
                    dst: dst_slot_id,
                });
                dst_slot_id.escape(&mut self.slot_registry);

                res_slot_id = dst_slot_id;
            }
            Expr::ArrayElem(ref name, ref idx, _) => {
                let arr_var_id = self.find_var_id(name);
                let arr_slot = self.get_next_temp_slot_id();
                irs.push(HIRInst::Load {
                    var: arr_var_id,
                    obj: arr_slot,
                });

                let idx_slot = self.gen_expr_ir(idx);

                let ret_slot = self.get_next_slot_id();
                irs.push(HIRInst::ArrayGet {
                    arr: arr_slot,
                    idx: idx_slot,
                    dst: ret_slot,
                });

                arr_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                idx_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                ret_slot.escape(&mut self.slot_registry);

                if let Some(sym) = self.var_registry.get(&arr_var_id) {
                    sym.obj_id
                        .set_near_use(&mut self.slot_registry, self.hir.len());
                };

                res_slot_id = ret_slot;
            }
        };

        for i in irs {
            self.hir.push(HIR::Inst(i));
        }

        res_slot_id
    }
    fn gen_block_ir(&mut self, block_id: BlockId, block: &Block) {
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
                    self.gen_for_hir(itor, start, end, step, inner_block);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::While(ref cond, ref inner_block, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_while_hir(cond, inner_block);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::If(ref cond, ref inner_block, ref mut elifs, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_if_hir(cond, inner_block, &mut elifs.clone());
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Func(ref name, ref argcs, ref var_type, ref _block, _) => {
                    self.gen_func_hir(name, argcs, var_type, block);
                }
                Stmt::Call(ref name, ref argcs, _scope_id) => {
                    let insts = self.gen_call_inst(name, argcs);
                    for inst in insts.into_iter() {
                        self.hir.push(HIR::Inst(inst));
                    }
                }
                Stmt::Return(ref ret_expr) => {
                    self.gen_return_hir(block_id, ret_expr);
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

        match &*left.borrow() {
            Expr::Var(name, _) => {
                let sym = self
                    .ast_symbol_table
                    .find_symbol(name)
                    .expect("Assignment left side must be a variable");
                let id = VarId(sym.id);
                let new_sym = HirVarSymbol::new(sym.clone(), value_slot_id);

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

                let old_obj_id = self.var_registry.get(&id).map(|s| s.obj_id);

                res.push(HIRInst::Bind {
                    var: id,
                    obj: value_slot_id,
                });

                if let Some(sym) = self.ast_symbol_table.find_symbol(name)
                    && (sym.level < self.ast_symbol_table.get_level() || sym.is_argc)
                {
                    value_slot_id.escape(&mut self.slot_registry);
                }

                self.var_registry.insert(id, new_sym);

                // Do not auto-dec old binding here; lifetime is handled by ownership of new value.
                value_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());
            }
            Expr::ArrayElem(name, idx, _) => {
                let arr_var_id = self.find_var_id(name);
                let arr_slot = self.get_next_temp_slot_id();
                res.push(HIRInst::Load {
                    var: arr_var_id,
                    obj: arr_slot,
                });

                let idx_slot = self.gen_expr_ir(idx);
                res.push(HIRInst::ArraySet {
                    arr: arr_slot,
                    idx: idx_slot,
                    src: value_slot_id,
                });

                arr_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                idx_slot.set_near_use(&mut self.slot_registry, self.hir.len());
                value_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());
                value_slot_id.escape(&mut self.slot_registry);

                if let Some(sym) = self.var_registry.get(&arr_var_id) {
                    sym.obj_id
                        .set_near_use(&mut self.slot_registry, self.hir.len());
                }
            }
            _ => panic!("Assignment left side must be a variable"),
        }

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
            self.gen_block_ir(then_block_id, &elifs[elif_idx].1);
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
    ) {
        let init_block_id = self.new_block();
        let cond_block_id = self.new_block();
        let body_block_id = self.new_block();
        let merge_block_id = self.new_merge_block();
        // Defer emitting blocks until the parent block is emitted to preserve source order.
        self.emit_block(init_block_id);
        let itor_id = self.gen_assign_hir(itor, start);
        let end_id = self.gen_expr_ir(end);
        let step_id = self.gen_expr_ir(step);

        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        self.emit_block(cond_block_id);

        let cond_id = self.get_next_obj_id();
        let cond_slot_id = self.get_next_slot_id();
        cond_slot_id.escape(&mut self.slot_registry);
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
        self.gen_block_ir(body_block_id, block);

        self.hir.push(HIR::Inst(HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Add,
            right: step_id,
            dst: itor_id,
        }));

        // Re-bind the loop iterator variable to the updated slot so subsequent
        // iterations see the new value rather than the old snapshot.
        let itor_var = match &*itor.borrow() {
            Expr::Var(name, _) => {
                let sym = self
                    .ast_symbol_table
                    .find_symbol(name)
                    .expect("loop iterator symbol missing");
                VarId(sym.id)
            }
            _ => panic!("iterator is not a variable"),
        };

        self.hir.push(HIR::Inst(HIRInst::Bind {
            var: itor_var,
            obj: itor_id,
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
    fn gen_while_hir(&mut self, cond: &Rc<RefCell<Expr>>, block: &Block) {
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
        self.gen_block_ir(body_block_id, block);
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: br_block_id,
        }));
        self.emit_block(merge_block_id);
    }
    fn gen_func_hir(
        &mut self,
        name: &String,
        argcs: &Vec<Argc>,
        #[allow(unused)] var_type: &HashSet<VarType>,
        block: &Block,
    ) {
        // Ensure expressions in the function body have their variable type sets
        // populated from the AST symbol table before generating HIR.
        let ast_fn_sym = match self.ast_symbol_table.find_symbol(name) {
            None => panic!(""),
            Some(sym) => sym,
        };
        self.hir.push(HIR::FuncLabel(hir::FuncId(ast_fn_sym.id)));
        let mut fn_sym = HirFuncSymbol {
            ty_set: ast_fn_sym.its_type,
            id: FuncId(ast_fn_sym.id),
            param_id: Vec::new(),
            is_main: name == "main",
            name: name.clone(),
        };

        let entry_block_id = self.new_block();
        self.emit_block(entry_block_id);
        let body_block_id = self.new_block();

        for i in 0..argcs.len() {
            let Argc {
                #[allow(unused)]
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

            self.var_to_slot.insert(param_id, i_slot_id);
            self.var_registry.insert(param_id, hir_sym);
            fn_sym.param_id.push(param_id);

            /*self.hir.push(HIR::Inst(HIRInst::Load {
                var: param_id,
                obj: i_slot_id,
            }));*/

            i_slot_id.escape(&mut self.slot_registry);
        }

        self.func_registry.insert(fn_sym.id, fn_sym.clone());

        let func_body_start = self.hir.len();

        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: body_block_id,
        }));
        self.emit_block(body_block_id);

        self.gen_block_ir(body_block_id, block);
        self.func_registry.insert(fn_sym.id, fn_sym.clone());

        // If function has no explicit return, insert an implicit `Ret` of the first parameter (or null if none).
        let mut has_ret = false;
        let mut idx = func_body_start;
        while idx < self.hir.len() {
            match &self.hir[idx] {
                HIR::FuncLabel(_) => break,
                HIR::Inst(HIRInst::Ret { .. }) => {
                    has_ret = true;
                    break;
                }
                _ => {}
            }
            idx += 1;
        }

        if !has_ret {
            if let Some(first_param) = fn_sym.param_id.first() {
                // load first param into a temp slot and return it
                let ret_slot = self.get_next_temp_slot_id();
                self.hir.push(HIR::Inst(HIRInst::Load {
                    var: *first_param,
                    obj: ret_slot,
                }));
                self.hir.push(HIR::Inst(HIRInst::Ret { ret_obj: ret_slot }));
            } else {
                // no params: synthesize a zero literal return to avoid falling through
                let mut s = ObjSlot::new(self.ast_symbol_table.get_level());
                s.push(Value::Const(hir::Const::Int(0)));
                let ret_slot = self.get_next_slot_id();
                self.slot_registry.insert(ret_slot, s);

                self.hir.push(HIR::Inst(HIRInst::New {
                    obj_type: VarType::Int,
                    val: "0".to_string(),
                    dst: ret_slot,
                }));
                self.hir.push(HIR::Inst(HIRInst::Ret { ret_obj: ret_slot }));
            }
        }

        /*for i in 0..argcs.len() {
            let Argc {
                #[allow(unused)]
                ref is_ref,
                ref arg_type,
                ref var_name,
            } = argcs[i];

            let param_id = self.find_var_id(var_name);

            let i_slot_id = self.var_registry.get(&param_id).expect("");

            self.hir.push(HIR::Inst(HIRInst::DecRef {
                obj: i_slot_id.obj_id,
            }));
        }*/
        // gen_block_ir will emit the body block and any nested blocks in correct order
    }
    fn gen_call_hir(&mut self, name: &String, argcs: &Vec<Rc<RefCell<Expr>>>) {
        let func_sym = self.find_func_sym(name);
        let mut param_ids = Vec::new();

        let dst_slot = self.get_next_slot_id();

        let mut call_args: Vec<SlotId> = Vec::new();
        for i in 0..argcs.len() {
            let arg_slot_id = self.gen_expr_ir(&argcs[i]);
            self.hir
                .push(HIR::Inst(HIRInst::IncRef { obj: arg_slot_id }));
            call_args.push(arg_slot_id);
            param_ids.push(arg_slot_id);
        }

        self.hir.push(HIR::Inst(HIRInst::Call {
            id: func_sym.id,
            args: call_args,
            dst: dst_slot,
        }));
        dst_slot.set_near_use(&mut self.slot_registry, self.hir.len());

        for i in param_ids {
            i.set_near_use(&mut self.slot_registry, self.hir.len());
        }

        self.func_registry.insert(func_sym.id, func_sym);
    }
    fn gen_call_inst(&mut self, name: &String, argcs: &Vec<Rc<RefCell<Expr>>>) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();
        let func_sym = self.find_func_sym(name);
        let mut param_ids = Vec::new();

        let ret_slot = self.get_next_slot_id();

        if argcs.len() >= 1 {
            let mut call_args: Vec<SlotId> = Vec::new();
            for i in 0..argcs.len() {
                let arg_slot_id = self.gen_expr_ir(&argcs[i]);
                self.hir
                    .push(HIR::Inst(HIRInst::IncRef { obj: arg_slot_id }));
                call_args.push(arg_slot_id);
                param_ids.push(arg_slot_id);
            }

            self.hir.push(HIR::Inst(HIRInst::Call {
                id: func_sym.id,
                args: call_args,
                dst: ret_slot,
            }));
        } else {
            self.hir.push(HIR::Inst(HIRInst::Call {
                id: func_sym.id,
                args: Vec::new(),
                dst: ret_slot,
            }));
        }
        ret_slot.set_near_use(&mut self.slot_registry, self.hir.len());
        self.func_registry.insert(func_sym.id, func_sym);

        for i in param_ids {
            i.set_near_use(&mut self.slot_registry, self.hir.len());
        }

        res
    }
    fn gen_return_hir(&mut self, #[allow(unused)] block_id: BlockId, ret_expr: &Rc<RefCell<Expr>>) {
        let value_slot_id = self.gen_expr_ir(ret_expr);

        self.hir.push(HIR::Inst(HIRInst::Ret {
            ret_obj: value_slot_id,
        }));
        value_slot_id.set_near_use(&mut self.slot_registry, self.hir.len());
        value_slot_id.escape(&mut self.slot_registry);
    }
    fn gen_stmt_hir(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign(ref left, ref right) => {
                self.gen_assign_hir(left, right);
            }
            Stmt::For(ref itor, ref start, ref end, ref step, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_for_hir(itor, start, end, step, block);
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::While(ref cond, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_while_hir(cond, block);
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::If(ref cond, ref block, ref elifs, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_if_hir(cond, block, &mut elifs.clone());
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::Func(ref name, ref argcs, ref var_type, ref block, scope_id) => {
                self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                self.gen_func_hir(name, argcs, var_type, block);
                self.ast_symbol_table.ret_to_parent_scope();
            }
            Stmt::Call(ref name, ref argcs, _) => {
                self.gen_call_hir(name, argcs);
            }
            Stmt::Return(_) => panic!("cannot generate return stmt hir in gen_stmt_hir"),
            Stmt::Default => panic!("unkonwn stmt"),
        }
    }

    fn into_file(&mut self) {
        fs::write("./build/hir.txt", "").unwrap();
        let mut hir_file = OpenOptions::new()
            .append(true)
            .open("./build/hir.txt")
            .unwrap();
        for i in self.hir.clone() {
            hir_file.write(&i.to_string().as_bytes()).unwrap();
            hir_file.write("\n".as_bytes()).unwrap();
        }
    }

    fn checker(&mut self) {
        let mut i = 0;
        while i < self.hir.len() {
            if let HIR::Block(_) = self.hir[i] {
                let mut flag = true;
                let block_start = i;
                i += 1;

                while flag && i < self.hir.len() {
                    match self.hir[i] {
                        HIR::Block(_) | HIR::FuncLabel(_) => {
                            if i == block_start + 1 {
                                // empty block: insert unreachable
                                self.hir.insert(i, HIR::Inst(HIRInst::Unreachable));
                            } else if let HIR::Inst(x) = &self.hir[i - 1] {
                                match x {
                                    HIRInst::Br { .. }
                                    | HIRInst::Jmp { .. }
                                    | HIRInst::Ret { .. }
                                    | HIRInst::Unreachable => {
                                        break;
                                    }
                                    _ => {
                                        self.hir.insert(i, HIR::Inst(HIRInst::Unreachable));
                                    }
                                }
                            }

                            flag = false;
                        }
                        _ => i += 1,
                    }
                }
            }

            if i < self.hir.len() {
                i += 1;
            }
        }

        i = 0;
        while i < self.hir.len() {
            fn wait_for_swap(ir: &HIR) -> bool {
                match ir {
                    HIR::Inst(x) => match x {
                        HIRInst::Br { .. }
                        | HIRInst::Jmp { .. }
                        | HIRInst::Ret { .. }
                        | HIRInst::Unreachable => true,
                        _ => false,
                    },
                    _ => false,
                }
            }
            if wait_for_swap(&self.hir[i]) {
                enum Ret {
                    Erase,
                    Swap,
                    Con,
                }
                fn can_swap(ir: &HIR) -> Ret {
                    match ir {
                        HIR::Inst(x) => match x {
                            HIRInst::Br { .. }
                            | HIRInst::Jmp { .. }
                            | HIRInst::Ret { .. }
                            | HIRInst::Unreachable => Ret::Erase,
                            _ => Ret::Swap,
                        },
                        _ => Ret::Con,
                    }
                }
                while i < self.hir.len() - 1 {
                    match can_swap(&self.hir[i + 1]) {
                        Ret::Swap => {
                            self.hir.swap(i, i + 1);
                            self.into_file();
                        }
                        Ret::Erase => {
                            self.hir.remove(i + 1);
                            self.into_file();
                            i -= 1;
                        }
                        Ret::Con => break,
                    }

                    i += 1;
                }
            }

            i += 1;
        }

        // For each Delete: if the slot did NOT escape, remove related IncRef/DecRef.
        // Otherwise move IncRef/DecRef before Delete (so they act on live object).
        // Additionally: if a slot has any IncRef/DecRef at all, skip inserting Delete for it.
        let mut idx = 0;
        while idx < self.hir.len() {
            if let HIR::Inst(HIRInst::Delete { dst }) = self.hir[idx] {
                // Skip Delete if there is an IncRef/DecRef for this slot in the same function.
                let mut has_ref_ops = false;
                for scan in 0..self.hir.len() {
                    match &self.hir[scan] {
                        HIR::FuncLabel(_) => {
                            if scan > idx {
                                break;
                            }
                        }
                        HIR::Inst(HIRInst::IncRef { obj }) | HIR::Inst(HIRInst::DecRef { obj })
                            if *obj == dst =>
                        {
                            has_ref_ops = true;
                            break;
                        }
                        _ => {}
                    }
                }

                if has_ref_ops {
                    self.hir.remove(idx);
                    continue;
                }

                // check slot escape info
                if let Some(slot_info) = self.slot_registry.get(&dst) {
                    if !slot_info.escape {
                        // remove any IncRef/DecRef for this dst in the same basic block
                        // scan backwards
                        let mut back = idx;
                        while back > 0 {
                            match &self.hir[back - 1] {
                                HIR::Block(_) | HIR::FuncLabel(_) => break,
                                HIR::Inst(x) => match x {
                                    HIRInst::IncRef { obj } | HIRInst::DecRef { obj }
                                        if *obj == dst =>
                                    {
                                        self.hir.remove(back - 1);
                                        // we've removed one element before idx, shift idx left
                                        idx -= 1;
                                        back -= 1;
                                        continue;
                                    }
                                    _ => {}
                                },
                            }

                            back -= 1;
                        }

                        // scan forwards
                        let mut scan = idx + 1;
                        while scan < self.hir.len() {
                            match &self.hir[scan] {
                                HIR::Block(_) | HIR::FuncLabel(_) => break,
                                HIR::Inst(x) => match x {
                                    HIRInst::IncRef { obj } | HIRInst::DecRef { obj }
                                        if *obj == dst =>
                                    {
                                        self.hir.remove(scan);
                                        // do not advance scan, new element now at `scan`
                                        continue;
                                    }
                                    _ => {}
                                },
                            }

                            scan += 1;
                        }
                    } else {
                        // slot escaped: keep refs, but ensure they are before Delete
                        let mut scan = idx + 1;
                        while scan < self.hir.len() {
                            match &self.hir[scan] {
                                HIR::Block(_) | HIR::FuncLabel(_) => break,
                                HIR::Inst(x) => match x {
                                    HIRInst::IncRef { obj } | HIRInst::DecRef { obj }
                                        if *obj == dst =>
                                    {
                                        let inst = self.hir.remove(scan);
                                        self.hir.insert(idx, inst);
                                        idx += 1;
                                        scan = idx + 1;
                                        continue;
                                    }
                                    _ => {}
                                },
                            }

                            scan += 1;
                        }
                    }
                }
            }

            idx += 1;
        }

        // Ensure every basic block ends with a terminator.
        // Prefer jumping to the next block within the same function; if none,
        // insert a sensible return: for main return int 0; otherwise return the
        // first parameter if it exists, else return int 0.
        let mut idx_block = 0;
        let mut cur_func: Option<FuncId> = None;
        while idx_block < self.hir.len() {
            if let HIR::FuncLabel(fid) = self.hir[idx_block] {
                cur_func = Some(fid);
            }

            if let HIR::Block(block_id) = self.hir[idx_block] {
                let mut j = idx_block + 1;
                let mut has_term = false;
                while j < self.hir.len() {
                    match &self.hir[j] {
                        HIR::Block(_) | HIR::FuncLabel(_) => break,
                        HIR::Inst(inst) => match inst {
                            HIRInst::Br { .. }
                            | HIRInst::Jmp { .. }
                            | HIRInst::Ret { .. }
                            | HIRInst::Unreachable => {
                                has_term = true;
                                break;
                            }
                            _ => {}
                        },
                    }
                    j += 1;
                }

                if !has_term {
                    // find the next block in the same function
                    let mut k = j;
                    let mut next_block: Option<BlockId> = None;
                    while k < self.hir.len() {
                        match &self.hir[k] {
                            HIR::FuncLabel(_) => break,
                            HIR::Block(bid) => {
                                next_block = Some(*bid);
                                break;
                            }
                            _ => {}
                        }
                        k += 1;
                    }

                    if let Some(target) = next_block {
                        self.hir.insert(j, HIR::Inst(HIRInst::Jmp { target }));
                        idx_block = j + 1;
                        continue;
                    }

                    // No subsequent block: synthesize a return.
                    let is_main = if let Some(fid) = cur_func {
                        if let Some(sym) = self.func_registry.get(&fid) {
                            sym.is_main
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_main {
                        let ret_slot = self.get_next_slot_id();
                        self.hir.push(HIR::Inst(HIRInst::New {
                            obj_type: VarType::Int,
                            val: "0".to_string(),
                            dst: ret_slot,
                        }));
                        self.hir.push(HIR::Inst(HIRInst::Ret { ret_obj: ret_slot }));
                    } else if let Some(fid) = cur_func
                        && let Some(sym) = self.func_registry.get(&fid)
                        && let Some(first_param) = sym.param_id.first()
                    {
                        if let Some(var_sym) = self.var_registry.get(first_param) {
                            self.hir.push(HIR::Inst(HIRInst::Ret {
                                ret_obj: var_sym.obj_id,
                            }));
                        } else {
                            let ret_slot = self.get_next_slot_id();
                            self.hir.push(HIR::Inst(HIRInst::New {
                                obj_type: VarType::Int,
                                val: "0".to_string(),
                                dst: ret_slot,
                            }));
                            self.hir.push(HIR::Inst(HIRInst::Ret { ret_obj: ret_slot }));
                        }
                    } else {
                        let ret_slot = self.get_next_slot_id();
                        self.hir.push(HIR::Inst(HIRInst::New {
                            obj_type: VarType::Int,
                            val: "0".to_string(),
                            dst: ret_slot,
                        }));
                        self.hir.push(HIR::Inst(HIRInst::Ret { ret_obj: ret_slot }));
                    }

                    idx_block = self.hir.len();
                    continue;
                }
            }

            idx_block += 1;
        }

        // Move each Delete into the last-use block and place it before that block's last instruction.
        let mut i = 0;
        while i < self.hir.len() {
            if let HIR::Inst(HIRInst::Delete { dst }) = self.hir[i].clone() {
                // find last index where dst is used within the same function
                let mut last_use = i;
                let mut j = i + 1;
                while j < self.hir.len() {
                    match &self.hir[j] {
                        HIR::FuncLabel(_) => break,
                        HIR::Block(_) => {}
                        HIR::Inst(inst) => {
                            let uses_dst = match inst {
                                HIRInst::Store { to, .. } => *to == dst,
                                HIRInst::ArrayNew { elems, dst: d } => {
                                    elems.iter().any(|e| *e == dst) || *d == dst
                                }
                                HIRInst::ArrayGet { arr, idx, dst: d } => {
                                    *arr == dst || *idx == dst || *d == dst
                                }
                                HIRInst::ArraySet { arr, idx, src } => {
                                    *arr == dst || *idx == dst || *src == dst
                                }
                                HIRInst::Br { cond, .. } => *cond == dst,
                                HIRInst::Call { dst: d, .. } => *d == dst,
                                HIRInst::BinOp {
                                    left,
                                    right,
                                    dst: d,
                                    ..
                                } => *left == dst || *right == dst || *d == dst,
                                HIRInst::UnaryOp { expr, dst: d, .. } => *expr == dst || *d == dst,
                                HIRInst::IncRef { obj } | HIRInst::DecRef { obj } => *obj == dst,
                                HIRInst::Bind { obj, .. } => *obj == dst,
                                HIRInst::Load { obj, .. } => *obj == dst,
                                HIRInst::Ret { ret_obj } => *ret_obj == dst,
                                HIRInst::Delete { dst: d } => *d == dst,
                                _ => false,
                            };

                            if uses_dst {
                                last_use = j;
                            }
                        }
                    }

                    j += 1;
                }

                if last_use > i {
                    // find the block that contains last_use by scanning backward
                    let mut block_start = None;
                    for b in (0..=last_use).rev() {
                        if let HIR::Block(_) = self.hir[b] {
                            block_start = Some(b);
                            break;
                        }
                    }

                    if let Some(bs) = block_start {
                        // find block end (index of next Block or FuncLabel) or end of hir
                        let mut block_end = self.hir.len();
                        for k in (bs + 1)..self.hir.len() {
                            match self.hir[k] {
                                HIR::Block(_) | HIR::FuncLabel(_) => {
                                    block_end = k;
                                    break;
                                }
                                _ => {}
                            }
                        }

                        // choose insertion position: before the block's last instruction
                        // if block has no instructions (block_end == bs + 1), insert at bs + 1
                        let insert_at = if block_end > bs + 1 {
                            block_end - 1
                        } else {
                            bs + 1
                        };

                        // remove original Delete; adjust insertion index if needed
                        let delete_inst = self.hir.remove(i);
                        let adjusted_insert = if i < insert_at {
                            insert_at - 1
                        } else {
                            insert_at
                        };
                        self.hir.insert(adjusted_insert, delete_inst);

                        i = adjusted_insert + 1;
                        continue;
                    }
                }
            }

            i += 1;
        }

        // Per-block pass: move ALL Delete instructions to the end of their enclosing block,
        // inserting them immediately before the block's final instruction.
        let mut new_hir: Vec<HIR> = Vec::new();
        let mut pos = 0;
        while pos < self.hir.len() {
            if let HIR::Block(_) = &self.hir[pos] {
                // find block_end (next Block or FuncLabel) or end
                let bs = pos;
                let mut block_end = self.hir.len();
                for e in (bs + 1)..self.hir.len() {
                    match &self.hir[e] {
                        HIR::Block(_) | HIR::FuncLabel(_) => {
                            block_end = e;
                            break;
                        }
                        _ => {}
                    }
                }

                // collect header, body, deletes
                new_hir.push(self.hir[bs].clone());
                let mut body_non_delete: Vec<HIR> = Vec::new();
                let mut deletes: Vec<HIR> = Vec::new();
                for idx in (bs + 1)..block_end {
                    match &self.hir[idx] {
                        HIR::Inst(HIRInst::Delete { .. }) => deletes.push(self.hir[idx].clone()),
                        other => body_non_delete.push(other.clone()),
                    }
                }

                if body_non_delete.is_empty() {
                    // no instructions in block, just append deletes after header
                    for d in deletes {
                        new_hir.push(d);
                    }
                } else {
                    // push all but last non-delete, then deletes, then last non-delete
                    for i in 0..(body_non_delete.len() - 1) {
                        new_hir.push(body_non_delete[i].clone());
                    }
                    for d in deletes {
                        new_hir.push(d);
                    }
                    new_hir.push(body_non_delete[body_non_delete.len() - 1].clone());
                }

                pos = block_end;
                continue;
            }

            new_hir.push(self.hir[pos].clone());
            pos += 1;
        }

        // replace HIR with reassembled HIR
        self.hir = new_hir;
    }

    pub(crate) fn gen_hir(&mut self) -> Vec<HIR> {
        for i in 0..self.stmts.len() {
            self.gen_stmt_hir(self.stmts[i].clone());
        }

        // Lifetime Delete pass disabled to avoid premature frees; refcounts handled explicitly.
        self.analyze_lifetime();

        self.checker();

        self.hir.clone()
    }
}
