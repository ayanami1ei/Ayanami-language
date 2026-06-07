use std::fmt::Write;

use super::ir::*;
use crate::hir::ir::{HirLiteral, HirType, VarId};

pub fn display_mir_program(program: &MirProgram) {
    print!("{}", mir_program_to_string(program));
}

pub fn mir_program_to_string(program: &MirProgram) -> String {
    let mut s = String::new();
    writeln!(s, "MIR Program").unwrap();
    for item in &program.items {
        write_item(item, 1, &mut s).unwrap();
    }
    s
}

fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

fn display_type(ty: &HirType) -> String {
    match ty {
        HirType::Int => "Int".into(),
        HirType::Float => "Float".into(),
        HirType::Char => "Char".into(),
        HirType::Void => "Void".into(),
        HirType::Bool => "Bool".into(),
        HirType::Named(s) => format!("Named({})", s),
        HirType::Unique(inner) => format!("unique {}", display_type(inner)),
        HirType::Shared(inner) => format!("shared {}", display_type(inner)),
        HirType::Weak(inner) => format!("weak {}", display_type(inner)),
        HirType::FatPtr { name, kind } => format!("fatptr({}, {})", name, display_type(kind)),
        HirType::Array(inner) => format!("[{}]", display_type(inner)),
    }
}

fn write_item(item: &MirItem, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    match item {
        MirItem::Fn(f) => {
            let params: Vec<String> = f
                .params
                .iter()
                .map(|(n, t)| format!("{}: {}", n, display_type(t)))
                .collect();
            writeln!(
                w,
                "{}Fn {} ({}) -> {} [locals: {}]",
                p,
                f.name,
                params.join(", "),
                display_type(&f.return_type),
                f.locals.len()
            )?;
            for local in &f.locals {
                writeln!(w, "{}  local {}: {}", p, local.name, display_type(&local.ty))?;
            }
            write_block(&f.body, level + 1, w)?;
        }
        MirItem::StructDef { name, fields } => {
            writeln!(w, "{}StructDef {} fields=[{}]", p, name, fields.iter().map(|(n, t)| format!("{}:{}", n, display_type(t))).collect::<Vec<_>>().join(", "))?;
        }
        MirItem::Namespace { name, items } => {
            writeln!(w, "{}Namespace {}", p, name)?;
            for item in items {
                write_item(item, level + 1, w)?;
            }
        }
    }
    Ok(())
}

fn write_block(stmts: &[MirStmt], level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    writeln!(w, "{}Block {{", p)?;
    for stmt in stmts {
        write_stmt(stmt, level + 1, w)?;
    }
    writeln!(w, "{}}}", p)?;
    Ok(())
}

