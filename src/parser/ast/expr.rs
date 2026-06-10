use crate::intern::Symbol;
use crate::parser::ast::binary_op::BinaryOp;
use crate::parser::ast::literal::Literal;
use crate::parser::ast::ty::Type;
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
    Move(Box<Expr>, Span),
    Clone(Box<Expr>, Span),
    ToUnique(Box<Expr>, Span),
    ToShared(Box<Expr>, Span),
    ToWeak(Box<Expr>, Span),
    MethodCall {
        object: Box<Expr>,
        method: Symbol,
        args: Vec<Expr>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field: Symbol,
        span: Span,
    },
    StructLiteral {
        type_name: Symbol,
        generic_args: Vec<Type>,
        fields: Vec<(Symbol, Expr)>,
        span: Span,
    },
    ArrayLiteral(Vec<Expr>, Span),
    ArraySized {
        elem_type: Type,
        count: Box<Expr>,
        span: Span,
    },
    Null(Span),  // null literal
    Ref(Box<Expr>, bool, Span),  // ref expr or ref mut expr
    Asm {
        template: String,
        outputs: Vec<(String, Box<Expr>)>,  // constraint, place to store
        inputs: Vec<(String, Box<Expr>)>,   // constraint, value
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    CallExpr {
        target: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    TryOp(Box<Expr>, Span),  // expr?
    EnumConstruct {
        enum_name: Symbol,
        variant_name: Symbol,
        tuple_args: Vec<Expr>,
        named_args: Vec<(Symbol, Expr)>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::FnCall { span, .. }
            |             Expr::Move(_, span)
            | Expr::Clone(_, span)
            | Expr::ToUnique(_, span)
            | Expr::ToShared(_, span)
            | Expr::ToWeak(_, span)
            | Expr::MethodCall { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::StructLiteral { span, .. }
            | Expr::ArrayLiteral(_, span)
            | Expr::ArraySized { span, .. }
            | Expr::Ref(_, _, span)
            | Expr::Null(span)
            | Expr::Asm { span, .. }
            | Expr::Index { span, .. }
            | Expr::CallExpr { span, .. }
            | Expr::TryOp(_, span) => *span,
            | Expr::EnumConstruct { span, .. } => *span,
            Expr::Literal(lit) => lit.span(),
            Expr::Ident(_, span) => *span,
        }
    }
}
