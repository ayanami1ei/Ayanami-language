use super::*;

pub(super) fn write_stmt_block(stmts: &[MirStmtBox], level: usize, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
    writeln!(w, "{:width$}Block {{", "", width = level * 2)?;
    for s in stmts { s.display_stmt(level + 1, w)?; }
    writeln!(w, "{:width$}}}", "", width = level * 2)?;
    Ok(())
}

/// AST 标注实参 → LIR 字符串形式（A2a）
pub(super) fn attr_arg_to_string(arg: &crate::parser::ast::AttrArg) -> String {
    use crate::parser::ast::AttrArg;
    match arg {
        AttrArg::KeyValue(k, v) => format!("{} = {}", k, attr_arg_to_string(v)),
        AttrArg::Expr(e) => crate::formatter::format_expr(e),
    }
}

/// A3b：声明效应 → LIR 布尔摘要（仅显式空集参与属性）。
pub(super) fn lir_effects(d: &crate::hir::effects::EffectDecl) -> LirEffects {
    LirEffects { no_throws: d.no_throws(), no_effects: d.no_effects() }
}

/// AST 标注列表 → LIR 标注列表
pub(super) fn attrs_to_lir(attrs: &[crate::parser::ast::Attr]) -> Vec<LirAttr> {
    attrs.iter().map(|a| LirAttr {
        // 内置（含 core:: 别名）只保留末段名，库宏保留全路径（A5a）
        name: if a.is_builtin() { a.name.as_str() } else { a.path_str() },
        args: a.args.iter().map(|x| attr_arg_to_string(x)).collect(),
    }).collect()
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
