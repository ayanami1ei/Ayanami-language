#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    /// M1.2：按位取反 `~`
    BitNot,
    /// M5 P2：裸指针解引用 `*p`（读写均需 unsafe）
    Deref,
}
