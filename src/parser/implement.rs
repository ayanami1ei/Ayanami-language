use crate::{
    error_type::Error,
    parser::Parser,
    symbol_table::{self, Symbol, SymbolTable},
    types::{Argc, Block, Expr, Stmt, Token, VarType},
};

impl Block {
    pub(crate) fn new() -> Block {
        Block { body: Vec::new() }
    }

    pub(crate) fn add_stmt(&mut self, stmt: Stmt) {
        self.body.push(stmt);
    }
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Vec<Token>>, symbol_table: SymbolTable) -> Parser {
        Parser {
            tokens,
            i: 0,
            j: 0,
            symbol_table,
        }
    }

    fn next(&mut self) -> Result<(), Error> {
        self.j += 1;
        if self.j >= self.tokens[self.i].len() {
            self.j = 0;
            self.i += 1;
        }
        if self.i > self.tokens.len() {
            Err(Error::new_error("don't have enough token".to_string()))
        } else {
            Ok(())
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), Error> {
        if self.tokens[self.i][self.j] == token {
            self.next()?;
            return Ok(());
        }
        Err(Error::new_error(format!(
            "expect token: {}, but find {}, in line {}",
            token,
            self.tokens[self.i][self.j],
            self.i + 1
        )))
    }

    fn is(&mut self, token: Token) -> Result<bool, Error> {
        if self.tokens[self.i][self.j] == token {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn peek(&self) -> Token {
        self.tokens[self.i][self.j].clone()
    }

    fn parser_add_sub(&mut self) -> Result<Expr, Error> {
        let mut left = match self.parser_mul_div() {
            Ok(v) => v,
            Err(mut e) => {
                return Err(e.with_context_front(format!(
                    "line {}, index {}, when parser ",
                    self.i, self.j
                )));
            }
        };

        loop {
            if self.is(Token::Operator("+".to_string()))? {
                let right = match self.parser_mul_div() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser add ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Add(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator("-".to_string()))? {
                let right = match self.parser_mul_div() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser add ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Sub(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }
    fn parser_mul_div(&mut self) -> Result<Expr, Error> {
        let mut left = match self.parser_condition() {
            Ok(v) => v,
            Err(mut e) => {
                return Err(e.with_context_front(format!(
                    "line {}, index {}, when parser ",
                    self.i, self.j
                )));
            }
        };

        loop {
            if self.is(Token::Operator("*".to_string()))? {
                let right = match self.parser_condition() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser mul ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Mul(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator("/".to_string()))? {
                let right = match self.parser_condition() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser div ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Div(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }
    fn parser_condition(&mut self) -> Result<Expr, Error> {
        let mut left = match self.parser_primary() {
            Ok(v) => v,
            Err(mut e) => {
                return Err(e.with_context_front(format!(
                    "line {}, index {}, when parser ",
                    self.i, self.j
                )));
            }
        };

        loop {
            if self.is(Token::Operator("==".to_string()))? {
                let right = match self.parser_primary() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser equal ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Equal(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator("<".to_string()))? {
                let right = match self.parser_primary() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser less ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Less(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator(">".to_string()))? {
                let right = match self.parser_primary() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser less ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::Greater(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator("<=".to_string()))? {
                let right = match self.parser_primary() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser less ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::LessEqual(Box::new(left), Box::new(right));
            } else if self.is(Token::Operator(">=".to_string()))? {
                let right = match self.parser_primary() {
                    Ok(v) => v,
                    Err(mut e) => {
                        return Err(e.with_context_front(format!(
                            "line {}, index {}, when parser less ",
                            self.i, self.j
                        )));
                    }
                };
                left = Expr::GreaterEqual(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }
    fn parser_primary(&mut self) -> Result<Expr, Error> {
        let peek = self.peek().clone();

        if let Token::Identifier(ref name) = peek {
            self.next()?;
            return Ok(Expr::Var(name.to_string(), crate::types::VarType::Unknown));
        } else if let Token::Num(x) = peek {
            self.next()?;
            Ok(Expr::ConstNum(x))
        } else if let Token::Operator(ref op) = peek {
            if self.is(Token::Operator("(".to_string()))? {
                let res = self.parser_add_sub()?;
                self.expect(Token::Operator(")".to_string()))?;
                return Ok(res);
            }
            if self.is(Token::Operator("[".to_string()))? {
                let res = self.parser_add_sub()?;
                self.expect(Token::Operator("]".to_string()))?;
                return Ok(res);
            }

            Err(Error::new_error(format!("unknown operator: {}", op)))
        } else {
            return Err(Error::new_error(format!("unknown token: {}", peek)));
        }
    }

    fn parser_assign(&mut self) -> Result<Stmt, Error> {
        let var = self.parser_add_sub()?;
        let _a = match self.expect(Token::Operator("=".to_string())) {
            Ok(_) => 1,
            Err(mut e) => return Err(e.with_context_front("expect a statement ".to_string())),
        };
        let body = self.parser_add_sub()?;

        if self.j != 0 {
            return Err(Error::new_error(format!(
                "the line {} must end here {}",
                self.i, self.j
            )));
        }

        Ok(Stmt::Assign(Box::<Expr>::new(var), Box::<Expr>::new(body)))
    }
    fn parser_for(&mut self) -> Result<Stmt, Error> {
        if let Token::Identifier(_) = self.peek() {
            let i = self.parser_add_sub()?;
            let i_name;
            if let Expr::Var(ref _name, _) = i {
                i_name = _name.clone();
            } else {
                return Err(Error::new_error(format!("expect a var")));
            }

            self.expect(Token::Keyword("in".to_string()))?;
            self.expect(Token::Operator("(".to_string()))?;

            let start = self.parser_add_sub()?;
            if let Expr::ConstNum(_) = start {
            } else if let Expr::Var(_, _) = start {
            } else {
                return Err(Error::new_error(format!("expect a const or var")));
            }

            self.expect(Token::Operator(",".to_string()))?;
            let end = self.parser_add_sub()?;
            if let Expr::ConstNum(_) = end {
            } else if let Expr::Var(_, _) = end {
            } else {
                return Err(Error::new_error(format!("expect a const or var")));
            }

            let mut step = Expr::ConstNum(1.0);
            if self.is(Token::Operator(",".to_string()))? {
                step = self.parser_add_sub()?;
            }

            self.expect(Token::Operator(")".to_string()))?;

            self.symbol_table.into_new_scope();
            let mut i_sym = Symbol::new_var(i_name, self.symbol_table.get_scope());
            i_sym.its_type = VarType::Int;
            self.symbol_table.add_symbol(i_sym);
            self.expect(Token::Operator("{".to_string()))?;

            let mut block = Block::new();
            while !self.is(Token::Operator("}".to_string()))? {
                block.add_stmt(self.parser_stmt()?);
            }

            self.symbol_table.ret_to_parent_scope();

            Ok(Stmt::For(
                Box::<Expr>::new(i),
                Box::<Expr>::new(start),
                Box::<Expr>::new(end),
                Box::<Expr>::new(step),
                block,
            ))
        } else {
            Err(Error::new_error(format!(
                "expect an identifier, but find {}",
                self.tokens[self.i][self.j]
            )))
        }
    }
    fn parser_while(&mut self) -> Result<Stmt, Error> {
        let condition = self.parser_add_sub()?;

        self.symbol_table.into_new_scope();

        self.expect(Token::Operator("{".to_string()))?;
        let mut block = Block::new();
        while !self.is(Token::Operator("}".to_string()))? {
            block.add_stmt(self.parser_stmt()?);
        }

        self.symbol_table.ret_to_parent_scope();

        Ok(Stmt::While(Box::<Expr>::new(condition), block))
    }
    fn parser_if(&mut self) -> Result<Stmt, Error> {
        let mut condition = self.parser_add_sub()?;

        self.symbol_table.into_new_scope();

        self.expect(Token::Operator("{".to_string()))?;
        let mut block = Block::new();
        while !self.is(Token::Operator("}".to_string()))? {
            block.add_stmt(self.parser_stmt()?);
        }

        self.symbol_table.ret_to_parent_scope();

        while self.is(Token::Identifier("elif".to_string()))? {
            condition = self.parser_add_sub()?;
            self.symbol_table.into_new_scope();

            self.expect(Token::Operator("{".to_string()))?;
            let mut block = Block::new();
            while !self.is(Token::Operator("}".to_string()))? {
                block.add_stmt(self.parser_stmt()?);
            }

            self.symbol_table.ret_to_parent_scope();
        }

        if self.is(Token::Identifier("else".to_string()))?{
            self.symbol_table.into_new_scope();

            self.expect(Token::Operator("{".to_string()))?;
            let mut block = Block::new();
            while !self.is(Token::Operator("}".to_string()))? {
                block.add_stmt(self.parser_stmt()?);
            }

            self.symbol_table.ret_to_parent_scope();
        }

        Ok(Stmt::If(Box::<Expr>::new(condition), block))
    }
    fn parser_func(&mut self) -> Result<Stmt, Error> {
        if let Token::Identifier(ref name) = self.peek() {
            self.next()?;

            self.expect(Token::Operator("(".to_string()))?;
            let mut args = Vec::<Argc>::new();
            let mut arg = Argc::new();

            self.symbol_table.into_new_scope();

            loop {
                if self.is(Token::Keyword("ref".to_string()))? {
                    arg.is_ref = true;
                } else if self.is(Token::Keyword("int".to_string()))? {
                    arg.arg_type = VarType::Int;
                } else if self.is(Token::Keyword("float".to_string()))? {
                    arg.arg_type = VarType::Float;
                } else if self.is(Token::Keyword("bool".to_string()))? {
                    arg.arg_type = VarType::Bool;
                } else if self.is(Token::Keyword("char".to_string()))? {
                    arg.arg_type = VarType::Char;
                } else if let Token::Identifier(ref arg_name) = self.peek() {
                    arg.var_name = arg_name.clone();

                    args.push(arg.clone());
                    arg = Argc::new();

                    let arg_sym = Symbol::new_argc(
                        arg_name.clone(),
                        arg.is_ref,
                        self.symbol_table.get_scope(),
                    );
                    self.symbol_table.add_symbol(arg_sym);
                    self.next()?;
                } else if self.is(Token::Operator(",".to_string()))? {
                    continue;
                } else if self.is(Token::Operator(")".to_string()))? {
                    break;
                }
            }

            self.expect(Token::Operator("{".to_string()))?;
            let mut block = Block::new();
            while !self.is(Token::Operator("}".to_string()))? {
                block.add_stmt(self.parser_stmt()?);
            }

            self.symbol_table.ret_to_parent_scope();

            return Ok(Stmt::Func(name.to_string(), args, VarType::Unknown, block));
        }

        Err(Error::new_error("".to_string()))
    }
    fn parser_return(&mut self) -> Result<Stmt, Error> {
        let ret = self.parser_add_sub()?;
        Ok(Stmt::Return(Box::<Expr>::new(ret)))
    }

    fn parser_stmt(&mut self) -> Result<Stmt, Error> {
        if self.is(Token::Keyword("fn".to_string()))? {
            Ok(self.parser_func()?)
        } else if self.is(Token::Keyword("for".to_string()))? {
            Ok(self.parser_for()?)
        } else if self.is(Token::Keyword("while".to_string()))? {
            Ok(self.parser_while()?)
        } else if self.is(Token::Keyword("if".to_string()))? {
            Ok(self.parser_if()?)
        } else if self.is(Token::Keyword("return".to_string()))? {
            Ok(self.parser_return()?)
        } else {
            Ok(self.parser_assign()?)
        }
    }

    pub(crate) fn parser(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut stmts = Vec::<Stmt>::new();
        while self.i < self.tokens.len() {
            stmts.push(self.parser_stmt()?)
        }

        Ok(stmts)
    }
}
