use crate::hir::*;

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
