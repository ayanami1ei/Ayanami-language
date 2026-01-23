use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

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
        }
    }

    fn get_next_obj_id(&mut self) -> ObjId {
        let id = self.next_objid;
        self.next_objid += 1;
        id
    }

    fn find_var_id(&mut self, name: &String) -> VarId {
        match self.ast_symbol_table.find_symbol(name) {
            None => panic!(""),
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

        self.hir.push(HIR::Block(id));
        self.block_registry.insert(id, Vec::<HIRInst>::new());

        id
    }
    fn add_inst_to_block(&mut self, id: BlockId, inst: HIRInst) {
        let mut block = match self.block_registry.get(&id) {
            None => panic!("no that block"),
            Some(x) => x.clone(),
        };

        block.push(inst);
        self.block_registry.insert(id, block.to_vec());
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

    fn gen_expr_ir(&mut self, expr: &Rc<RefCell<Expr>>) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();

        match *expr.borrow() {
            Expr::Add(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                let left_objid = self.next_objid - 1;
                res.append(&mut self.gen_expr_ir(right));
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
                res.append(&mut self.gen_expr_ir(left));
                let left_objid = self.next_objid - 1;
                res.append(&mut self.gen_expr_ir(right));
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
                res.append(&mut self.gen_expr_ir(left));
                let left_objid = self.next_objid - 1;
                res.append(&mut self.gen_expr_ir(right));
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
                res.append(&mut self.gen_expr_ir(left));
                let left_objid = self.next_objid - 1;
                res.append(&mut self.gen_expr_ir(right));
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
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Equal,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Greater(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Greater,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Less(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Less,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::GreaterEqual(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::GreaterEqual,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::LessEqual(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

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
            Expr::FuncCall(ref name, ref argcs, _) => {
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
                res.append(&mut self.gen_expr_ir(expr));
                res.push(HIRInst::UnaryOp {
                    op: UnaryOperation::Not,
                    expr: self.next_objid - 1,
                    dst: self.get_next_obj_id(),
                });
            }
        };

        res
    }
    fn gen_block_ir(&mut self, block_id: BlockId, block: &Block, ret_obj:&mut Vec<ObjId>) {
        for mut i in block.body.clone() {
            match i {
                Stmt::Assign(ref left, ref right) => {
                    let mut insts = self.gen_assign_inst(left, right);
                    for inst in insts.drain(..) {
                        self.add_inst_to_block(block_id, inst);
                    }
                }
                Stmt::For(ref itor, ref start, ref end, ref step, ref inner_block) => {
                    self.gen_for_hir(itor, start, end, step, inner_block)
                }
                Stmt::While(ref cond, ref inner_block) => self.gen_while_hir(cond, inner_block),
                Stmt::If(ref cond, ref inner_block, ref mut elifs) => {
                    self.gen_if_hir(cond, inner_block, &mut elifs.clone())
                }
                Stmt::Func(ref _name, ref _argcs, ref _var_type, ref _block) => todo!(),
                Stmt::Return(ref ret_expr) => self.gen_return_hir(block_id, ret_expr, ret_obj),
                Stmt::Default => {}
            }
        }
    }

    fn gen_assign_inst(
        &mut self,
        left: &Rc<RefCell<Expr>>,
        right: &Rc<RefCell<Expr>>,
    ) -> Vec<HIRInst> {
        let mut res = self.gen_expr_ir(right);
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
        while i < block_ids.len() - 1 {
            let cond = self.gen_expr_ir(&elifs[elif_idx].0);
            let cond_id = self.next_objid - 1;

            let br_block_id = block_ids[i];
            i += 1;
            let then_block_id = block_ids[i];
            i += 1;
            let else_block_id = block_ids[i + 1];

            for i in cond {
                self.add_inst_to_block(br_block_id, i);
            }
            self.add_inst_to_block(
                br_block_id,
                HIRInst::Br {
                    cond: cond_id,
                    then_block: then_block_id,
                    else_block: else_block_id,
                },
            );

            self.gen_block_ir(then_block_id, &elifs[elif_idx].1, &mut Vec::new());
            elif_idx += 1;
            self.add_inst_to_block(then_block_id, HIRInst::Jmp { target: merge_id });
        }

        for i in block_ids {
            self.emit_block(i);
        }
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

        self.emit_block(init_block_id);
        let itor_id = self.next_objid;
        let itor_ir = self.gen_assign_hir(itor, start);
        self.hir.push(HIR::Inst(HIRInst::Jmp {
            target: cond_block_id,
        }));

        let end_id = self.next_objid;
        let end_ir = self.gen_expr_ir(end);
        let step_id = self.next_objid;
        let step_ir = self.gen_expr_ir(step);
        for i in end_ir {
            self.add_inst_to_block(cond_block_id, i);
        }
        for i in step_ir {
            self.add_inst_to_block(cond_block_id, i);
        }

        let cond_id = self.next_objid;
        let cond_ir = HIRInst::BinOp {
            left: itor_id,
            op: BinOperator::Less,
            right: end_id,
            dst: self.get_next_obj_id(),
        };
        self.add_inst_to_block(cond_block_id, cond_ir);

        self.add_inst_to_block(
            cond_block_id,
            HIRInst::Br {
                cond: cond_id,
                then_block: body_block_id,
                else_block: merge_block_id,
            },
        );
        self.emit_block(cond_block_id);

        self.gen_block_ir(body_block_id, block, &mut Vec::new());
        let temp_obj_id = self.next_objid;
        self.get_next_obj_id();
        self.add_inst_to_block(
            body_block_id,
            HIRInst::BinOp {
                left: itor_id,
                op: BinOperator::Add,
                right: step_id,
                dst: temp_obj_id,
            },
        );
        self.add_inst_to_block(
            body_block_id,
            HIRInst::Jmp {
                target: cond_block_id,
            },
        );

        self.emit_block(body_block_id);
        self.emit_block(merge_block_id);
    }
    fn gen_while_hir(&mut self, cond: &Rc<RefCell<Expr>>, block: &Block) {
        let cond = self.gen_expr_ir(cond);
        let cond_id = self.next_objid - 1;

        let br_block_id = self.new_block();
        let body_block_id = self.new_block();
        let merge_block_id = self.new_block();

        for i in cond {
            self.add_inst_to_block(br_block_id, i);
        }
        self.add_inst_to_block(
            br_block_id,
            HIRInst::Br {
                cond: cond_id,
                then_block: body_block_id,
                else_block: merge_block_id,
            },
        );
        self.emit_block(br_block_id);

        self.gen_block_ir(body_block_id, block, &mut Vec::new());
        self.emit_block(body_block_id);
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
        }

        self.add_inst_to_block(
            entry_block_id,
            HIRInst::Jmp {
                target: body_block_id,
            },
        );

        self.gen_block_ir(body_block_id, block, &mut fn_sym.ret_obj_id);
        self.emit_block(body_block_id);
    }
    fn gen_return_hir(&mut self, block_id: BlockId, ret_expr: &Rc<RefCell<Expr>>, ret_obj:&mut Vec<ObjId>) {
        let res_obj_id = self.next_objid;
        let res = self.gen_expr_ir(ret_expr);
        for i in res {
            self.add_inst_to_block(block_id, i);
        }
        self.add_inst_to_block(
            block_id,
            HIRInst::Ret {
                ret_obj: res_obj_id,
            },
        );
        ret_obj.push(res_obj_id);
        self.emit_block(block_id);
    }
    fn gen_stmt_hir(&mut self, mut stmt: Stmt) {
        match stmt {
            Stmt::Assign(ref left, ref right) => self.gen_assign_hir(left, right),
            Stmt::For(ref itor, ref start, ref end, ref step, ref block) => {
                self.gen_for_hir(itor, start, end, step, block)
            }
            Stmt::While(ref cond, ref block) => self.gen_while_hir(cond, block),
            Stmt::If(ref cond, ref block, ref elifs) => {
                self.gen_if_hir(cond, block, &mut elifs.clone())
            }
            Stmt::Func(ref name, ref argcs, ref var_type, ref block) => {
                self.gen_func_hir(name, argcs, var_type, block)
            }
            Stmt::Return(ref ret_expr) => panic!("cannot generate return stmt hir in gen_stmt_hir"),
            Stmt::Default => panic!("unkonwn stmt"),
        }
    }

    pub(crate) fn gen_hir(&mut self) -> Vec<HIR> {
        for i in 0..self.stmts.len() {
            self.gen_stmt_hir(self.stmts[i].clone());
        }

        self.hir.clone()
    }
}
