use std::fmt;

use crate::hir::{BinOperator, BlockId, FuncId, HIR, HIRInst, ObjId, UnaryOperation, VarId};

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BlockId(id) = self;
        write!(f, "{}", id)
    }
}
impl fmt::Display for ObjId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ObjId(id) = self;
        write!(f, "{}", id)
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
        match self {
            HIRInst::New { obj_type, dst } => write!(f, "New {:?} -> obj_{}", obj_type, dst),
            HIRInst::Br {
                cond,
                then_block,
                else_block,
            } => write!(
                f,
                "Br cond: {}, then-block: {}, else-block: {}",
                cond, then_block, else_block
            ),
            HIRInst::Jmp { target } => write!(f, "Jmp to block_{}", target),
            HIRInst::Call { id, ret } => write!(f, "Call func_{}, ret: {:?}", id, ret),
            HIRInst::BinOp {
                left,
                op,
                right,
                dst,
            } => write!(f, "obj_{} {} obj_{} -> obj_{}", left, op, right, dst),
            HIRInst::UnaryOp { op, expr, dst } => write!(f,"{} obj_{} -> obj_{}", op,expr,dst),
            HIRInst::IncRef { obj } => write!(f,"Increase reference obj_{}",obj),
            HIRInst::DecRef { obj } => write!(f,"Decrease reference obj_{}",obj),
            HIRInst::Bind { var, obj } => write!(f,"Bind var_{} and obj_{}",var,obj),
            HIRInst::Load { var, obj } => write!(f,"Load var_{}'s obj to obj_{}",var,obj),
            HIRInst::Ret { ret_obj } => write!(f,"Ret obj_{}", ret_obj),
        }
    }
}

impl fmt::Display for HIR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HIR::Inst(hirinst) => write!(f,"{}",hirinst),
            HIR::Block(block_id) => write!(f, "block_{}: ", block_id),
        }
    }
}
