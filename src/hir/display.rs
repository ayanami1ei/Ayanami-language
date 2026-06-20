use std::fmt::Write;

use crate::hir::*;

pub fn display_hir_program(program: &HirProgram) {
    print!("{}", hir_program_to_string(program));
}

pub fn hir_program_to_string(program: &HirProgram) -> String {
    let mut s = String::new();
    writeln!(s, "HIR Program").unwrap();
    for item in &program.items {
        write_item(item, 1, &mut s).unwrap();
    }
    s
}

pub(crate) fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

pub(crate) fn display_type(ty: &HirType) -> String {
    match ty {
        HirType::Int => "Int".into(),
        HirType::Float => "Float".into(),
        HirType::Char => "Char".into(),
        HirType::Void => "Void".into(),
        HirType::Bool => "Bool".into(),
        HirType::Named(s) => format!("Named({})", s),
        HirType::Unique(inner) => format!("unique {}", display_type(inner)),
        HirType::Shared(inner) => format!("shared {}", display_type(inner)),
        HirType::FnPtr(..) => "fn(...)".to_string(),
        HirType::Weak(inner) => format!("weak {}", display_type(inner)),
        HirType::FatPtr { name, kind } => format!("fatptr({}, {})", name, display_type(kind)),
        HirType::Array(inner) | HirType::ArraySized(inner, _) => format!("[{}]", display_type(inner)),
        HirType::Ref(inner, mutable) => {
            if *mutable {
                format!("ref mut {}", display_type(inner))
            } else {
                format!("ref {}", display_type(inner))
            }
        }
    }
}

fn write_item(item: &HirItem, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    match item {
        HirItem::Fn(f) => {
            let params: Vec<String> = f.params.iter()
                .map(|(n, t)| format!("{}: {}", n, display_type(t)))
                .collect();
            writeln!(w, "{}Fn {} ({}) -> {} [locals: {}]",
                p, f.name, params.join(", "), display_type(&f.return_type), f.locals.len())?;
            for local in &f.locals {
                writeln!(w, "{}  local {}: {}", p, local.name, display_type(&local.ty))?;
            }
            write_block(&f.body, level + 1, w)?;
        }
        HirItem::StructDef(def) => {
            let fields: Vec<String> = def.fields.iter()
                .map(|f| format!("{}: {}", f.name, display_type(&f.ty)))
                .collect();
            writeln!(w, "{}StructDef {} fields=[{}]", p, def.name, fields.join(", "))?;
        }
        HirItem::Namespace { name, items } => {
            writeln!(w, "{}Namespace {}", p, name)?;
            for item in items {
                write_item(item, level + 1, w)?;
            }
        }
        HirItem::InterfaceDef { name, generic_params, methods } => {
            let gp_str: Vec<String> = generic_params.iter().map(|(n, _)| n.to_string()).collect();
            if gp_str.is_empty() {
                writeln!(w, "{}InterfaceDef {}", p, name)?;
            } else {
                writeln!(w, "{}InterfaceDef {}[{}]", p, name, gp_str.join(", "))?;
            }
            for m in methods {
                let params: Vec<String> = m.params.iter()
                    .map(|(n, t)| format!("{}: {}", n, display_type(t)))
                    .collect();
                writeln!(w, "{}  fn {}({}) -> {}",
                    p, m.name, params.join(", "), display_type(&m.return_type))?;
            }
        }
    }
    Ok(())
}

fn write_block(block: &HirBlock, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    writeln!(w, "{}Block {{", p)?;
    for stmt in &block.stmts {
        write_stmt(stmt, level + 1, w)?;
    }
    writeln!(w, "{}}}", p)?;
    Ok(())
}

pub(crate) fn write_expr(expr: &HirNodeBox, level: usize, w: &mut impl Write) -> std::fmt::Result {
    expr.display(level, &mut *w)
}

fn write_stmt(stmt: &HirStmt, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    match stmt {
        HirStmt::Assign { target, value } => {
            writeln!(w, "{}Assign", p)?;
            writeln!(w, "{}  target:", p)?;
            write_expr(target, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        HirStmt::FieldAssign { object, field, field_index, value, .. } => {
            writeln!(w, "{}FieldAssign field={} index={}", p, field, field_index)?;
            writeln!(w, "{}  object:", p)?;
            write_expr(object, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        HirStmt::IndexAssign { object, index, value } => {
            writeln!(w, "{}IndexAssign", p)?;
            writeln!(w, "{}  object:", p)?;
            write_expr(object, level + 1, w)?;
            writeln!(w, "{}  index:", p)?;
            write_expr(index, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        HirStmt::Return { value } => {
            writeln!(w, "{}Return", p)?;
            if let Some(v) = value {
                write_expr(v, level + 1, w)?;
            } else {
                writeln!(w, "{}  (none)", p)?;
            }
        }
        HirStmt::If { cond, then_block, elifs, else_block, .. } => {
            writeln!(w, "{}If", p)?;
            writeln!(w, "{}  cond:", p)?;
            write_expr(cond, level + 1, w)?;
            writeln!(w, "{}  then:", p)?;
            write_block(then_block, level + 1, w)?;
            for (i, (c, b)) in elifs.iter().enumerate() {
                writeln!(w, "{}  elif[{}]:", p, i)?;
                writeln!(w, "{}    cond:", p)?;
                write_expr(c, level + 2, w)?;
                write_block(b, level + 1, w)?;
            }
            if let Some(b) = else_block {
                writeln!(w, "{}  else:", p)?;
                write_block(b, level + 1, w)?;
            }
        }
        HirStmt::While { cond, body } => {
            writeln!(w, "{}While", p)?;
            writeln!(w, "{}  cond:", p)?;
            write_expr(cond, level + 1, w)?;
            writeln!(w, "{}  body:", p)?;
            write_block(body, level + 1, w)?;
        }
        HirStmt::Expr(expr) => {
            writeln!(w, "{}Expr", p)?;
            write_expr(expr, level + 1, w)?;
        }
        HirStmt::Block(stmts) => {
            writeln!(w, "{}Block {{", p)?;
            for s in stmts {
                write_stmt(s, level + 1, w)?;
            }
            writeln!(w, "{}}}", p)?;
        }
        HirStmt::Break => {
            writeln!(w, "{}Break", p)?;
        }
        HirStmt::Continue => {
            writeln!(w, "{}Continue", p)?;
        }
    }
    Ok(())
}
