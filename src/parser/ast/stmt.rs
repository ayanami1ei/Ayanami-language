use crate::intern::Symbol;
use crate::parser::ast::block::Block;
use crate::parser::ast::expr::Expr;
use crate::parser::ast::ty::Type;
use crate::parser::ast::vis::Visibility;
use crate::span::Span;

/// 声明上的标注：`#[name]` / `#[name(arg, ...)]`
#[derive(Debug, Clone)]
pub struct Attr {
    /// A5a：`pkg::macro` 前缀（内置裸名时为空，`core::` 为内置别名）
    pub qualifier: Vec<Symbol>,
    pub name: Symbol,
    pub args: Vec<AttrArg>,
    pub span: Span,
}

impl Attr {
    /// 是否为编译器内置标注（裸名或 `core::` 前缀）。
    pub fn is_builtin(&self) -> bool {
        self.qualifier.is_empty()
            || (self.qualifier.len() == 1 && self.qualifier[0].as_str() == "core")
    }

    /// 完整路径文本（`inline` / `core::inline` / `pkg::macro`）。
    pub fn path_str(&self) -> String {
        let mut segs: Vec<String> = self.qualifier.iter().map(|s| s.as_str()).collect();
        segs.push(self.name.as_str());
        segs.join("::")
    }
}

/// 标注实参（A2a）：`key = value`（如 cfg(target = "linux")）或任意表达式
/// （如 requires(x > 0)、inline(always)）。
#[derive(Debug, Clone)]
pub enum AttrArg {
    KeyValue(Symbol, Box<AttrArg>),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub attrs: Vec<Attr>,
    pub name: Symbol,
    pub self_keyword: Symbol,   // "shared" or "unique"
    pub params: Vec<(Symbol, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub enum EnumFields {
    Named(Vec<(Symbol, Type)>),
    Tuple(Vec<Type>),
    None,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Symbol,
    pub fields: EnumFields,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Phase 1.3：模式（字面量 / `_` / 绑定 / 枚举变体 / 或模式）
    pub pattern: crate::parser::ast::pattern::Pattern,
    /// `if guard` 条件（可引用模式绑定）
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    FnDecl {
        attrs: Vec<Attr>,
        vis: Visibility,
        is_inline: bool,
        extern_c: bool,
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,  // (name, constraint_interface)
        params: Vec<(Symbol, Type)>,
        /// 形参标注（与 params 等长并行；A1b：noalias/nonnull）
        param_attrs: Vec<Vec<Attr>>,
        return_type: Type,
        body: Block,
        span: Span,
    },
    /// M6.2：全局变量 `static [mut] NAME [: T] = expr`（常量初始化，可寻址）
    StaticDecl {
        attrs: Vec<Attr>,
        vis: Visibility,
        is_mut: bool,
        name: Symbol,
        ty: Option<Type>,
        value: Box<Expr>,
        span: Span,
    },
    /// M6.1：编译期常量 `const NAME [: T] = expr`
    ConstDecl {
        attrs: Vec<Attr>,
        vis: Visibility,
        name: Symbol,
        ty: Option<Type>,
        value: Box<Expr>,
        span: Span,
    },
    Assign {
        name: Symbol,
        is_mut: bool,
        value: Expr,
        span: Span,
    },
    FieldAssign {
        object: Box<Expr>,
        field: Symbol,
        value: Expr,
        span: Span,
    },
    IndexAssign {
        object: Box<Expr>,
        index: Box<Expr>,
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
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
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
        attrs: Vec<Attr>,
        vis: Visibility,
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        fields: Vec<(Symbol, Type)>,
        /// A4a：字段级属性（与 fields 等长并行，如 `#[follow_with(...)]`）
        field_attrs: Vec<Vec<Attr>>,
        span: Span,
    },
    EnumDef {
        attrs: Vec<Attr>,
        vis: Visibility,
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        variants: Vec<EnumVariant>,
        span: Span,
    },
    InterfaceDef {
        attrs: Vec<Attr>,
        name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        methods: Vec<InterfaceMethod>,
        span: Span,
    },
    ImplBlock {
        attrs: Vec<Attr>,
        type_name: Symbol,
        generic_params: Vec<(Symbol, Option<Symbol>)>,
        methods: Vec<Stmt>,
        span: Span,
    },
    /// A2f：语句级标注包装（`#[cfg]`/`#[invariant]`）
    Attributed {
        attrs: Vec<Attr>,
        stmt: Box<Stmt>,
        span: Span,
    },
    Import {
        path: String,
        /// A5a：`import "pkg" { macro1, macro2 }` 的短名列表（宏展开 A5b）
        macros: Vec<Symbol>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::FnDecl { span, .. }
            |             Stmt::Assign { span, .. }
            | Stmt::FieldAssign { span, .. }
            | Stmt::IndexAssign { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::If { span, .. }
            | Stmt::For { span, .. }
            | Stmt::While { span, .. }
            | Stmt::Match { span, .. }
            | Stmt::ExprStmt { span, .. }
            |             Stmt::Namespace { span, .. }
            | Stmt::StructDef { span, .. }
            | Stmt::EnumDef { span, .. }
            | Stmt::InterfaceDef { span, .. }
            | Stmt::ImplBlock { span, .. }
            | Stmt::ConstDecl { span, .. }
            | Stmt::StaticDecl { span, .. }
            | Stmt::Import { span, .. }
            | Stmt::Attributed { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Continue { span, .. } => *span,
        }
    }
}
