use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::{
    symbol_table::SymbolTable,
    types::{Stmt, VarType},
};
pub mod implement;
pub mod life_time;
pub mod overrides;
pub mod print;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub struct VarId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub struct SlotId {
    id: i32,
    pub(crate) temp: bool,
}
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub struct FuncId(pub(crate) i32);
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Default)]
pub struct BlockId {
    pub(crate) id: i32,
    pub(crate) is_merge: bool,
}
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default, Debug, PartialOrd, Ord)]
pub struct ObjId {
    id: i32,
    home_level_id: i32,
    cur_leve_id: i32,
}
#[derive(Clone, Default)]
pub struct ObjSlot {
    set: HashSet<Value>,
    #[allow(unused)]
    home_level_id: i32,
    cur_leve_id: i32,

    last_use: usize,
    escape: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct FloatKey(pub f64);

#[derive(Default)]
pub(super) enum StorageClass {
    Local,
    Param,
    #[default]
    Temp,
}

#[derive(Default)]
struct HirVarSymbol {
    #[allow(unused)]
    pub(super) ty_set: HashSet<VarType>, // 可能指向的对象类型集合
    #[allow(unused)]
    pub(super) mutability: bool, // 能不能 Bind
    #[allow(unused)]
    pub(super) storage: StorageClass, // local / param / temp
    pub(super) obj_id: SlotId,
}

#[derive(Default, Clone)]
pub(crate) struct HirFuncSymbol {
    #[allow(unused)]
    pub(super) ty_set: HashSet<VarType>, // 可能的返回值类型集合
    pub(super) id: FuncId,
    pub(super) param_id: Vec<VarId>,
    pub(crate) is_main: bool,
    pub(crate) name: String,
}

pub struct HirGenerator {
    stmts: Vec<Stmt>,

    next_objid: ObjId,
    next_blockid: BlockId,
    next_slotid: SlotId,

    var_registry: HashMap<VarId, HirVarSymbol>,
    obj_registry: HashMap<i32, ObjId>,
    pub(crate) func_registry: HashMap<FuncId, HirFuncSymbol>,
    block_registry: HashMap<BlockId, Vec<HIRInst>>,
    slot_registry: HashMap<SlotId, ObjSlot>,
    pub(crate) var_to_slot: HashMap<VarId, SlotId>,

    ast_symbol_table: SymbolTable,

    hir: Vec<HIR>,
}

#[derive(Clone)]
pub enum HIR {
    Inst(HIRInst),
    Block(BlockId),
    FuncLabel(FuncId),
}

#[derive(Clone)]
pub enum HIRInst {
    #[allow(unused)]
    New {
        obj_type: VarType,
        val: String,
        dst: SlotId,
    },
    Delete {
        dst: SlotId,
    },
    Store {
        from: Value,
        to: SlotId,
    },
    ArraySet {
        arr: SlotId,
        idx: SlotId,
        src: SlotId,
    },
    ArrayNew {
        elems: Vec<SlotId>,
        dst: SlotId,
    },
    ArrayGet {
        arr: SlotId,
        idx: SlotId,
        dst: SlotId,
    },
    Br {
        cond: SlotId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Jmp {
        target: BlockId,
    },
    Call {
        id: FuncId,
        args: Vec<SlotId>,
        dst: SlotId,
    },
    BinOp {
        left: SlotId,
        op: BinOperator,
        right: SlotId,
        dst: SlotId,
    },
    UnaryOp {
        op: UnaryOperation,
        expr: SlotId,
        dst: SlotId,
    },
    IncRef {
        obj: SlotId,
    },
    DecRef {
        obj: SlotId,
    },
    Bind {
        var: VarId,
        obj: SlotId,
    },
    Load {
        var: VarId,
        obj: SlotId,
    },
    Ret {
        ret_obj: SlotId,
    },
    Unreachable,
}

#[derive(Clone)]
pub enum BinOperator {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    #[allow(unused)]
    And,
    #[allow(unused)]
    Or,
}

#[derive(Clone)]
pub enum UnaryOperation {
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Const {
    Int(i64),
    Float(FloatKey),
    Char(char),
    Bool(bool),
    String(String),
    #[allow(unused)]
    Null,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Const(Const),
    Obj(ObjId),
    #[default]
    Null,
}

impl From<f64> for Const {
    fn from(v: f64) -> Self {
        Const::Float(FloatKey::from(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Const(Const::from(v))
    }
}
