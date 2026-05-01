use crate::intern::Symbol;
use crate::parser::ast::binary_op::BinaryOp;
use crate::parser::ast::literal::Literal;
use crate::parser::ast::unary_op::UnaryOp;
use crate::span::Span;

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
