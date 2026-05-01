use crate::intern::Symbol;
use crate::span::Span;

// ── 运算符 ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ── 字面量 ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64, Span),
    Float(f64, Span),
    Char(char, Span),
    String(String, Span),
    Bool(bool, Span),
}

impl Literal {
    pub fn span(&self) -> Span {
        match self {
            Literal::Int(_, s)
            | Literal::Float(_, s)
            | Literal::Char(_, s)
            | Literal::String(_, s)
            | Literal::Bool(_, s) => *s,
        }
    }
}

// ── 类型 ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Type {
    Int(Span),
    Float(Span),
    Char(Span),
    Void(Span),
    Named(Symbol, Span),
    Ref(Box<Type>, Span),
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s)
            | Type::Float(s)
            | Type::Char(s)
            | Type::Void(s)
            | Type::Named(_, s)
            | Type::Ref(_, s) => *s,
        }
    }
}

// ── 表达式 ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        arg: Box<Expr>,
        span: Span,
    },
    Literal(Literal),
    Ident(Symbol, Span),
    FnCall {
        name: Symbol,
        args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::FnCall { span, .. } => *span,
            Expr::Literal(lit) => lit.span(),
            Expr::Ident(_, span) => *span,
        }
    }
}
