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
    /// bbp：if 表达式（分支块尾作为值）
    If {
        cond: Box<Expr>,
        then_block: crate::parser::ast::block::Block,
        elifs: Vec<(Expr, crate::parser::ast::block::Block)>,
        else_block: Option<crate::parser::ast::block::Block>,
        span: Span,
    },
    /// M1.3：显式类型转换 `expr as T`
    Cast {
        expr: Box<Expr>,
        ty: Type,
        span: Span,
    },
    Literal(Literal),
    /// M1.5：带类型后缀的字面量（`1u8` / `1.5f32`）
    Suffixed {
        lit: Literal,
        suffix: Symbol,
        span: Span,
    },
    Ident(Symbol, Span),
    FnCall {
        name: Symbol,
        args: Vec<Expr>,
        /// 显式泛型实参（`ns.fn[T1, T2](...)`；普通调用为空）
        generic_args: Vec<Type>,
        span: Span,
    },
    Move(Box<Expr>, Span),
    Clone(Box<Expr>, Span),
    ToUnique(Box<Expr>, Span),
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
    /// M6.2c：重复字面量 `[value; count]`（常量上下文 → 数组常量）
    ArrayRepeat {
        value: Box<Expr>,
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
    Match {
        value: Box<Expr>,
        arms: Vec<crate::parser::ast::stmt::MatchArm>,
        span: Span,
    },
    EnumConstruct {
        enum_name: Symbol,
        variant_name: Symbol,
        tuple_args: Vec<Expr>,
        named_args: Vec<(Symbol, Expr)>,
        span: Span,
    },
    Lambda {
        params: Vec<(Symbol, Type)>,
        return_type: Type,
        body: Vec<crate::parser::ast::stmt::Stmt>,
        span: Span,
    },
    /// A5c-2 函数宏调用：`#name(args)`（name 可含 `pkg::` 前缀，解析后以 `.` 连接）
    MacroCall {
        name: Symbol,
        args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary { span, .. }
            | Expr::If { span, .. }
            | Expr::Cast { span, .. }
            | Expr::Suffixed { span, .. }
            | Expr::Unary { span, .. }
            | Expr::FnCall { span, .. }
            |             Expr::Move(_, span)
            | Expr::Clone(_, span)
            | Expr::ToUnique(_, span)
            | Expr::MethodCall { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::StructLiteral { span, .. }
            | Expr::ArrayLiteral(_, span)
            | Expr::ArrayRepeat { span, .. }
            | Expr::ArraySized { span, .. }
            | Expr::Ref(_, _, span)
            | Expr::Null(span)
            | Expr::Asm { span, .. }
            | Expr::Index { span, .. }
            | Expr::CallExpr { span, .. }
            | Expr::TryOp(_, span) => *span,
            | Expr::MacroCall { span, .. } => *span,
            | Expr::Match { span, .. } => *span,
            | Expr::EnumConstruct { span, .. } => *span,
            | Expr::Lambda { span, .. } => *span,
            Expr::Literal(lit) => lit.span(),
            Expr::Ident(_, span) => *span,
        }
    }
}