fn write_expr(expr: &MirExpr, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    match expr {
        MirExpr::Literal(lit, _) => {
            let s = match lit {
                HirLiteral::Int(n) => format!("Int({})", n),
                HirLiteral::Float(n) => format!("Float({})", n),
                HirLiteral::Char(c) => format!("Char('{}')", c),
                HirLiteral::String(s) => format!("String(\"{}\")", s),
                HirLiteral::Bool(b) => format!("Bool({})", b),
            };
            writeln!(w, "{}Literal({})", p, s)?;
        }
        MirExpr::Local(VarId(id), ty, moved) => {
            let m = if *moved { " [moved]" } else { "" };
            writeln!(w, "{}Local(v{} : {}{})", p, id, display_type(ty), m)?;
        }
        MirExpr::Binary { op, lhs, rhs, ty } => {
            writeln!(w, "{}Binary {{ op: {:?}, ty: {} }}", p, op, display_type(ty))?;
            writeln!(w, "{}  lhs:", p)?;
            write_expr(lhs, level + 1, w)?;
            writeln!(w, "{}  rhs:", p)?;
            write_expr(rhs, level + 1, w)?;
        }
        MirExpr::Unary { op, arg, ty } => {
            writeln!(w, "{}Unary {{ op: {:?}, ty: {} }}", p, op, display_type(ty))?;
            write_expr(arg, level + 1, w)?;
        }
        MirExpr::Call { fn_id, args, ty } => {
            writeln!(w, "{}Call(fn{}, ty: {})", p, fn_id.0, display_type(ty))?;
            for arg in args {
                write_expr(arg, level + 1, w)?;
            }
        }
        MirExpr::Move(inner, ty) => {
            writeln!(w, "{}Move(ty: {})", p, display_type(ty))?;
            write_expr(inner, level + 1, w)?;
        }
        MirExpr::Clone(inner, ty) => {
            writeln!(w, "{}Clone(ty: {})", p, display_type(ty))?;
            write_expr(inner, level + 1, w)?;
        }
        MirExpr::ToUnique(inner, ty) => {
            writeln!(w, "{}ToUnique(ty: {})", p, display_type(ty))?;
            write_expr(inner, level + 1, w)?;
        }
        MirExpr::ToShared(inner, ty) => {
            writeln!(w, "{}ToShared(ty: {})", p, display_type(ty))?;
            write_expr(inner, level + 1, w)?;
        }
        MirExpr::ToWeak(inner, ty) => {
            writeln!(w, "{}ToWeak(ty: {})", p, display_type(ty))?;
            write_expr(inner, level + 1, w)?;
        }
        MirExpr::VirtualCall { receiver, interface, method_index, args, ty } => {
            writeln!(w, "{}VirtualCall iface={} method={} ty={}", p, interface, method_index, display_type(ty))?;
            writeln!(w, "{}  receiver:", p)?;
            write_expr(receiver, level + 1, w)?;
            for arg in args {
                write_expr(arg, level + 1, w)?;
            }
        }
        MirExpr::MakeFatPtr { value, concrete_type, interface_name, ty } => {
            writeln!(w, "{}MakeFatPtr {} -> {} ty={}", p, concrete_type, interface_name, display_type(ty))?;
            write_expr(value, level + 1, w)?;
        }
        MirExpr::FieldAccess { object, field, ty, .. } => {
            writeln!(w, "{}FieldAccess {} ty={}", p, field, display_type(ty))?;
            write_expr(object, level + 1, w)?;
        }
        MirExpr::StructLiteral { type_name, fields, ty } => {
            writeln!(w, "{}StructLiteral {} fields={} ty={}", p, type_name, fields.len(), display_type(ty))?;
            for (name, e) in fields {
                writeln!(w, "{}  {}:", p, name)?;
                write_expr(e, level + 1, w)?;
            }
        }
        MirExpr::ArrayLiteral(elems, ty) => {
            writeln!(w, "{}ArrayLiteral len={} ty={}", p, elems.len(), display_type(ty))?;
            for e in elems {
                write_expr(e, level + 1, w)?;
            }
        }
        MirExpr::ArraySized { count, elem_ty, ty } => {
            writeln!(w, "{}ArraySized {{ count: {}, elem_ty: {}, ty: {} }}", p, count, display_type(elem_ty), display_type(ty))?;
        }
        MirExpr::Index { object, index, ty } => {
            writeln!(w, "{}Index ty={}", p, display_type(ty))?;
            writeln!(w, "{}  object:", p)?;
            write_expr(object, level + 1, w)?;
            writeln!(w, "{}  index:", p)?;
            write_expr(index, level + 1, w)?;
        }
    }
    Ok(())
}

fn write_stmt(stmt: &MirStmt, level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    match stmt {
        MirStmt::Assign { target, value } => {
            writeln!(w, "{}Assign", p)?;
            writeln!(w, "{}  target:", p)?;
            write_expr(target, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        MirStmt::FieldAssign { object, field, field_index, field_ty, value } => {
            writeln!(w, "{}FieldAssign field={} index={} ty={}", p, field, field_index, display_type(field_ty))?;
            writeln!(w, "{}  object:", p)?;
            write_expr(object, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        MirStmt::IndexAssign { object, index, value } => {
            writeln!(w, "{}IndexAssign", p)?;
            writeln!(w, "{}  object:", p)?;
            write_expr(object, level + 1, w)?;
            writeln!(w, "{}  index:", p)?;
            write_expr(index, level + 1, w)?;
            writeln!(w, "{}  value:", p)?;
            write_expr(value, level + 1, w)?;
        }
        MirStmt::Return { value } => {
            writeln!(w, "{}Return", p)?;
            if let Some(v) = value {
                write_expr(v, level + 1, w)?;
            } else {
                writeln!(w, "{}  (none)", p)?;
            }
        }
        MirStmt::If {
            cond,
            then_block,
            elifs,
            else_block,
        } => {
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
        MirStmt::While { cond, body } => {
            writeln!(w, "{}While", p)?;
            writeln!(w, "{}  cond:", p)?;
            write_expr(cond, level + 1, w)?;
            writeln!(w, "{}  body:", p)?;
            write_block(body, level + 1, w)?;
        }
        MirStmt::Expr(expr) => {
            writeln!(w, "{}Expr", p)?;
            write_expr(expr, level + 1, w)?;
        }
        MirStmt::Block(stmts) => {
            writeln!(w, "{}Block {{", p)?;
            for s in stmts {
                write_stmt(s, level + 1, w)?;
            }
            writeln!(w, "{}}}", p)?;
        }
        MirStmt::Drop(VarId(id), ty) => {
            writeln!(w, "{}Drop(v{} : {})", p, id, display_type(ty))?;
        }
        MirStmt::Retain(VarId(id), ty) => {
            writeln!(w, "{}Retain(v{} : {})", p, id, display_type(ty))?;
        }
        MirStmt::Release(VarId(id), ty) => {
            writeln!(w, "{}Release(v{} : {})", p, id, display_type(ty))?;
        }
    }
    Ok(())
}
