use crate::hir::ir::{HirItem, HirProgram, HirStmt};
use std::path::Path;

pub fn check_hir_returns(hir: &HirProgram, src_path: &Path) -> Result<(), String> {
    for item in &hir.items {
        if let HirItem::Fn(f) = item {
            if f.extern_c {
                continue;
            }
            if matches!(f.return_type, crate::hir::ir::HirType::Void) {
                continue;
            }
            if f.name.as_str() == "main" {
                continue;
            }
            if !has_return_in_item(item) {
                let (ln, col) = if f.span.start_line > 0 || f.span.start_col > 0 {
                    (f.span.start_line, f.span.start_col)
                } else {
                    (1, 1)
                };
                return Err(format!(
                    "{}:{}:{}: error: function `{}` has non-void return type but no return statement",
                    src_path.display(),
                    ln,
                    col,
                    f.name
                ));
            }
        }
    }
    Ok(())
}

fn has_return_in_item(item: &HirItem) -> bool {
    match item {
        HirItem::Fn(f) => f.body.stmts.iter().any(|s| {
            if matches!(s, HirStmt::Return { .. }) {
                return true;
            }
            if let HirStmt::If {
                then_block,
                elifs,
                else_block,
                ..
            } = s
            {
                if then_block.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                    return true;
                }
                for (_, b) in elifs {
                    if b.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                        return true;
                    }
                }
                if let Some(b) = else_block {
                    if b.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                        return true;
                    }
                }
            }
            false
        }),
        HirItem::StructDef(_) | HirItem::InterfaceDef { .. } => true,
        HirItem::Namespace { items, .. } => items.iter().all(|i| has_return_in_item(i)),
        HirItem::Custom(_) => false,
    }
}

fn has_return_in_stmt(s: &HirStmt) -> bool {
    match s {
        HirStmt::Return { .. } => true,
        HirStmt::If {
            then_block,
            elifs,
            else_block,
            ..
        } => {
            if then_block.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                return true;
            }
            for (_, b) in elifs {
                if b.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                    return true;
                }
            }
            if let Some(b) = else_block {
                if b.stmts.iter().any(|s2| has_return_in_stmt(s2)) {
                    return true;
                }
            }
            false
        }
        HirStmt::While { body, .. } => body.stmts.iter().any(|s2| has_return_in_stmt(s2)),
        HirStmt::Block(stmts) => stmts.iter().any(|s2| has_return_in_stmt(s2)),
        _ => false,
    }
}
