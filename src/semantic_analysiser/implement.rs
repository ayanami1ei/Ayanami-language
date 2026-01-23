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
            symbol_table,
            dummy: Rc::new(RefCell::new(Stmt::Default)),
        }
    }

    fn can_be_int(x: f64) -> bool {
        x.is_finite() && x.fract() == 0.0 && x.abs() <= (1_i64 << 53) as f64
    }

    fn infer_type(
        symbol_table: SymbolTable,
        expr: &Rc<RefCell<Expr>>,
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
                if let Some(sym) = symbol_table.find_symbol(name) {
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
            Expr::FuncCall(ref name, ref argcs, ref mut ty_set) => {
                // Check function symbol exists
                let fn_sym = if let Some(s) = symbol_table.find_symbol(name) {
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

                // Check each argument: argument must be an existing variable and its type must be compatible
                for i in 0..argcs.len() {
                    let call_arg = &argcs[i];
                    let param_sym = &fn_sym.args[i];

                    // find the passed variable
                    if let Some(arg_sym) = symbol_table.find_symbol(&call_arg.var_name) {
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
                let type_a = Self::infer_type(symbol_table.clone(), a)?;
                let type_b = Self::infer_type(symbol_table.clone(), b)?;
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
                let type_a = Self::infer_type(symbol_table.clone(), a)?;
                let type_b = Self::infer_type(symbol_table.clone(), b)?;
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
                let type_a = Self::infer_type(symbol_table.clone(), a)?;
                let type_b = Self::infer_type(symbol_table.clone(), b)?;
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
                let type_a = Self::infer_type(symbol_table.clone(), a)?;
                let type_b = Self::infer_type(symbol_table.clone(), b)?;
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

        let b_type = Self::infer_type(self.symbol_table.clone(), &b)?;

        if let Expr::Var(ref name, _) = *a.borrow() {
            if let Some(mut a_sym) = self.symbol_table.find_symbol(name) {
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
                *self.dummy.borrow_mut() = Stmt::Assign(new_a, new_b);
                let mut a_sym = Symbol::new_var(name.clone(), self.symbol_table.get_scope());
                a_sym.its_type = b_type.clone();
                self.symbol_table.add_symbol(a_sym);
            }
            Ok(())
        } else {
            Err(Error::new_error(
                "illegal type of assign, must be var".to_string(),
            ))
        }
    }
    fn semantic_analysise_for(&mut self) -> Result<(), Error> {
        let (itor, start, end, step, block) =
            if let Stmt::For(ref _itor, ref _start, ref _end, ref _step, ref block) =
                *self.dummy.borrow()
            {
                (
                    _itor.clone(),
                    _start.clone(),
                    _end.clone(),
                    _step.clone(),
                    block.clone(),
                )
            } else {
                return Err(Error::new_error("".to_string()));
            };

        if let Expr::Var(ref name, _) = *itor.borrow() {
            self.symbol_table.into_new_scope();
            if let Some(itor_sym) = self.symbol_table.find_symbol(name) {
                if !itor_sym.its_type.contains(&VarType::Int) {
                    return Err(Error::new_error(format!(
                        "the itor must be type int, but find {:?}",
                        itor_sym.its_type
                    )));
                }
            } else {
                let mut itor_sym = Symbol::new_var(name.clone(), self.symbol_table.get_scope());
                itor_sym.its_type.insert(VarType::Int);
                self.symbol_table.add_symbol(itor_sym);
            }

            if !Self::infer_type(self.symbol_table.clone(), &start)?.contains(&VarType::Int)
                || !Self::infer_type(self.symbol_table.clone(), &end)?.contains(&VarType::Int)
                || !Self::infer_type(self.symbol_table.clone(), &step)?.contains(&VarType::Int)
            {
                return Err(Error::new_error(format!(
                    "the elements of for must be type int",
                )));
            }

            for stmt in block.body.clone() {
                self.semantic_analysise_stmt(stmt)?;
            }

            self.symbol_table.ret_to_parent_scope();
        }

        Ok(())
    }
    fn semantic_analysise_while(&mut self) -> Result<(), Error> {
        let (cond, block) = if let Stmt::While(ref _cond, ref block) = *self.dummy.borrow() {
            (_cond.clone(), block.clone())
        } else {
            return Err(Error::new_error("".to_string()));
        };

        if !(Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Bool)
            || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Int)
            || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Float))
        {
            return Err(Error::new_error("condition must be type bool".to_string()));
        }

        self.symbol_table.into_new_scope();

        for stmt in block.body.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }

        self.symbol_table.ret_to_parent_scope();

        Ok(())
    }
    fn semantic_analysise_if(&mut self) -> Result<(), Error> {
        let (cond, block, elifs) =
            if let Stmt::If(ref _cond, ref block, ref elifs) = *self.dummy.borrow() {
                (_cond.clone(), block.clone(), elifs.clone())
            } else {
                return Err(Error::new_error("".to_string()));
            };

        if !(Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Bool)
            || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Int)
            || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Float))
        {
            return Err(Error::new_error("condition must be type bool".to_string()));
        }

        self.symbol_table.into_new_scope();

        for stmt in block.body.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }

        self.symbol_table.ret_to_parent_scope();

        for i in 0..elifs.len() {
            let (_cond, block) = &elifs[i];
            let cond = _cond.clone();
            if !(Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Bool)
                || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Int)
                || Self::infer_type(self.symbol_table.clone(), &cond)?.contains(&VarType::Float))
            {
                return Err(Error::new_error("condition must be type bool".to_string()));
            }

            self.symbol_table.into_new_scope();

            for stmt in block.body.clone() {
                self.semantic_analysise_stmt(stmt)?;
            }

            self.symbol_table.ret_to_parent_scope();
        }

        Ok(())
    }
    fn semantic_analysise_func(&mut self) -> Result<(), Error> {
        let (name, _args, block) =
            if let Stmt::Func(ref name, ref _args, _, ref block) = *self.dummy.borrow() {
                (name.clone(), _args.clone(), block.clone())
            } else {
                return Err(Error::new_error("".to_string()));
            };

        let mut args = Vec::<Symbol>::new();
        for a in _args {
            let mut t =
                Symbol::new_argc(a.var_name.clone(), a.is_ref, self.symbol_table.get_scope());
            t.its_type.insert(a.arg_type.clone());
            args.push(t);
        }
        let fn_sym = Symbol::new_func(name.clone(), args.clone(), self.symbol_table.get_scope());

        self.symbol_table.add_symbol(fn_sym);
        self.symbol_table.into_new_scope();

        for a in args {
            self.symbol_table.add_symbol(a);
        }

        for stmt in block.body.clone() {
            if let Stmt::Return(ref _ret) = stmt {
                self.semantic_analysise_ret(&name, _ret)?;
            } else {
                self.semantic_analysise_stmt(stmt)?;
            }
        }

        self.symbol_table.ret_to_parent_scope();

        Ok(())
    }
    fn semantic_analysise_ret(
        &mut self,
        name: &String,
        _ret: &Rc<RefCell<Expr>>,
    ) -> Result<(), Error> {
        let ret = _ret.clone();
        let ret_type = Self::infer_type(self.symbol_table.clone(), &ret)?;
        self.symbol_table.add_symbol_type(&name, ret_type.clone());

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
            _ => Ok(()),
        }
    }

    pub(crate) fn semantic_analysise(&mut self) -> Result<Vec<Stmt>, Error> {
        self.symbol_table.reset();
        for stmt in self.stmts.clone() {
            self.semantic_analysise_stmt(stmt)?;
        }
        Ok(self.stmts.clone())
    }
}
