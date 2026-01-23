use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    error_type::Error,
    semantic_analysiser::SemanticAnalysiser,
    symbol_table::{Symbol, SymbolTable},
    types::{Expr, Stmt, VarType},
};

impl SemanticAnalysiser {
    pub(crate) fn new(stmts: Vec<Stmt>, symbol_table: SymbolTable) -> SemanticAnalysiser {
        SemanticAnalysiser {
            stmts,
            symbol_table: Rc::new(RefCell::new(symbol_table)),
            dummy: Rc::new(RefCell::new(Stmt::Default)),
        }
    }

    fn can_be_int(x: f64) -> bool {
        x.is_finite() && x.fract() == 0.0 && x.abs() <= (1_i64 << 53) as f64
    }

    fn infer_type(
        symbol_table: Rc<RefCell<SymbolTable>>,
        expr: Rc<RefCell<Expr>>,
    ) -> Result<HashSet<VarType>, Error> {
        let mut expr_ref = expr.borrow_mut();
        match *expr_ref {
            Expr::ConstNum(x, ref mut ty_set) => {
                #[cfg(debug_assertions)]
                {
                    println!("{}", x);
                }
                *ty_set = if Self::can_be_int(x) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Int);
                    set
                } else {
                    let mut set = HashSet::new();
                    set.insert(VarType::Float);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::ConstChar(_, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Char);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Var(ref name, ref mut ty_set) => {
                if let Some(sym) = symbol_table.borrow().find_symbol(name) {
                    if sym.its_type.is_empty() {
                        return Err(Error::new_error(format!(
                            "cannot infer the type of var {}",
                            name
                        )));
                    }
                    *ty_set = sym.its_type.clone();
                    Ok(sym.its_type.clone())
                } else {
                    Err(Error::new_error(format!(
                        "cannot infer the type of var {}",
                        name
                    )))
                }
            }
            Expr::FuncCall(ref name, ref argcs, ref mut ty_set, scope_id) => {
                // Check function symbol exists
                let fn_sym = if let Some(s) = symbol_table.borrow().find_symbol(name) {
                    s
                } else {
                    return Err(Error::new_error("undefined function".to_string()));
                };

                if !fn_sym.is_func {
                    return Err(Error::new_error(format!("{} is not a function", name)));
                }

                // Check parameter count
                if fn_sym.args.len() != argcs.len() {
                    return Err(Error::new_error(format!(
                        "argument count mismatch for function {}: expected {}, found {}",
                        name,
                        fn_sym.args.len(),
                        argcs.len()
                    )));
                }

                // Enter function scope for argument checking
                let mut st = symbol_table.borrow_mut();
                let saved = st.get_scope();
                st.set_area_ptr_by_id(scope_id);

                // Check each argument: argument must be an existing variable and its type must be compatible
                for i in 0..argcs.len() {
                    let call_arg = &argcs[i];
                    let param_sym = &fn_sym.args[i];

                    // find the passed variable
                    if let Some(arg_sym) = st.find_symbol(&call_arg.var_name) {
                        // ref-ness should match between declaration and call-site
                        if param_sym.is_ref != call_arg.is_ref {
                            return Err(Error::new_error(format!(
                                "ref-ness mismatch for parameter {} of function {}",
                                param_sym.name, name
                            )));
                        }

                        // if parameter type is known, require intersection
                        if !param_sym.its_type.is_empty() && !arg_sym.its_type.is_empty() {
                            let inter = param_sym
                                .its_type
                                .intersection(&arg_sym.its_type)
                                .cloned()
                                .collect::<HashSet<VarType>>();
                            if inter.is_empty() {
                                return Err(Error::new_error(format!(
                                    "type mismatch for parameter {} of function {}",
                                    param_sym.name, name
                                )));
                            }
                        }
                    } else {
                        return Err(Error::new_error(format!(
                            "undefined variable {} in call to {}",
                            call_arg.var_name, name
                        )));
                    }
                }

                st.area_ptr = Rc::downgrade(&saved);

                // Use function symbol's return-type set if available
                if fn_sym.its_type.is_empty() {
                    *ty_set = {
                        let mut set = HashSet::new();
                        set.insert(VarType::Unknown);
                        set
                    };
                } else {
                    *ty_set = fn_sym.its_type.clone();
                }

                Ok(ty_set.clone())
            }
            Expr::Add(ref a, ref b, ref mut ty_set) => {
                let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
                let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;
                *ty_set = if type_a.contains(&VarType::Float) || type_b.contains(&VarType::Float) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Float);
                    set
                } else if type_a.contains(&VarType::Char) || type_b.contains(&VarType::Char) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Char);
                    set
                } else {
                    let mut set = HashSet::new();
                    set.insert(VarType::Int);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Sub(ref a, ref b, ref mut ty_set) => {
                let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
                let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;
                *ty_set = if type_a.contains(&VarType::Float) || type_b.contains(&VarType::Float) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Float);
                    set
                } else if type_a.contains(&VarType::Char) || type_b.contains(&VarType::Char) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Char);
                    set
                } else {
                    let mut set = HashSet::new();
                    set.insert(VarType::Int);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Mul(ref a, ref b, ref mut ty_set) => {
                let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
                let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;
                *ty_set = if type_a.contains(&VarType::Float) || type_b.contains(&VarType::Float) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Float);
                    set
                } else if type_a.contains(&VarType::Char) || type_b.contains(&VarType::Char) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Char);
                    set
                } else {
                    let mut set = HashSet::new();
                    set.insert(VarType::Int);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Div(ref a, ref b, ref mut ty_set) => {
                let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
                let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;
                *ty_set = if type_a.contains(&VarType::Float) || type_b.contains(&VarType::Float) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Float);
                    set
                } else if type_a.contains(&VarType::Char) || type_b.contains(&VarType::Char) {
                    let mut set = HashSet::new();
                    set.insert(VarType::Char);
                    set
                } else {
                    let mut set = HashSet::new();
                    set.insert(VarType::Int);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Equal(_, _, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Greater(_, _, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Less(_, _, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::GreaterEqual(_, _, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::LessEqual(_, _, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
            Expr::Not(_, ref mut ty_set) => {
                *ty_set = {
                    let mut set = HashSet::new();
                    set.insert(VarType::Bool);
                    set
                };
                Ok(ty_set.clone())
            }
        }
    }

    fn semantic_analysise_assign(&mut self) -> Result<(), Error> {
        let (a, b) = if let Stmt::Assign(ref _a, ref _b) = *self.dummy.borrow() {
            (_a.clone(), _b.clone())
        } else {
            return Err(Error::new_error("".to_string()));
        };

        let b_type = Self::infer_type(self.symbol_table.clone(), b.clone())?;

        if let Expr::Var(ref name, _) = *a.borrow() {
            if let Some(mut a_sym) = (*self.symbol_table).borrow().find_symbol(name) {
                if a_sym
                    .its_type
                    .intersection(&b_type)
                    .collect::<Vec<&VarType>>()
                    .len()
                    == 0
                {
                    a_sym.its_type = a_sym.its_type.intersection(&b_type).cloned().collect();
                }
            } else {
                let new_a = Rc::new(RefCell::new(Expr::Var(name.clone(), b_type.clone())));
                let new_b = b.clone();
                *std::cell::RefCell::borrow_mut(&self.dummy) = Stmt::Assign(new_a, new_b);
                let mut a_sym =
                    Symbol::new_var(name.clone(), (*self.symbol_table).borrow().get_scope());
                a_sym.its_type = b_type.clone();
                (*self.symbol_table).borrow_mut().add_symbol(a_sym);
            }
            Ok(())
        } else {
            Err(Error::new_error(
                "illegal type of assign, must be var".to_string(),
            ))
        }
    }
    fn semantic_analysise_for(&mut self) -> Result<(), Error> {
        let (itor, start, end, step, block, scope_id) =
            if let Stmt::For(ref _itor, ref _start, ref _end, ref _step, ref block, scope_id) =
                *self.dummy.borrow()
            {
                (
                    _itor.clone(),
                    _start.clone(),
                    _end.clone(),
                    _step.clone(),
                    block.clone(),
                    scope_id,
                )
            } else {
                return Err(Error::new_error("".to_string()));
            };

        (*self.symbol_table)
            .borrow_mut()
            .set_area_ptr_by_id(scope_id);

        if let Expr::Var(ref name, _) = *itor.borrow() {
            (*self.symbol_table).borrow_mut().into_new_scope();
            if let Some(itor_sym) = (*self.symbol_table).borrow().find_symbol(name) {
                if !itor_sym.its_type.contains(&VarType::Int) {
                    return Err(Error::new_error(format!(
                        "the itor must be type int, but find {:?}",
                        itor_sym.its_type
                    )));
                }
            } else {
                let mut itor_sym =
                    Symbol::new_var(name.clone(), (*self.symbol_table).borrow().get_scope());
                itor_sym.its_type.insert(VarType::Int);
                (*self.symbol_table).borrow_mut().add_symbol(itor_sym);
            }

            if !Self::infer_type(self.symbol_table.clone(), start.clone())?.contains(&VarType::Int)
                || !Self::infer_type(self.symbol_table.clone(), end.clone())?
                    .contains(&VarType::Int)
                || !Self::infer_type(self.symbol_table.clone(), step.clone())?
                    .contains(&VarType::Int)
            {
                return Err(Error::new_error(format!(
                    "the elements of for must be type int",
                )));
            }

            for stmt in block.body.clone() {
                self.semantic_analysise_stmt(stmt)?;
            }

            (*self.symbol_table).borrow_mut().ret_to_parent_scope();
        }

        Ok(())
    }
    fn semantic_analysise_while(&mut self) -> Result<(), Error> {
        let (cond, block, scope_id) =
            if let Stmt::While(ref _cond, ref block, scope_id) = *self.dummy.borrow() {
                (_cond.clone(), block.clone(), scope_id)
            } else {
                return Err(Error::new_error("".to_string()));
            };

        (*self.symbol_table)
            .borrow_mut()
            .set_area_ptr_by_id(scope_id);

        if !(Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Bool)
            || Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Int)
            || Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Float))
        {
            return Err(Error::new_error("condition must be type bool".to_string()));
        }

        (*self.symbol_table).borrow_mut().into_new_scope();

        for stmt in block.body.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }

        (*self.symbol_table).borrow_mut().ret_to_parent_scope();

        (*self.symbol_table).borrow_mut().reset();

        Ok(())
    }
    fn semantic_analysise_if(&mut self) -> Result<(), Error> {
        let (cond, block, elifs, scope_id) =
            if let Stmt::If(ref _cond, ref block, ref elifs, scope_id) = *self.dummy.borrow() {
                (_cond.clone(), block.clone(), elifs.clone(), scope_id)
            } else {
                return Err(Error::new_error("".to_string()));
            };

        (*self.symbol_table)
            .borrow_mut()
            .set_area_ptr_by_id(scope_id);

        if !(Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Bool)
            || Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Int)
            || Self::infer_type(self.symbol_table.clone(), cond.clone())?.contains(&VarType::Float))
        {
            return Err(Error::new_error("condition must be type bool".to_string()));
        }

        (*self.symbol_table).borrow_mut().into_new_scope();

        for stmt in block.body.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }

        (*self.symbol_table).borrow_mut().ret_to_parent_scope();

        for i in 0..elifs.len() {
            let (_cond, block) = &elifs[i];
            let cond = _cond.clone();
            if !(Self::infer_type(self.symbol_table.clone(), cond.clone())?
                .contains(&VarType::Bool)
                || Self::infer_type(self.symbol_table.clone(), cond.clone())?
                    .contains(&VarType::Int)
                || Self::infer_type(self.symbol_table.clone(), cond.clone())?
                    .contains(&VarType::Float))
            {
                return Err(Error::new_error("condition must be type bool".to_string()));
            }

            (*self.symbol_table).borrow_mut().into_new_scope();

            for stmt in block.body.clone() {
                self.semantic_analysise_stmt(stmt)?;
            }

            (*self.symbol_table).borrow_mut().ret_to_parent_scope();
        }

        (*self.symbol_table).borrow_mut().reset();

        Ok(())
    }
    fn semantic_analysise_func(&mut self) -> Result<(), Error> {
        let (name, _args, block, scope_id) =
            if let Stmt::Func(ref name, ref _args, _, ref block, scope_id) = *self.dummy.borrow() {
                (name.clone(), _args.clone(), block.clone(), scope_id)
            } else {
                return Err(Error::new_error("".to_string()));
            };

        {
            // 进入函数体作用域（优先使用函数符号中记录的 body_scope_id）
            let mut symtab_binding = self.symbol_table.borrow_mut();
            if let Some(func_sym) = symtab_binding.find_symbol(&name) {
                if let Some(body_id) = func_sym.body_scope_id {
                    symtab_binding.set_area_ptr_by_id(body_id);
                } else {
                    symtab_binding.set_area_ptr_by_id(scope_id);
                }
            } else {
                symtab_binding.set_area_ptr_by_id(scope_id);
            }
        }

        for stmt in block.body.clone() {
            if let Stmt::Return(ref _ret) = stmt {
                self.semantic_analysise_ret(&name, _ret)?;
            } else {
                self.semantic_analysise_stmt(stmt)?;
            }
        }

        (*self.symbol_table).borrow_mut().ret_to_parent_scope();

        (*self.symbol_table).borrow_mut().reset();

        Ok(())
    }
    fn semantic_analysise_call(&mut self) -> Result<(), Error> {
        if let Stmt::Call(ref name, ref argcs, scope_id) = *self.dummy.borrow() {
            // 首先在当前（调用者）作用域推断每个实参的类型
            let mut arg_types: Vec<HashSet<VarType>> = Vec::new();
            for arg in argcs {
                let arg_expr = Expr::Var(arg.var_name.clone(), HashSet::new());
                let ty =
                    Self::infer_type(self.symbol_table.clone(), Rc::new(RefCell::new(arg_expr)))?;
                arg_types.push(ty);
            }

            // 记录被调用函数的参数名（从函数符号中读取）
            let func_sym_opt = self.symbol_table.borrow().find_symbol(name);
            if func_sym_opt.is_none() {
                return Err(Error::new_error(format!("undefined function {}", name)));
            }
            let func_sym = func_sym_opt.unwrap();

            // 进入被调用函数的作用域，合并实参类型到被调用函数的参数符号上
            (*self.symbol_table)
                .borrow_mut()
                .set_area_ptr_by_id(scope_id);
            for i in 0..arg_types.len() {
                if i >= func_sym.args.len() {
                    break;
                }
                let param_name = func_sym.args[i].name.clone();
                (*self.symbol_table)
                    .borrow_mut()
                    .add_symbol_type(&param_name, arg_types[i].clone());
            }
            (*self.symbol_table).borrow_mut().ret_to_parent_scope();

            Ok(())
        } else {
            Err(Error::new_error("".to_string()))
        }
    }
    fn semantic_analysise_ret(
        &mut self,
        name: &String,
        _ret: &Rc<RefCell<Expr>>,
    ) -> Result<(), Error> {
        let ret = _ret.clone();
        let ret_type = Self::infer_type(self.symbol_table.clone(), ret)?;
        self.symbol_table
            .borrow_mut()
            .add_symbol_type(&name, ret_type.clone());

        #[cfg(debug_assertions)]
        {
            println!("{} ret type {:?}", name.clone(), ret_type);
        }

        Ok(())
    }
    fn semantic_analysise_stmt(&mut self, stmt: Stmt) -> Result<(), Error> {
        self.dummy = Rc::new(RefCell::new(stmt));

        let stmt_type = {
            let borrowed = self.dummy.borrow();
            match *borrowed {
                Stmt::Assign(..) => 0,
                Stmt::Call(..) => 6,
                Stmt::For(..) => 1,
                Stmt::While(..) => 2,
                Stmt::If(..) => 3,
                Stmt::Func(..) => 4,
                _ => 5,
            }
        };

        match stmt_type {
            0 => self.semantic_analysise_assign(),
            1 => self.semantic_analysise_for(),
            2 => self.semantic_analysise_while(),
            3 => self.semantic_analysise_if(),
            4 => self.semantic_analysise_func(),
            6 => self.semantic_analysise_call(),
            _ => Ok(()),
        }
    }

    pub(crate) fn semantic_analysise(&mut self) -> Result<Vec<Stmt>, Error> {
        (*self.symbol_table).borrow_mut().reset();
        for stmt in self.stmts.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }
        Ok(self.stmts.clone())
    }
}
