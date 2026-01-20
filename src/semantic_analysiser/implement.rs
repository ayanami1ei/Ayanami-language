use std::env::args;

use crate::{
    error_type::Error,
    semantic_analysiser::SemanticAnalysiser,
    symbol_table::{self, Symbol, SymbolTable},
    types::{Expr, Stmt, VarType},
};

impl SemanticAnalysiser {
    pub(crate) fn new(stmts: Vec<Stmt>, symbol_table: SymbolTable) -> SemanticAnalysiser {
        SemanticAnalysiser {
            stmts,
            symbol_table,
            dummy: Stmt::Default,
        }
    }

    fn can_be_int(x: f64) -> bool {
        x.is_finite() && x.fract() == 0.0 && x.abs() <= (1_i64 << 53) as f64
    }

    fn infer_type(symbol_table: SymbolTable, expr: Expr) -> Result<VarType, Error> {
        if let Expr::ConstNum(x) = expr {
            #[cfg(debug_assertions)]
            {
                println!("{}", x);
            }
            if Self::can_be_int(x) {
                return Ok(VarType::Int);
            } else {
                return Ok(VarType::Float);
            }
        } else if let Expr::ConstChar(_) = expr {
            return Ok(VarType::Char);
        } else if let Expr::Var(ref name, _) = expr {
            if let Some(sym) = symbol_table.find_symbol(name) {
                if sym.its_type!=VarType::Unknown{
                    return Ok(sym.its_type)
                }
                return Err(Error::new_error(format!(
                    "cannot infer the type of var {}",
                    name
                )));
            } else {
                return Err(Error::new_error(format!(
                    "cannot infer the type of var {}",
                    name
                )));
            }
        } else if let Expr::FuncCall(name, args) = expr {
            if let Some(_) = symbol_table.find_symbol(&name) {
            } else {
                return Err(Error::new_error("undefined function".to_string()));
            }

            todo!()
        } else if let Expr::Add(_a, _b) = expr {
            let a = _a.as_ref();
            let b = _b.as_ref();

            let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
            let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;

            if type_a == VarType::Float || type_b == VarType::Float {
                return Ok(VarType::Float);
            } else if type_a == VarType::Char || type_b == VarType::Char {
                return Ok(VarType::Char);
            } else {
                return Ok(VarType::Int);
            }
        } else if let Expr::Sub(_a, _b) = expr {
            let a = _a.as_ref();
            let b = _b.as_ref();

            let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
            let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;

            if type_a == VarType::Float || type_b == VarType::Float {
                return Ok(VarType::Float);
            } else if type_a == VarType::Char || type_b == VarType::Char {
                return Ok(VarType::Char);
            } else {
                return Ok(VarType::Int);
            }
        } else if let Expr::Mul(_a, _b) = expr {
            let a = _a.as_ref();
            let b = _b.as_ref();

            let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
            let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;

            if type_a == VarType::Float || type_b == VarType::Float {
                return Ok(VarType::Float);
            } else if type_a == VarType::Char || type_b == VarType::Char {
                return Ok(VarType::Char);
            } else {
                return Ok(VarType::Int);
            }
        } else if let Expr::Div(_a, _b) = expr {
            let a = _a.as_ref();
            let b = _b.as_ref();

            let type_a = Self::infer_type(symbol_table.clone(), a.clone())?;
            let type_b = Self::infer_type(symbol_table.clone(), b.clone())?;

            if type_a == VarType::Float || type_b == VarType::Float {
                return Ok(VarType::Float);
            } else if type_a == VarType::Char || type_b == VarType::Char {
                return Ok(VarType::Char);
            } else {
                return Ok(VarType::Int);
            }
        } else if let Expr::Equal(_, _) = expr {
            return Ok(VarType::Bool);
        } else if let Expr::Greater(_, _) = expr {
            return Ok(VarType::Bool);
        } else if let Expr::Less(_, _) = expr {
            return Ok(VarType::Bool);
        } else if let Expr::GreaterEqual(_, _) = expr {
            return Ok(VarType::Bool);
        } else if let Expr::LessEqual(_, _) = expr {
            return Ok(VarType::Bool);
        } else if let Expr::Not(_) = expr {
            return Ok(VarType::Bool);
        } else {
            Err(Error::new_error("cannot infer type".to_string()))
        }
    }

