use std::collections::{HashMap, HashSet};

use crate::{
    symbol_table::SymbolTable,
    types::{Stmt, VarType},
};
pub(crate) mod implement;
pub(crate) mod overrides;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct VarId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct FuncId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct BlockId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct ObjId(i32);

#[derive(Default)]
pub(super) enum StorageClass {
    Local,
    Param,
    #[default]
    Temp,
}

#[derive(Default)]
struct HirVarSymbol {
    pub(super) ty_set: HashSet<VarType>, // 可能指向的对象类型集合
    pub(super) mutability: bool,         // 能不能 Bind
    pub(super) storage: StorageClass,    // local / param / temp
    pub(super) obj_id:ObjId,
}

#[derive(Default)]
struct HirFuncSymbol {
    pub(super) ty_set: HashSet<VarType>, // 可能的返回值类型集合
}

pub(crate) struct HirGenerator {
    stmts: Vec<Stmt>,

    next_objid: ObjId,
    next_tempvar_id: ObjId,
    next_blockid: BlockId,

    var_registry: HashMap<VarId, HirVarSymbol>,
    func_registry: HashMap<FuncId, HirFuncSymbol>,
    block_registry: HashMap<BlockId, Vec<HIRInst>,>,

    ast_symbol_table: SymbolTable,

    hir:Vec<HIR>
}

#[derive(Clone)]
pub(crate) enum HIR {
    Inst(HIRInst),
    Block(BlockId)
}

#[derive(Clone)]
pub(crate) enum HIRInst {
    New {
        obj_type: HashSet<VarType>,
        dst: ObjId,
    },
    Br {
        cond: ObjId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jmp {
        target: BlockId,
    },
    BinOp {
        left: ObjId,
        op: BinOperator,
        right: ObjId,
        dst: ObjId,
    },
    UnaryOp {
        op: UnaryOperation,
        expr: ObjId,
        dst: ObjId,
    },
    IncRef {
        obj: ObjId,
    },
    DecRef {
        obj: ObjId,
    },
    Bind {
        var: VarId,
        obj: ObjId,
    },
    Load {
        var: VarId,
        obj: ObjId,
    },
}

#[derive(Clone)]
pub(super) enum BinOperator {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

#[derive(Clone)]
pub(super) enum UnaryOperation {
    Not,
}
