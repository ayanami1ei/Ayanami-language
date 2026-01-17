use core::fmt;

use crate::{
    error_type::Error,
    hir::{
        BinOp, HIR, HirBlock, HirBlockId, HirExpr, HirExprId, HirStmt, HirStmtId, Instruction,
        SymbolId, TempId, UnOp, Value,
    },
    symbol_table::SymbolTable,
    types::{Block, Expr, Stmt},
};

impl HIR {
    fn ast_to_hir_expr(&mut self, expr: &Expr) -> Result<HirExprId, Error> {
        let hir_expr = match expr {
            Expr::ConstNum(n) => HirExpr::ConstNum(*n),
            Expr::ConstChar(c) => HirExpr::ConstChar(*c),
            Expr::Var(name, _) => {
                let symbol = self
                    .symbol_table
                    .find_symbol(name)
                    .ok_or_else(|| Error::new_error("Symbol not found".to_string()))?;
                HirExpr::Var(symbol.id)
            }
            Expr::Add(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Add,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Sub(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Sub,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Mul(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Mul,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Div(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Div,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Equal(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Equal,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Greater(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Greater,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Less(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::Less,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::GreaterEqual(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::GreaterEqual,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::LessEqual(lhs, rhs) => {
                let lhs_id = self.ast_to_hir_expr(lhs)?;
                let rhs_id = self.ast_to_hir_expr(rhs)?;
                HirExpr::Binary {
                    op: BinOp::LessEqual,
                    lhs: lhs_id,
                    rhs: rhs_id,
                }
            }
            Expr::Not(e) => {
                let expr_id = self.ast_to_hir_expr(e)?;
                HirExpr::Unary {
                    op: UnOp::Not,
                    expr: expr_id,
                }
            }
            _ => return Err(Error::new_error("Unsupported expr".to_string())),
        };
        self.exprs.push(hir_expr);
        Ok(self.exprs.len() - 1)
    }

    fn ast_to_hir_stmt(&mut self, stmt: &Stmt) -> Result<HirStmtId, Error> {
        let hir_stmt = match stmt {
            Stmt::Assign(lhs, rhs) => {
                if let Expr::Var(name, _) = lhs.as_ref() {
                    let symbol = self
                        .symbol_table
                        .find_symbol(name)
                        .ok_or_else(|| Error::new_error("Symbol not found".to_string()))?;
                    let rhs_id = self.ast_to_hir_stmt_expr(rhs)?;
                    HirStmt::Assign {
                        lhs: symbol.id,
                        rhs: rhs_id,
                    }
                } else {
                    return Err(Error::new_error("Invalid assign lhs".to_string()));
                }
            }
            Stmt::If(cond, block) => {
                let cond_id = self.ast_to_hir_expr(cond)?;
                let then_block_id = self.ast_to_hir_block(block)?;
                HirStmt::If {
                    cond: cond_id,
                    then_block: then_block_id,
                    else_block: None,
                }
            }
            Stmt::While(cond, block) => {
                let cond_id = self.ast_to_hir_expr(cond)?;
                let body_id = self.ast_to_hir_block(block)?;
                HirStmt::While {
                    cond: cond_id,
                    body: body_id,
                }
            }
            Stmt::For(itor, start, end, step, block) => {
                if let Expr::Var(name, _) = itor.as_ref() {
                    let symbol = self
                        .symbol_table
                        .find_symbol(name)
                        .ok_or_else(|| Error::new_error("Symbol not found".to_string()))?;
                    let start_id = self.ast_to_hir_expr(start)?;
                    let end_id = self.ast_to_hir_expr(end)?;
                    let step_id = self.ast_to_hir_expr(step)?;
                    let body_id = self.ast_to_hir_block(block)?;
                    HirStmt::For {
                        iter: symbol.id,
                        start: start_id,
                        end: end_id,
                        step: step_id,
                        body: body_id,
                    }
                } else {
                    return Err(Error::new_error("Invalid for iter".to_string()));
                }
            }
            Stmt::Return(expr) => {
                let expr_id = self.ast_to_hir_expr(expr)?;
                HirStmt::Return(Some(expr_id))
            }
            _ => return Err(Error::new_error("Unsupported stmt".to_string())),
        };
        self.stmts.push(hir_stmt);
        Ok(self.stmts.len() - 1)
    }

    fn ast_to_hir_stmt_expr(&mut self, expr: &Expr) -> Result<HirExprId, Error> {
        self.ast_to_hir_expr(expr)
    }

    fn ast_to_hir_block(&mut self, block: &Block) -> Result<HirBlockId, Error> {
        let mut stmt_ids = Vec::new();
        for stmt in &block.body {
            let id = self.ast_to_hir_stmt(stmt)?;
            stmt_ids.push(id);
        }
        let hir_block = HirBlock { stmts: stmt_ids };
        self.blocks.push(hir_block);
        Ok(self.blocks.len() - 1)
    }

    pub(crate) fn new(ast_stmts: Vec<Stmt>, symbol_table: SymbolTable) -> Result<HIR, Error> {
        let mut hir = HIR {
            exprs: Vec::new(),
            stmts: Vec::new(),
            blocks: Vec::new(),
            symbol_table,
            t_index: 0,
            for_index: 0,
            while_index: 0,
            if_index: 0,
        };
        let _root_block_id = hir.ast_to_hir_block(&Block { body: ast_stmts })?;
        Ok(hir)
    }

    fn gen_expr_ir(&mut self, expr_id: HirExprId) -> Result<(Value, Vec<Instruction>), Error> {
        let expr = self.exprs[expr_id].clone();
        match expr {
            HirExpr::ConstNum(n) => {
                self.t_index += 1;
                let temp = Value::Temp(self.t_index);
                let instr = Instruction::Assign {
                    dst: temp.clone(),
                    src: Value::ConstNum(n),
                };
                Ok((temp, vec![instr]))
            }
            HirExpr::ConstChar(c) => {
                self.t_index += 1;
                let temp = Value::Temp(self.t_index);
                let instr = Instruction::Assign {
                    dst: temp.clone(),
                    src: Value::ConstChar(c),
                };
                Ok((temp, vec![instr]))
            }
            HirExpr::Var(symbol_id) => {
                self.t_index += 1;
                let temp = Value::Temp(self.t_index);
                let instr = Instruction::Assign {
                    dst: temp.clone(),
                    src: Value::Symbol(symbol_id),
                };
                Ok((temp, vec![instr]))
            }
            HirExpr::Binary { op, lhs, rhs } => {
                let (lhs_val, lhs_instrs) = self.gen_expr_ir(lhs)?;
                let (rhs_val, rhs_instrs) = self.gen_expr_ir(rhs)?;
                self.t_index += 1;
                let temp = Value::Temp(self.t_index);
                let instr = Instruction::BinOp {
                    dst: self.t_index,
                    op,
                    lhs: lhs_val,
                    rhs: rhs_val,
                };
                let mut instrs = lhs_instrs;
                instrs.extend(rhs_instrs);
                instrs.push(instr);
                Ok((temp, instrs))
            }
            HirExpr::Unary { op, expr } => {
                let (expr_val, expr_instrs) = self.gen_expr_ir(expr)?;
                self.t_index += 1;
                let temp = Value::Temp(self.t_index);
                let instr = Instruction::UnaryOp {
                    dst: self.t_index,
                    op,
                    expr: expr_val,
                };
                let mut instrs = expr_instrs;
                instrs.push(instr);
                Ok((temp, instrs))
            }
        }
    }

    fn gen_stmt_ir(&mut self, stmt_id: HirStmtId) -> Result<Vec<Instruction>, Error> {
        let stmt = self.stmts[stmt_id].clone();
        match stmt {
            HirStmt::Assign { lhs, rhs } => {
                let (rhs_val, instrs) = self.gen_expr_ir(rhs)?;
                let mut instrs = instrs;
                instrs.push(Instruction::Assign {
                    dst: Value::Symbol(lhs),
                    src: rhs_val,
                });
                Ok(instrs)
            }
            HirStmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let (cond_val, cond_instrs) = self.gen_expr_ir(cond)?;
                let mut instrs = cond_instrs;
                let label_then = format!("if_{}", self.if_index);
                let label_end = format!("if_end_{}", self.if_index);
                self.if_index += 1;
                instrs.push(Instruction::If(cond_val));
                instrs.push(Instruction::Jmp(label_then.clone()));
                if let Some(else_block_id) = else_block {
                    let else_instrs = self.gen_block_ir(else_block_id)?;
                    instrs.extend(else_instrs);
                }
                instrs.push(Instruction::Jmp(label_end.clone()));
                instrs.push(Instruction::Label(label_then));
                let then_instrs = self.gen_block_ir(then_block)?;
                instrs.extend(then_instrs);
                instrs.push(Instruction::Label(label_end));
                Ok(instrs)
            }
            HirStmt::While { cond, body } => {
                let label_start = format!("while_{}", self.while_index);
                let label_end = format!("while_end_{}", self.while_index);
                self.while_index += 1;
                let mut instrs = vec![Instruction::Label(label_start.clone())];
                let (cond_val, cond_instrs) = self.gen_expr_ir(cond)?;
                instrs.extend(cond_instrs);
                instrs.push(Instruction::If(cond_val));
                instrs.push(Instruction::Jmp(label_end.clone()));
                let body_instrs = self.gen_block_ir(body)?;
                instrs.extend(body_instrs);
                instrs.push(Instruction::Jmp(label_start));
                instrs.push(Instruction::Label(label_end));
                Ok(instrs)
            }
            HirStmt::For {
                iter,
                start,
                end,
                step,
                body,
            } => {
                let label_start = format!("for_{}", self.for_index);
                let label_end = format!("for_end_{}", self.for_index);
                self.for_index += 1;
                let mut instrs = Vec::new();
                // Assign start to iter
                let (start_val, start_instrs) = self.gen_expr_ir(start)?;
                instrs.extend(start_instrs);
                instrs.push(Instruction::Assign {
                    dst: Value::Symbol(iter),
                    src: start_val,
                });
                instrs.push(Instruction::Label(label_start.clone()));
                // Check condition
                let (end_val, end_instrs) = self.gen_expr_ir(end)?;
                instrs.extend(end_instrs);
                self.t_index += 1;
                let cond_temp = Value::Temp(self.t_index);
                instrs.push(Instruction::BinOp {
                    dst: self.t_index,
                    op: BinOp::Less,
                    lhs: Value::Symbol(iter),
                    rhs: end_val,
                });
                instrs.push(Instruction::If(cond_temp));
                instrs.push(Instruction::Jmp(label_end.clone()));
                // Body
                let body_instrs = self.gen_block_ir(body)?;
                instrs.extend(body_instrs);
                // Increment
                let (step_val, step_instrs) = self.gen_expr_ir(step)?;
                instrs.extend(step_instrs);
                self.t_index += 1;
                let new_iter_temp = Value::Temp(self.t_index);
                instrs.push(Instruction::BinOp {
                    dst: self.t_index,
                    op: BinOp::Add,
                    lhs: Value::Symbol(iter),
                    rhs: step_val,
                });
                instrs.push(Instruction::Assign {
                    dst: Value::Symbol(iter),
                    src: new_iter_temp,
                });
                instrs.push(Instruction::Jmp(label_start));
                instrs.push(Instruction::Label(label_end));
                Ok(instrs)
            }
            HirStmt::Return(expr_id) => {
                if let Some(expr_id) = expr_id {
                    let (val, instrs) = self.gen_expr_ir(expr_id)?;
                    let mut instrs = instrs;
                    instrs.push(Instruction::Return(Some(val)));
                    Ok(instrs)
                } else {
                    Ok(vec![Instruction::Return(None)])
                }
            }
            HirStmt::Expr(expr_id) => {
                let (_, instrs) = self.gen_expr_ir(expr_id)?;
                Ok(instrs)
            }
        }
    }

    fn gen_block_ir(&mut self, block_id: HirBlockId) -> Result<Vec<Instruction>, Error> {
        let block = self.blocks[block_id].clone();
        let mut instrs = Vec::new();
        for stmt_id in block.stmts {
            instrs.extend(self.gen_stmt_ir(stmt_id)?);
        }
        Ok(instrs)
    }

    pub(crate) fn gen_ir(&mut self) -> Result<Vec<Instruction>, Error> {
        if self.blocks.is_empty() {
            return Ok(Vec::new());
        }
        self.gen_block_ir(0) // Assume root block is 0
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Temp(id) => write!(f, "t{}", id),
            Value::Symbol(id) => write!(f, "s{}", id),
            Value::ConstNum(n) => write!(f, "{}", n),
            Value::ConstChar(c) => write!(f, "'{}'", c),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Assign { dst, src } => write!(f, "    {} = {}", dst, src),
            Instruction::BinOp { dst, op, lhs, rhs } => {
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Equal => "==",
                    BinOp::Greater => ">",
                    BinOp::Less => "<",
                    BinOp::GreaterEqual => ">=",
                    BinOp::LessEqual => "<=",
                };
                write!(f, "    t{} = {} {} {}", dst, lhs, op_str, rhs)
            }
            Instruction::UnaryOp { dst, op, expr } => {
                let op_str = match op {
                    UnOp::Not => "!",
                };
                write!(f, "    t{} = {}{}", dst, op_str, expr)
            }
            Instruction::Label(label) => write!(f, "{}:", label),
            Instruction::Jmp(label) => write!(f, "    jmp {}", label),
            Instruction::If(cond) => write!(f, "    if {}", cond),
            Instruction::Return(val) => match val {
                Some(v) => write!(f, "    ret {}", v),
                None => write!(f, "    ret"),
            },
            Instruction::Param(val) => write!(f, "    param {}", val),
            Instruction::Default => write!(f, ""),
        }
    }
}
