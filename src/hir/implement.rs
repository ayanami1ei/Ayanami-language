use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    hir::{BlockId, HIRInst, HirGenerator, HirSymbol, ObjId, StorageClass, UnaryOperation, VarId},
    symbol_table::{Symbol, SymbolTable},
    types::{Expr, Stmt, VarType},
};

impl HirSymbol {
    pub(super) fn new(ast_sym: Symbol) -> HirSymbol {
        let mutability = true;
        let mut storage = StorageClass::Local;

        if ast_sym.is_argc {
            storage = StorageClass::Param;
        }

        HirSymbol {
            ty_set: ast_sym.its_type,
            mutability,
            storage,
        }
    }

    pub(super) fn new_temp_var(ty_set: HashSet<VarType>) -> HirSymbol {
        HirSymbol {
            ty_set,
            mutability: true,
            storage: StorageClass::Temp,
        }
    }
}

impl HirGenerator {
    pub(crate) fn new(stmts: Vec<Stmt>, ast_symbol_table: SymbolTable) -> HirGenerator {
        HirGenerator {
            stmts,

            next_objid: ObjId(0),
            next_tempvar_id: ObjId(0),

            var_registry: HashMap::<VarId, HirSymbol>::new(),
            obj_registry: HashMap::<ObjId, HirSymbol>::new(),
            block_registry: HashMap::<BlockId, HirSymbol>::new(),

            ast_symbol_table,
        }
    }

    fn get_next_obj_id(&mut self) -> ObjId {
        let id = self.next_objid;
        self.next_objid += 1;
        id
    }
    fn new_temp_var(&mut self) -> ObjId {
        let id = self.next_tempvar_id;
        self.next_tempvar_id += 1;
        id
    }

    fn gen_expr_ir(&mut self, expr: &Rc<RefCell<Expr>>) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();

        match *expr.borrow() {
            Expr::Add(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Add,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Sub(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Sub,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Mul(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Mul,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Div(ref left, ref right, _) => {
                res.append(&mut self.gen_expr_ir(left));
                res.append(&mut self.gen_expr_ir(right));

                res.push(HIRInst::BinOp {
                    left: self.next_objid - 2,
                    op: super::BinOperator::Div,
                    right: self.next_objid - 2,
                    dst: self.get_next_obj_id(),
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
                res.push(HIRInst::New {
                    obj_type: VarType::Char,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::ConstNum(x, _) => {
                res.push(HIRInst::New {
                    obj_type: VarType::Int,
                    dst: self.get_next_obj_id(),
                });
            }
            Expr::Var(ref name, _) => todo!(),
            Expr::FuncCall(ref name, ref argcs, _) => todo!(),
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

    fn gen_assign_hir(
        &mut self,
        left: &Rc<RefCell<Expr>>,
        right: &Rc<RefCell<Expr>>,
    ) -> Vec<HIRInst> {
        let mut res = self.gen_expr_ir(right);

        let new_sym: HirSymbol;
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
                new_sym = HirSymbol::new(sym);
            }
        }

        self.var_registry.insert(id, new_sym);
        res.push(HIRInst::Bind {
            var: id,
            obj: self.next_objid - 1,
        });
        res
    }
    fn gen_if_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        vec![]
    }
    fn gen_for_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        vec![]
    }
    fn gen_while_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        vec![]
    }
    fn gen_func_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        vec![]
    }
    fn gen_return_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        vec![]
    }
    fn gen_stmt_hir(&mut self, stmt: Stmt) -> Vec<HIRInst> {
        if let Stmt::Assign(ref left, ref right) = stmt {
            return self.gen_assign_hir(left, right);
        }

        Vec::new()
    }

    pub(crate) fn gen_hir(&mut self) -> Vec<HIRInst> {
        let mut res = Vec::<HIRInst>::new();
        for i in 0..self.stmts.len() {
            res.append(&mut self.gen_stmt_hir(self.stmts[i].clone()));
        }

        res
    }
}
