use crate::{symbol_table::SymbolTable, types::{Stmt, VarType}};
pub(crate) mod implement;

pub(crate) struct VarId(i32);
pub(crate) struct BlockId(i32);
pub(crate) struct ObjId(i32);

pub(crate) enum HIRInst {
    New{
        obj_type:VarType,
        dst:ObjId,
    },
    Br{
        cond:VarId,
        then_block:BlockId,
        else_block:BlockId,
    },
    Jmp{
        target:BlockId,
    },
    BinOp{
        left:VarId,
        op:String,
        right:VarId
    },
    IncRef {
        obj: ObjId,
    },
    DecRef {
        obj: ObjId,
    },
    
}