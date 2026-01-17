use crate::{symbol_table::SymbolTable, types::Stmt};
pub(crate) mod implement;

pub type SymbolId = i32;
pub type TempId = usize;
pub type HirExprId = usize;
pub type HirStmtId = usize;
pub type HirBlockId = usize;

#[derive(Debug, Clone)]
pub enum HirExpr {
    ConstNum(f64),
    ConstChar(char),
    Var(SymbolId),
    Binary {
        op: BinOp,
        lhs: HirExprId,
        rhs: HirExprId,
    },
    Unary {
        op: UnOp,
        expr: HirExprId,
    },
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Assign {
        lhs: SymbolId,
        rhs: HirExprId,
    },
    If {
        cond: HirExprId,
        then_block: HirBlockId,
        else_block: Option<HirBlockId>,
    },
    While {
        cond: HirExprId,
        body: HirBlockId,
    },
    For {
        iter: SymbolId,
        start: HirExprId,
        end: HirExprId,
        step: HirExprId,
        body: HirBlockId,
    },
    Func{
        params:Vec<HirExpr>,
        body:HirBlockId,
        ret_type:
    },
    Return(Option<HirExprId>),
    Expr(HirExprId),
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmtId>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Temp(TempId),
    Symbol(SymbolId),
    ConstNum(f64),
    ConstChar(char),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Assign {
        dst: Value,
        src: Value,
    },
    BinOp {
        dst: TempId,
        op: BinOp,
        lhs: Value,
        rhs: Value,
    },
    UnaryOp {
        dst: TempId,
        op: UnOp,
        expr: Value,
    },
    Label(String),
    Jmp(String),
    If(Value),
    Return(Option<Value>),
    Param(Value),
    Default,
}

pub(crate) struct HIR {
    pub exprs: Vec<HirExpr>,
    pub stmts: Vec<HirStmt>,
    pub blocks: Vec<HirBlock>,
    pub symbol_table: SymbolTable,
    t_index: usize,
    for_index: usize,
    while_index: usize,
    if_index: usize,
}
