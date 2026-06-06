use crate::intern::Symbol;
use crate::parser::ast::block::Block;
use crate::parser::ast::expr::Expr;
use crate::parser::ast::ty::Type;
use crate::parser::ast::vis::Visibility;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: Symbol,
    pub self_keyword: Symbol,   // "shared" or "unique"
    pub params: Vec<(Symbol, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    FnDecl {
        vis: Visibility,
        name: Symbol,
        params: Vec<(Symbol, Type)>,
        return_type: Type,
        body: Block,
        span: Span,
    },
    Assign {
        name: Symbol,
        value: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    If {
        cond: Expr,
        then_block: Block,
        elifs: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },
    For {
        iterator: Symbol,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Block,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        span: Span,
    },
    ExprStmt {
        expr: Expr,
        span: Span,
    },
    Namespace {
        vis: Visibility,
        name: Symbol,
        items: Vec<Stmt>,
        span: Span,
    },
    StructDef {
        vis: Visibility,
        name: Symbol,
        fields: Vec<(Symbol, Type)>,
        span: Span,
    },
    InterfaceDef {
        name: Symbol,
        methods: Vec<InterfaceMethod>,
        span: Span,
    },
    ImplBlock {
        type_name: Symbol,
        methods: Vec<Stmt>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::FnDecl { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::For { span, .. }
            | Stmt::While { span, .. }
            | Stmt::ExprStmt { span, .. }
            |             Stmt::Namespace { span, .. }
            | Stmt::StructDef { span, .. }
            | Stmt::InterfaceDef { span, .. }
            | Stmt::ImplBlock { span, .. } => *span,
        }
    }
}
