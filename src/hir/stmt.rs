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
    },
    FieldAssign {
        object: HirNodeBox,
        field: Symbol,
        field_index: usize,
        field_ty: HirType,
        value: HirNodeBox,
    },
    IndexAssign {
        object: HirNodeBox,
        index: HirNodeBox,
        value: HirNodeBox,
    },
    Return {
        value: Option<HirNodeBox>,
    },
    If {
        cond: HirNodeBox,
        then_block: HirBlock,
        elifs: Vec<(HirNodeBox, HirBlock)>,
        else_block: Option<HirBlock>,
    },
    While {
        cond: HirNodeBox,
        body: HirBlock,
    },
    Break,
    Continue,
    Expr(HirNodeBox),
    /// A2c：`#[assume(cond)]` → llvm.assume
    Assume(HirNodeBox),
    /// A2d/A2e：契约检查（requires/ensures/invariant；失败 abort 并报位置）
    Contract { kind: ContractKind, cond: HirNodeBox, line: usize, col: usize },
    Block(Vec<HirStmt>),
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
