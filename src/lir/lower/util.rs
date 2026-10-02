use super::*;

pub(super) fn write_stmt_block(stmts: &[MirStmtBox], level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    writeln!(w, "{:width$}Block {{", "", width = level * 2)?;
    for s in stmts { s.display_stmt(level + 1, w)?; }
    writeln!(w, "{:width$}}}", "", width = level * 2)?;
    Ok(())
}

pub(super) fn display_hir_type(ty: &HirType) -> String {
    crate::hir::display::display_type(ty)
}

pub(super) fn extract_var(val: &LirValue) -> VarId {
    match val {
        LirValue::Var(v) => *v,
        _ => panic!("expected Var, got {:?}", val),
    }
}
