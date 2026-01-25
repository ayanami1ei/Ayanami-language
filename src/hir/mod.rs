use std::collections::{HashMap, HashSet};

use crate::{
    symbol_table::SymbolTable,
    types::{Stmt, VarType},
};
pub(crate) mod implement;
pub(crate) mod overrides;
pub(crate) mod print;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct VarId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct FuncId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct BlockId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default, Debug)]
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
    pub(super) obj_id: Value,
}

#[derive(Default, Clone)]
struct HirFuncSymbol {
    pub(super) ty_set: HashSet<VarType>, // 可能的返回值类型集合
    pub(super) ret_obj_id: Vec<Value>,
    pub(super) id: FuncId,
    pub(super) param_id:Vec<VarId>
}

pub(crate) struct HirGenerator {
    stmts: Vec<Stmt>,

    next_objid: ObjId,
    next_tempvar_id: ObjId,
    next_blockid: BlockId,

    var_registry: HashMap<VarId, HirVarSymbol>,
    func_registry: HashMap<FuncId, HirFuncSymbol>,
    block_registry: HashMap<BlockId, Vec<HIRInst>>,

    ast_symbol_table: SymbolTable,

    hir: Vec<HIR>,
    pending_block_emits: Vec<BlockId>,
}

#[derive(Clone)]
pub(crate) enum HIR {
    Inst(HIRInst),
    Block(BlockId),
}

#[derive(Clone)]
pub(crate) enum HIRInst {
    New {
        obj_type: HashSet<VarType>,
        dst: ObjId,
    },
    Br {
        cond: Value,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jmp {
        target: BlockId,
    },
    Call {
        id: FuncId,
        ret: Value,
    },
    BinOp {
        left: Value,
        op: BinOperator,
        right: Value,
        dst: ObjId,
    },
    UnaryOp {
        op: UnaryOperation,
        expr: Value,
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
        obj: Value,
    },
    Load {
        var: VarId,
        obj: Value,
    },
    Ret {
        ret_obj: Value,
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

#[derive(Debug, Clone, Copy)]
pub(super) enum Const{
    Int(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Null
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) enum Value{
    Const(Const),
    Obj(ObjId),
    #[default]
    Null
}