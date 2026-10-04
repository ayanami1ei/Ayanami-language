use crate::span::Span;
use crate::hir::*;

/// 契约种类（A2d/A2e/A2f）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    Require,
    Ensure,
    Invariant,
}

impl ContractKind {
    pub fn label(&self) -> &'static str {
        match self {
            ContractKind::Require => "require",
            ContractKind::Ensure => "ensure",
            ContractKind::Invariant => "invariant",
        }
    }

    /// runtime.c 中的失败处理函数（noreturn）。
    pub fn runtime_fn(&self) -> &'static str {
        match self {
            ContractKind::Require => "__ayanami_require_fail",
            ContractKind::Ensure => "__ayanami_ensure_fail",
            ContractKind::Invariant => "__ayanami_invariant_fail",
        }
    }
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Assign {
        target: HirNodeBox,
        value: HirNodeBox,
        span: Span,
    },
    FieldAssign {
        object: HirNodeBox,
        field: Symbol,
        field_index: usize,
        field_ty: HirType,
        value: HirNodeBox,
        span: Span,
    },
    IndexAssign {
        object: HirNodeBox,
        index: HirNodeBox,
        value: HirNodeBox,
        span: Span,
    },
    /// `ref mut` 目标写入：穿透引用存储（`i = v`，i 为 ref mut 参数/局部）
    DerefAssign {
        target: HirNodeBox,
        value: HirNodeBox,
        span: Span,
    },
    Return {
        value: Option<HirNodeBox>,
        span: Span,
    },
    If {
        cond: HirNodeBox,
        then_block: HirBlock,
        elifs: Vec<(HirNodeBox, HirBlock)>,
        else_block: Option<HirBlock>,
        span: Span,
    },
    While {
        cond: HirNodeBox,
        body: HirBlock,
        span: Span,
    },
    Break { span: Span },
    Continue { span: Span },
    Expr { expr: HirNodeBox, span: Span },
    /// A2c：`#[assume(cond)]` → llvm.assume
    Assume { cond: HirNodeBox, span: Span },
    /// A2d/A2e：契约检查（requires/ensures/invariant；失败 abort 并报位置）
    Contract { kind: ContractKind, cond: HirNodeBox, line: usize, col: usize },
    Block { stmts: Vec<HirStmt>, span: Span },
}

impl HirStmt {
    /// 语句源码位置（合成语句为默认 Span，line/col 为 0）
    pub fn span(&self) -> Span {
        match self {
            HirStmt::Assign { span, .. }
            | HirStmt::FieldAssign { span, .. }
            | HirStmt::IndexAssign { span, .. }
            | HirStmt::DerefAssign { span, .. }
            | HirStmt::Return { span, .. }
            | HirStmt::If { span, .. }
            | HirStmt::While { span, .. }
            | HirStmt::Break { span }
            | HirStmt::Continue { span }
            | HirStmt::Expr { span, .. }
            | HirStmt::Assume { span, .. }
            | HirStmt::Block { span, .. } => *span,
            HirStmt::Contract { line, col, .. } => Span {
                start_line: *line,
                start_col: *col,
                end_line: *line,
                end_col: *col,
                start_byte: 0,
                end_byte: 0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
}

impl HirBlock {
    pub fn new(stmts: Vec<HirStmt>) -> Self {
        Self { stmts }
    }
}
