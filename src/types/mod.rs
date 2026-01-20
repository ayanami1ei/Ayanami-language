pub(crate) mod implement;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    Identifier(String),
    Operator(String),
    Keyword(String),
    Num(f64),
}

#[derive(Debug, Clone)]
pub(crate) struct Argc {
    pub(crate) is_ref: bool,
    pub(crate) arg_type: VarType,
    pub(crate) var_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Block {
    pub(crate) body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VarType {
    Int,
    Float,
    Bool,
    Char,
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    ConstNum(f64),
    ConstChar(char),
    Var(String, VarType),
    FuncCall(String, Vec<Argc>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Equal(Box<Expr>, Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    GreaterEqual(Box<Expr>, Box<Expr>),
    LessEqual(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Assign(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>, Block),
    While(Box<Expr>, Block),
    If(Box<Expr>, Block, Vec<(Box<Expr>, Block)>),
    Func(String, Vec<Argc>, VarType, Block),
    Return(Box<Expr>),
    Default,
}
