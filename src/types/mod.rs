use std::{cell::RefCell, collections::HashSet, rc::Rc};

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
    #[allow(unused)]
    pub(crate) id: i32,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(u32)]
pub(crate) enum VarType {
    Int = 1,
    Float = 2,
    Bool = 3,
    Char = 4,
    String = 5,
    Unknown = 6,
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    ConstNum(f64, HashSet<VarType>),
    #[allow(unused)]
    ConstChar(char, HashSet<VarType>),
    Var(String, HashSet<VarType>),
    FuncCall(String, Vec<Rc<RefCell<Expr>>>, HashSet<VarType>, i32),

    Add(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    Sub(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    Mul(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    Div(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),

    Equal(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    Greater(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    Less(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    GreaterEqual(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    LessEqual(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>, HashSet<VarType>),
    #[allow(unused)]
    Not(Rc<RefCell<Expr>>, HashSet<VarType>),
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Assign(Rc<RefCell<Expr>>, Rc<RefCell<Expr>>),
    Call(String, Vec<Rc<RefCell<Expr>>>, i32),
    For(
        Rc<RefCell<Expr>>,
        Rc<RefCell<Expr>>,
        Rc<RefCell<Expr>>,
        Rc<RefCell<Expr>>,
        Block,
        i32,
    ),
    While(Rc<RefCell<Expr>>, Block, i32),
    If(
        Rc<RefCell<Expr>>,
        Block,
        Vec<(Rc<RefCell<Expr>>, Block)>,
        i32,
    ),
    Func(String, Vec<Argc>, HashSet<VarType>, Block, i32),
    Return(Rc<RefCell<Expr>>),
    Default,
}
