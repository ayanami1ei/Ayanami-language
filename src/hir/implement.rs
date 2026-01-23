use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use llvm_sys::lto::thinlto_codegen_add_cross_referenced_symbol;

use crate::{
    hir::{
        BinOperator, BlockId, FuncId, HIR, HIRInst, HirFuncSymbol, HirGenerator, HirVarSymbol,
        ObjId, StorageClass, UnaryOperation, VarId,
    },
    symbol_table::{Symbol, SymbolTable},
    types::{Argc, Block, Expr, Stmt, VarType},
};

impl HirVarSymbol {
    pub(super) fn new(ast_sym: Symbol, obj_id: ObjId) -> HirVarSymbol {
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

    pub(super) fn new_temp_var(ty_set: HashSet<VarType>, obj_id: ObjId) -> HirVarSymbol {
        HirVarSymbol {
            ty_set,
            mutability: true,
            storage: StorageClass::Temp,
            obj_id,
        }
    }
}

impl HirGenerator {
    pub(crate) fn new(stmts: Vec<Stmt>, ast_symbol_table: SymbolTable) -> HirGenerator {
        HirGenerator {
            stmts,

            next_objid: ObjId(0),
            next_tempvar_id: ObjId(0),
            next_blockid: BlockId(0),

            var_registry: HashMap::<VarId, HirVarSymbol>::new(),
            func_registry: HashMap::<FuncId, HirFuncSymbol>::new(),
            block_registry: HashMap::<BlockId, Vec<HIRInst>>::new(),

            ast_symbol_table,

            hir: Vec::<HIR>::new(),
            pending_block_emits: Vec::<BlockId>::new(),
        }
    }

