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

/// 打印标注：`#[name]` / `#[name(arg, ...)]`
pub(super) fn write_attrs(out: &mut String, attrs: &[Attr], level: usize) {
    for a in attrs {
        let _ = write!(out, "{}#[{}", indent(level), a.path_str());
        if !a.args.is_empty() {
            let rendered: Vec<String> = a.args.iter().map(write_attr_arg).collect();
            let _ = write!(out, "({})", rendered.join(", "));
        }
        let _ = writeln!(out, "]");
    }
}

/// 标注实参 → 源码文本
pub(super) fn write_attr_arg(arg: &crate::parser::ast::AttrArg) -> String {
    use crate::parser::ast::AttrArg;
    match arg {
        AttrArg::KeyValue(k, v) => format!("{} = {}", k, write_attr_arg(v)),
        AttrArg::Expr(e) => super::format_expr(e),
    }
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

pub(super) fn write_params(out: &mut String, params: &[(Symbol, Type)], param_attrs: &[Vec<crate::parser::ast::Attr>]) {
    let _ = write!(out, "(");
    for (i, (name, ty)) in params.iter().enumerate() {
        if i > 0 { let _ = write!(out, ", "); }
        if let Some(attrs) = param_attrs.get(i) {
            for a in attrs {
                let _ = write!(out, "#[{}", a.path_str());
                if !a.args.is_empty() {
                    let rendered: Vec<String> = a.args.iter().map(write_attr_arg).collect();
                    let _ = write!(out, "({})", rendered.join(", "));
                }
                let _ = write!(out, "] ");
            }
        }
        // impl/interface 方法 self 形参：ref self / ref mut self / self（unique self 兼容旧包）
        if name.as_str() == "self" {
            match ty {
                Type::Ref(_, true, _) => { let _ = write!(out, "ref mut self"); continue; }
                Type::Ref(_, false, _) => { let _ = write!(out, "ref self"); continue; }
                Type::Unique(_, _) => { let _ = write!(out, "unique self"); continue; }
                _ => { let _ = write!(out, "self"); continue; }
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
