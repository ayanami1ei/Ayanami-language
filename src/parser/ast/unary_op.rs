#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    /// M1.2：按位取反 `~`
    BitNot,
}
