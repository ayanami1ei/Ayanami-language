use super::*;
use super::expr::write_type;

pub(super) fn write_stmt_separator(out: &mut String, stmt: &Stmt) {
    match stmt {
        FnDecl { .. } | StructDef { .. } | EnumDef { .. } | InterfaceDef { .. }
        | ImplBlock { .. } | Namespace { .. } | Import { .. } => {
            out.push_str("\n");
        }
        _ => {}
    }
}

pub(super) fn indent(level: usize) -> String {
    INDENT.repeat(level)
}

pub(super) fn write_block_same_line(out: &mut String, block: &Block, level: usize) {
    if block.stmts.is_empty() {
        let _ = write!(out, "{{}}");
        return;
    }
    let i = indent(level);
    let _ = writeln!(out, "{{");
    for stmt in &block.stmts {
        write_stmt(out, stmt, level + 1);
    }
    let _ = write!(out, "{}}}", i);
}

pub(super) fn write_generic_params(out: &mut String, params: &[(Symbol, Option<Symbol>)]) {
    if params.is_empty() { return; }
    let _ = write!(out, "[");
    for (i, (name, constraint)) in params.iter().enumerate() {
        if i > 0 { let _ = write!(out, ", "); }
        let _ = write!(out, "{}", name);
        if let Some(c) = constraint {
            let _ = write!(out, ": {}", c);
        }
    }
    let _ = write!(out, "]");
}

pub(super) fn write_params(out: &mut String, params: &[(Symbol, Type)]) {
    let _ = write!(out, "(");
    for (i, (name, ty)) in params.iter().enumerate() {
        if i > 0 { let _ = write!(out, ", "); }
        // Format impl method self parameter: shared self / unique self
        if name.as_str() == "self" {
            match ty {
                Type::Unique(inner, _) if matches!(inner.as_ref(), Type::Named(_, _) | Type::Generic(_, _, _)) => {
                    let _ = write!(out, "unique self");
                    continue;
                }
                _ => {}
            }
        }
        let _ = write!(out, "{} {}", write_type(ty), name);
    }
    let _ = write!(out, ")");
}

pub(super) fn write_return_type(out: &mut String, ty: &Type) {
    match ty {
        Type::Void(_) => {}
        _ => {
            let _ = write!(out, " -> {}", write_type(ty));
        }
    }
}
