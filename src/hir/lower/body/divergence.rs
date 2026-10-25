//! `-> !` 函数体发散性判断（M1.9）：不再只看最后一条语句。
use super::*;

/// 块是否必然发散：任一条语句自身发散即可（其后语句不可达）。
pub(super) fn block_diverges(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(stmt_diverges)
}

fn stmt_diverges(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Return { .. } => true,
        HirStmt::Expr { expr, .. } => {
            matches!(strip_ownership(expr_type(expr)), HirType::Never)
        }
        HirStmt::Block { stmts, .. } => block_diverges(stmts),
        // 所有分支（必须含 else）都发散
        HirStmt::If { then_block, elifs, else_block, .. } => {
            else_block.as_ref().map_or(false, |e| block_diverges(&e.stmts))
                && block_diverges(&then_block.stmts)
                && elifs.iter().all(|(_, b)| block_diverges(&b.stmts))
        }
        // `while true` 且没有退出本层的 break（嵌套循环的 break 归内层）
        HirStmt::While { cond, body, .. } => {
            matches!(cond.as_const(), Some(HirLiteral::Bool(true)))
                && !has_break(&body.stmts)
        }
        _ => false,
    }
}

fn has_break(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(|s| match s {
        HirStmt::Break { .. } => true,
        HirStmt::Block { stmts, .. } => has_break(stmts),
        HirStmt::If { then_block, elifs, else_block, .. } => {
            has_break(&then_block.stmts)
                || elifs.iter().any(|(_, b)| has_break(&b.stmts))
                || else_block.as_ref().map_or(false, |b| has_break(&b.stmts))
        }
        _ => false,
    })
}