    fn get_next_obj_id(&mut self) -> ObjId {
        let id = self.next_objid;
        self.next_objid += 1;
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

    fn emit_pending_blocks(&mut self) {
        if self.pending_block_emits.len() == 0 {
            return;
        }

        // Drain in FIFO order
        let pending: Vec<BlockId> = self.pending_block_emits.drain(..).collect();
        for id in pending {
            self.emit_block(id);
        }
    }

    fn gen_expr_ir(&mut self, expr: &Rc<RefCell<Expr>>) {
        let mut res = Vec::<HIRInst>::new();

        match *expr.borrow() {
            Expr::Add(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                let left_objid = self.next_objid - 1;
                self.gen_expr_ir(right);
                let right_objid = self.next_objid - 1;

                let new_obj_id = self.next_objid;
                res.push(HIRInst::New {
                    obj_type: left
                        .borrow()
                        .get_type_set()
                        .union(right.borrow().get_type_set())
                        .cloned()
                        .collect(),
                    dst: self.get_next_obj_id(),
                });

                res.push(HIRInst::BinOp {
                    left: left_objid,
                    op: super::BinOperator::Add,
                    right: right_objid,
                    dst: new_obj_id,
                });
            }
            Expr::Sub(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                let left_objid = self.next_objid - 1;
                self.gen_expr_ir(right);
                let right_objid = self.next_objid - 1;

                let new_obj_id = self.next_objid;
                res.push(HIRInst::New {
                    obj_type: left
                        .borrow()
                        .get_type_set()
                        .union(right.borrow().get_type_set())
                        .cloned()
                        .collect(),
                    dst: self.get_next_obj_id(),
                });

                res.push(HIRInst::BinOp {
                    left: left_objid,
                    op: super::BinOperator::Sub,
                    right: right_objid,
                    dst: new_obj_id,
                });
            }
            Expr::Mul(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                let left_objid = self.next_objid - 1;
                self.gen_expr_ir(right);
                let right_objid = self.next_objid - 1;

                let new_obj_id = self.next_objid;
                res.push(HIRInst::New {
                    obj_type: left
                        .borrow()
                        .get_type_set()
                        .union(right.borrow().get_type_set())
                        .cloned()
                        .collect(),
                    dst: self.get_next_obj_id(),
                });

                res.push(HIRInst::BinOp {
                    left: left_objid,
                    op: super::BinOperator::Mul,
                    right: right_objid,
                    dst: new_obj_id,
                });
            }
            Expr::Div(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                let left_objid = self.next_objid - 1;
                self.gen_expr_ir(right);
                let right_objid = self.next_objid - 1;

                let new_obj_id = self.next_objid;
                res.push(HIRInst::New {
                    obj_type: left
                        .borrow()
                        .get_type_set()
                        .union(right.borrow().get_type_set())
                        .cloned()
                        .collect(),
                    dst: self.get_next_obj_id(),
                });

                res.push(HIRInst::BinOp {
                    left: left_objid,
                    op: super::BinOperator::Div,
                    right: right_objid,
                    dst: new_obj_id,
                });
            }
            Expr::Equal(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                self.gen_expr_ir(right);

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Equal,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Greater(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                self.gen_expr_ir(right);

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Greater,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Less(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                self.gen_expr_ir(right);

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Less,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::GreaterEqual(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                self.gen_expr_ir(right);

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::GreaterEqual,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::LessEqual(ref left, ref right, _) => {
                self.gen_expr_ir(left);
                self.gen_expr_ir(right);

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::LessEqual,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::ConstChar(c, _) => {
                let mut set = HashSet::<VarType>::new();
                set.insert(VarType::Char);
                res.push(HIRInst::New {
                    obj_type: set,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::ConstNum(x, _) => {
                let mut set = HashSet::<VarType>::new();
                set.insert(VarType::Int);
                res.push(HIRInst::New {
                    obj_type: set,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Var(ref name, ref var_type) => {
                let temp_obj_id = self.next_objid;
                res.push(HIRInst::New {
                    obj_type: var_type.clone(),
                    dst: self.get_next_obj_id(),
                });

                res.push(HIRInst::Load {
                    var: self.find_var_id(name),
                    obj: temp_obj_id,
                });
            }
            Expr::FuncCall(ref name, ref argcs, _, _) => {
                let func_sym = self.find_func_sym(name);

                // For each argument (call-site) produce a temporary object and load the variable into it.

                for a in argcs {
                    let temp_obj = self.next_objid;
                    // create a placeholder object for the argument
                    res.push(HIRInst::New {
                        obj_type: HashSet::new(),
                        dst: self.get_next_obj_id(),
                    });

                    // load the argument variable into the temp object
                    let var_id = self.find_var_id(&a.var_name);
                    res.push(HIRInst::Load {
                        var: var_id,
                        obj: temp_obj,
                    });
                }

                res.push(HIRInst::Call {
                    id: func_sym.id,
                    ret: func_sym.ret_obj_id,
                });
            }
            Expr::Not(ref expr, _) => {
                self.gen_expr_ir(expr);
                res.push(HIRInst::UnaryOp {
                    op: UnaryOperation::Not,
                    expr: self.next_objid - 1,
                    dst: self.get_next_obj_id(),
                });
            }
        };

        for i in res {
            self.hir.push(HIR::Inst(i));
        }
    }
    fn gen_block_ir(&mut self, block_id: BlockId, block: &Block, ret_obj: &mut Vec<ObjId>) {
        for mut i in block.body.clone() {
            match i {
                Stmt::Assign(ref left, ref right) => {
                    let mut insts = self.gen_assign_inst(left, right);
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
                    println!("while");
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_while_hir(cond, inner_block);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::If(ref cond, ref inner_block, ref mut elifs, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_if_hir(cond, inner_block, &mut elifs.clone());
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Func(ref name, ref argcs, ref var_type, ref _block, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    self.gen_func_hir(name, argcs, var_type, block);
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Call(ref name, ref argcs, scope_id) => {
                    self.ast_symbol_table.set_area_ptr_by_id(scope_id);
                    let insts = self.gen_call_inst(name, argcs);
                    for inst in insts.into_iter() {
                        self.hir.push(HIR::Inst(inst));
                    }
                    self.ast_symbol_table.ret_to_parent_scope();
                }
                Stmt::Return(ref ret_expr) => self.gen_return_hir(block_id, ret_expr, ret_obj),
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
    ) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();
        self.gen_expr_ir(right);
        let obj_id = self.next_objid - 1;

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
                new_sym = HirVarSymbol::new(sym, obj_id);
            }
        }

        let old_obj_id = self.var_registry.get(&id).map(|s| s.obj_id);

        self.var_registry.insert(id, new_sym);
        res.push(HIRInst::Bind {
            var: id,
            obj: obj_id,
        });
        res.push(HIRInst::IncRef { obj: obj_id });
        if let Some(old) = old_obj_id {
            res.push(HIRInst::DecRef { obj: old });
        }

        res
    }

    fn gen_assign_hir(&mut self, left: &Rc<RefCell<Expr>>, right: &Rc<RefCell<Expr>>) {
        let res = self.gen_assign_inst(left, right);

        for i in &res {
            self.hir.push(HIR::Inst(i.clone()))
        }
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
            block_ids.push(self.new_block());
        }
        block_ids.push(self.new_block()); //merge
        let merge_id = block_ids[block_ids.len() - 1];

        let mut i = 0;
        let mut elif_idx = 0;
        while elifs.len() != 0 && i < block_ids.len() - 1 && elif_idx < elifs.len() {
            let br_block_id = block_ids[i];
            self.emit_block(br_block_id);
            i += 1;

            self.gen_expr_ir(&elifs[elif_idx].0);
            let cond_id = self.next_objid - 1;

            let then_block_id = block_ids[i];
            i += 1;
            let else_block_id = block_ids[i + 1];

            self.hir.push(HIR::Inst(HIRInst::Br {
                cond: cond_id,
                then_block: then_block_id,
                else_block: else_block_id,
            }));

            self.emit_block(then_block_id);
            self.gen_block_ir(then_block_id, &elifs[elif_idx].1, &mut Vec::new());
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
        let merge_block_id = self.new_block();
        // Defer emitting blocks until the parent block is emitted to preserve source order.
        self.emit_block(init_block_id);
        let itor_id = self.next_objid;
        self.gen_assign_hir(itor, start);
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        self.emit_block(cond_block_id);
        let end_id = self.next_objid;
        self.gen_expr_ir(end);
        let step_id = self.next_objid;
        self.gen_expr_ir(step);

        let cond_id = self.next_objid;
        let cond_ir = HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Less,
            right: end_id,
            dst: self.get_next_obj_id(),
        };
        self.hir.push(HIR::Inst(cond_ir));

        self.hir.push(HIR::Inst(HIRInst::Br {
            cond: cond_id,
            then_block: body_block_id,
            else_block: merge_block_id,
        }));
        // cond_block_id will be emitted later

        self.emit_block(body_block_id);
        self.gen_block_ir(body_block_id, block, &mut Vec::new());
        let temp_obj_id = self.next_objid;
        self.get_next_obj_id();
        self.hir.push(HIR::Inst(HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Add,
            right: step_id,
            dst: temp_obj_id,
        }));
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        // Defer emitting these blocks until the parent block is emitted
        self.emit_block(merge_block_id);
    }
    fn gen_while_hir(&mut self, cond: &Rc<RefCell<Expr>>, block: &Block) {
        let br_block_id = self.new_block();
        let cond_id = self.next_objid;
        self.emit_block(br_block_id);
        self.gen_expr_ir(cond);

        let body_block_id = self.new_block();
        let merge_block_id = self.new_block();

        self.hir.push(HIR::Inst(HIRInst::Br {
            cond: cond_id,
            then_block: body_block_id,
            else_block: merge_block_id,
        }));
        // Defer emitting blocks until the parent block is emitted to preserve source order.
        self.emit_block(body_block_id);
        self.gen_block_ir(body_block_id, block, &mut Vec::new());
        self.hir.push(HIR::Inst(HIRInst::Jmp { target: br_block_id }));
        self.emit_block(merge_block_id);
    }
    fn gen_func_hir(
        &mut self,
        name: &String,
        argcs: &Vec<Argc>,
        var_type: &HashSet<VarType>,
        block: &Block,
    ) {
        let ast_fn_sym = match self.ast_symbol_table.find_symbol(name) {
            None => panic!(""),
            Some(sym) => sym,
        };
        let mut fn_sym = HirFuncSymbol {
            ty_set: ast_fn_sym.its_type,
            ret_obj_id: Vec::<ObjId>::new(),
            id: FuncId(ast_fn_sym.id),
        };

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
            let hir_sym = HirVarSymbol {
                ty_set,
                mutability: true,
                storage: StorageClass::Param,
                obj_id: ObjId(i as i32),
            };

            self.var_registry.insert(param_id, hir_sym);
            self.hir.push(HIR::Inst(HIRInst::Bind {
                var: param_id,
                obj: ObjId(0),
            }));
        }

        self.func_registry.insert(fn_sym.id, fn_sym.clone());

        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: body_block_id,
        }));
        self.emit_block(body_block_id);

        self.gen_block_ir(body_block_id, block, &mut fn_sym.ret_obj_id);
        // gen_block_ir will emit the body block and any nested blocks in correct order
    }
    fn gen_call_hir(&mut self, name: &String, argcs: &Vec<Argc>) {
        let func_sym = self.find_func_sym(name);

        for a in argcs {
            let dst = self.get_next_obj_id();
            self.hir.push(HIR::Inst(HIRInst::New {
                obj_type: HashSet::new(),
                dst,
            }));

            let var_id = self.find_var_id(&a.var_name);
            self.hir.push(HIR::Inst(HIRInst::Load {
                var: var_id,
                obj: dst,
            }));
        }

        self.hir.push(HIR::Inst(HIRInst::Call {
            id: func_sym.id,
            ret: func_sym.ret_obj_id,
        }));
    }

    fn gen_call_inst(&mut self, name: &String, argcs: &Vec<Argc>) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();
        let func_sym = self.find_func_sym(name);

        for a in argcs {
            let dst = self.get_next_obj_id();
            res.push(HIRInst::New {
                obj_type: HashSet::new(),
                dst,
            });

            let var_id = self.find_var_id(&a.var_name);
            res.push(HIRInst::Load {
                var: var_id,
                obj: dst,
            });
        }

        res.push(HIRInst::Call {
            id: func_sym.id,
            ret: func_sym.ret_obj_id,
        });

        res
    }
    fn gen_return_hir(
        &mut self,
        block_id: BlockId,
        ret_expr: &Rc<RefCell<Expr>>,
        ret_obj: &mut Vec<ObjId>,
    ) {
        let res_obj_id = self.next_objid;
        self.gen_expr_ir(ret_expr);

        self.hir.push(HIR::Inst(HIRInst::Ret {
            ret_obj: res_obj_id,
        }));
        ret_obj.push(res_obj_id);
        //self.emit_block(block_id);
    }
    fn gen_stmt_hir(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign(ref left, ref right) => self.gen_assign_hir(left, right),
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
            Stmt::Call(ref name, ref argcs, scope_id) => {
                self.gen_call_hir(name, argcs);
            }
            Stmt::Return(ref ret_expr) => panic!("cannot generate return stmt hir in gen_stmt_hir"),
            Stmt::Default => panic!("unkonwn stmt"),
        }
    }

    pub(crate) fn gen_hir(&mut self) -> Vec<HIR> {
        for i in 0..self.stmts.len() {
            // Flush any blocks produced by previous statements (e.g. loops/ifs)
            self.emit_pending_blocks();
            self.gen_stmt_hir(self.stmts[i].clone());
        }

        // Flush any remaining pending blocks
        self.emit_pending_blocks();

        self.hir.clone()
    }
}
