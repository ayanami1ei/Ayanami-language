use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::{
    symbol_table::SymbolTable,
    types::{Stmt, VarType},
};
pub(crate) mod implement;
pub(crate) mod life_time;
pub(crate) mod overrides;
pub(crate) mod print;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct VarId(i32);
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct SlotId {
    id: i32,
}
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct FuncId(i32);
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Default)]
pub(crate) struct BlockId {
    id: i32,
    is_merge: bool,
}
#[derive(Clone, Copy, Eq, Hash, PartialEq, Default, Debug, PartialOrd, Ord)]
pub(crate) struct ObjId {
    id: i32,
    home_level_id: i32,
    cur_leve_id: i32,
}
#[derive(Clone, Default)]
pub(crate) struct ObjSlot {
    set: HashSet<Value>,
    home_level_id: i32,
    cur_leve_id: i32,

    last_use:usize,
    escape:bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FloatKey(pub(crate) f64);

impl From<f64> for FloatKey {
    fn from(v: f64) -> Self {
        FloatKey(v)
    }
}

impl PartialEq for FloatKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for FloatKey {}

impl Hash for FloatKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}

impl PartialOrd for FloatKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FloatKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

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
    pub(super) obj_id: SlotId,
}

#[derive(Default, Clone)]
struct HirFuncSymbol {
    pub(super) ty_set: HashSet<VarType>, // 可能的返回值类型集合
    pub(super) ret_obj_id: SlotId,
    pub(super) id: FuncId,
    pub(super) param_id: Vec<VarId>,
}

pub(crate) struct HirGenerator {
    stmts: Vec<Stmt>,

    next_objid: ObjId,
    next_blockid: BlockId,
    next_slotid: SlotId,

    var_registry: HashMap<VarId, HirVarSymbol>,
    obj_registry: HashMap<i32, ObjId>,
    func_registry: HashMap<FuncId, HirFuncSymbol>,
    block_registry: HashMap<BlockId, Vec<HIRInst>>,
    slot_registry: HashMap<SlotId, ObjSlot>,

    ast_symbol_table: SymbolTable,

    hir: Vec<HIR>,
}

#[derive(Clone)]
pub(crate) enum HIR {
    Inst(HIRInst),
    Block(BlockId),
    Func(FuncId),
}

#[derive(Clone)]
pub(crate) enum HIRInst {
    New {
        obj_type: HashSet<VarType>,
        dst: ObjId,
    },
    Delete {
        dst: SlotId,
    },
    Store {
        from: Value,
        to: SlotId,
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
        ret: SlotId,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Const {
    Int(i64),
    Float(FloatKey),
    Char(char),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Value {
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
