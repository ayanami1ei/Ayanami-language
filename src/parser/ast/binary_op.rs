#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    /// M1.2：位运算与移位
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}