    fn semantic_analysise_assign(&mut self) -> Result<(), Error> {
        if let Stmt::Assign(ref _a, ref _b) = self.dummy {
            let a = _a.as_ref().clone();
            let b = _b.as_ref().clone();

            let b_type = Self::infer_type(self.symbol_table.clone(), b.clone())?;

            if let Expr::Var(name, _) = a {
                if let Some(a_sym) = self.symbol_table.find_symbol(&name) {
                    if a_sym.its_type != b_type {
                        return Err(Error::new_error(format!(
                            "wrong type of {}, expect {}, find: {}",
                            name, a_sym.its_type, b_type
                        )));
                    }
                } else {
                    self.dummy = Stmt::Assign(
                        Box::<Expr>::new(Expr::Var(name.clone(), b_type.clone())),
                        Box::<Expr>::new(b),
                    );
                    let mut a_sym = Symbol::new_var(name, self.symbol_table.get_scope());
                    a_sym.its_type = b_type.clone();
                    self.symbol_table.add_symbol(a_sym);
                }
                Ok(())
            } else {
                Err(Error::new_error(
                    "illegal type of assign, must be var".to_string(),
                ))
            }
        } else {
            Err(Error::new_error("".to_string()))
        }
    }
    fn semantic_analysise_for(&mut self) -> Result<(), Error> {
        if let Stmt::For(ref _itor, ref _start, ref _end, ref _step, ref block) = self.dummy {
            let itor = _itor.as_ref().clone();
            let start = _start.as_ref().clone();
            let end = _end.as_ref().clone();
            let step = _step.as_ref().clone();

            if let Expr::Var(ref name, _) = itor {
                self.symbol_table.into_new_scope();
                if let Some(itor_sym) = self.symbol_table.find_symbol(name) {
                    if itor_sym.its_type != VarType::Int {
                        return Err(Error::new_error(format!(
                            "the itor must be type int, but find {}",
                            itor_sym.its_type
                        )));
                    }
                } else {
                    let mut itor_sym = Symbol::new_var(name.clone(), self.symbol_table.get_scope());
                    itor_sym.its_type = VarType::Int;
                    self.symbol_table.add_symbol(itor_sym);
                }

                if Self::infer_type(self.symbol_table.clone(), start)? != VarType::Int
                    || Self::infer_type(self.symbol_table.clone(), end)? != VarType::Int
                    || Self::infer_type(self.symbol_table.clone(), step)? != VarType::Int
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
        } else {
            Err(Error::new_error("".to_string()))
        }
    }
    fn semantic_analysise_while(&mut self) -> Result<(), Error> {
        if let Stmt::While(_cond, block) = self.dummy.clone() {
            let cond = _cond.as_ref().clone();
            if Self::infer_type(self.symbol_table.clone(), cond)? != VarType::Bool {
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
    fn semantic_analysise_if(&mut self) -> Result<(), Error> {
        if let Stmt::If(ref _cond, block, ref elifs) = self.dummy.clone() {
            let cond = _cond.as_ref().clone();
            if Self::infer_type(self.symbol_table.clone(), cond)? != VarType::Bool {
                return Err(Error::new_error("condition must be type bool".to_string()));
            }

            self.symbol_table.into_new_scope();

            for stmt in block.body.clone() {
                self.semantic_analysise_stmt(stmt)?;
            }

            self.symbol_table.ret_to_parent_scope();

            for i in 0..elifs.len() {
                let (ref _cond, ref block) = elifs[i];
                let cond = _cond.as_ref().clone();
                if Self::infer_type(self.symbol_table.clone(), cond.clone())? != VarType::Bool && 
                Self::infer_type(self.symbol_table.clone(), cond.clone())? != VarType::Int &&
                Self::infer_type(self.symbol_table.clone(), cond.clone())? != VarType::Float{
                    return Err(Error::new_error("condition must be type bool".to_string()));
                }

                self.symbol_table.into_new_scope();

                for stmt in block.body.clone() {
                    self.semantic_analysise_stmt(stmt)?;
                }

                self.symbol_table.ret_to_parent_scope();
            }
        }

        Ok(())
    }
    fn semantic_analysise_func(&mut self) -> Result<(), Error> {
        if let Stmt::Func(name, _args, ret_type, block) = self.dummy.clone() {
            let mut args = Vec::<Symbol>::new();
            for a in _args {
                let mut t = Symbol::new_argc(a.var_name, a.is_ref, self.symbol_table.get_scope());
                t.its_type = a.arg_type;
                args.push(t);
            }
            let fn_sym =
                Symbol::new_func(name.clone(), args.clone(), self.symbol_table.get_scope());

            self.symbol_table.add_symbol(fn_sym);
            self.symbol_table.into_new_scope();

            for a in args {
                self.symbol_table.add_symbol(a);
            }

            for stmt in block.body.clone() {
                if let Stmt::Return(_ret) = stmt {
                    self.semantic_analysise_ret(&name, &_ret)?;
                } else {
                    self.semantic_analysise_stmt(stmt)?;
                }
            }

            self.symbol_table.ret_to_parent_scope();
        }

        Ok(())
    }
    fn semantic_analysise_ret(&mut self, name: &String, _ret: &Box<Expr>) -> Result<(), Error> {
        let ret = _ret.as_ref().clone();
        let ret_type = Self::infer_type(self.symbol_table.clone(), ret)?;
        if let Some(_fn_sym) = self.symbol_table.find_symbol(name) {
            if ret_type != _fn_sym.its_type && _fn_sym.its_type != VarType::Unknown {
                return Err(Error::new_error(format!(
                    "return expect type {},but find {}",
                    _fn_sym.its_type, ret_type
                )));
            } else {
                self.symbol_table.set_symbol_type(&name, ret_type.clone());
            }
        } else {
            self.symbol_table.set_symbol_type(&name, ret_type.clone());
        }

        #[cfg(debug_assertions)]{
            println!("{} ret type {}",name.clone(), ret_type);
        }

        Ok(())
    }
    fn semantic_analysise_stmt(&mut self, stmt: Stmt) -> Result<(), Error> {
        self.dummy = stmt;

        match self.dummy {
            Stmt::Assign(..) => self.semantic_analysise_assign(),
            Stmt::For(..) => self.semantic_analysise_for(),
            Stmt::While(..) => self.semantic_analysise_while(),
            Stmt::If(..) => self.semantic_analysise_if(),
            Stmt::Func(..) => self.semantic_analysise_func(),
            // 其他语句类型按需要补
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
