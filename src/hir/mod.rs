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
pub(crate) struct BlockId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct ObjId(i32);

#[derive(Debug, Default)]
pub(super) enum StorageClass {
    Local,
    Param,
    #[default]
    Temp,
}

#[derive(Debug, Default)]
struct HirSymbol {
    pub(super) ty_set: HashSet<VarType>, // 可能指向的对象类型集合
    pub(super) mutability: bool,         // 能不能 Bind
    pub(super) storage: StorageClass,    // local / param / temp
}

pub(crate) struct HirGenerator {
    stmts: Vec<Stmt>,

    next_objid: ObjId,
    next_tempvar_id: ObjId,

    var_registry: HashMap<VarId, HirSymbol>,
    obj_registry: HashMap<ObjId, HirSymbol>,
    block_registry: HashMap<BlockId, HirSymbol>,

    ast_symbol_table: SymbolTable,
}

pub(crate) enum HIRInst {
    New {
        obj_type: VarType,
        dst: ObjId,
    },
    Br {
        cond: VarId,
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
        dst:ObjId,
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
}

pub(super) enum UnaryOperation {
    Not
}
