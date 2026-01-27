use std::fmt;

use crate::hir::{
    BinOperator, BlockId, Const, FloatKey, FuncDef, FuncId, HIR, HIRInst, ObjId, ObjSlot, SlotId, UnaryOperation, Value, VarId
};

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BlockId { id, is_merge } = self;
        if *is_merge {
            return write!(f, "merge_{}", id);
        }
        write!(f, "{}", id)
    }
}
impl fmt::Display for ObjId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ObjId {
            id,
            home_level_id,
            cur_leve_id,
        } = self;
        write!(
            f,
            "(obj_{}, home_level_id: {}, cur_leve_id: {})",
            id, home_level_id, cur_leve_id
        )
    }
}
impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let VarId(id) = self;
        write!(f, "{}", id)
    }
}
impl fmt::Display for FuncId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FuncId(id) = self;
        write!(f, "{}", id)
    }
}
impl fmt::Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Const::Int(x) => write!(f, "Int: {}", x),
            Const::Float(x) => write!(f, "Float: {}", x),
            Const::Char(x) => write!(f, "Char: {}", x),
            Const::Bool(x) => write!(f, "Bool: {}", x),
            Const::Null => write!(f, ""),
        }
    }
}
impl fmt::Display for FloatKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FloatKey(x) = self;
        write!(f, "{}", x)
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SlotId { id } = self;
        write!(f, "slot_{}", id)
    }
}
impl fmt::Display for ObjSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.set.clone() {
            write!(f, "{}, ", i)?;
        }
        write!(f, "")
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Const(x) => write!(f, "Const_{}", x),
            Value::Obj(obj_id) => write!(f, "{}", obj_id),
            Value::Null => todo!(),
        }
    }
}

impl fmt::Display for FuncDef{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            FuncDef::Start(func_id) => write!(f,"{} start", func_id),
            FuncDef::End(func_id) => write!(f,"{} end", func_id),
        }
    }
}

impl fmt::Display for BinOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOperator::Add => write!(f, "+"),
            BinOperator::Sub => write!(f, "-"),
            BinOperator::Mul => write!(f, "*"),
            BinOperator::Div => write!(f, "/"),
            BinOperator::Equal => write!(f, "=="),
            BinOperator::Greater => write!(f, ">"),
            BinOperator::Less => write!(f, "<"),
            BinOperator::GreaterEqual => write!(f, ">="),
            BinOperator::LessEqual => write!(f, "<="),
            BinOperator::And => write!(f, "&&"),
            BinOperator::Or => write!(f, "||"),
        }
    }
}
impl fmt::Display for UnaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperation::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for HIRInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "    ")?;
        match self {
            HIRInst::New { obj_type, dst } => {
                write!(f, "New ")?;

                for i in obj_type {
                    write!(f, "{} ", i)?;
                }

                write!(f, "-> {}", dst)
            }
            HIRInst::Delete { dst } => {
                write!(f, "//Delete {}", dst)
            }
            HIRInst::Store { from, to } => write!(f, "Store {} -> {}", from, to),
            HIRInst::Br {
                cond,
                then_block,
                else_block,
            } => write!(
                f,
                "Br cond: {}, then-block: block_{}, else-block: block_{}",
                cond, then_block, else_block
            ),
            HIRInst::Jmp { target } => write!(f, "Jmp to block_{}", target),
            HIRInst::Call { id, ret } => {
                write!(f, "Call func_{}, ret_slot:{}", id, ret)?;
                write!(f, "")
            }
            HIRInst::BinOp {
                left,
                op,
                right,
                dst,
            } => write!(f, "{} {} {} -> {}", left, op, right, dst),
            HIRInst::UnaryOp { op, expr, dst } => write!(f, "{} {} -> {}", op, expr, dst),
            HIRInst::Bind { var, obj } => write!(f, "Bind var_{} and {}", var, obj),
            HIRInst::Load { var, obj } => write!(f, "Load var_{}'s obj to {}", var, obj),
            HIRInst::Ret { ret_obj } => write!(f, "Ret {}", ret_obj),
            HIRInst::IncRef { obj } => write!(f, "Increase Reference of {}", obj),
            HIRInst::DecRef { obj } => write!(f, "Decrease Reference of {}", obj),
        }
    }
}

impl fmt::Display for HIR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HIR::Inst(hirinst) => write!(f, "    {}", hirinst),
            HIR::Block(block_id) => write!(f, "    block_{}: ", block_id),
            HIR::FuncLabel(func_id) => write!(f, "func_{}:", func_id),
        }
    }
}
