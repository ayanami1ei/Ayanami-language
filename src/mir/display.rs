use std::fmt::Write;

use super::ir::*;
use crate::hir::ir::{HirLiteral, HirType, VarId};
use crate::hir::display::display_type;

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
            write_stmts(&f.body, level + 1, w)?;
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

fn write_stmts(stmts: &[MirStmtBox], level: usize, w: &mut impl Write) -> std::fmt::Result {
    let p = pad(level);
    writeln!(w, "{}Block {{", p)?;
    for stmt in stmts {
        stmt.display_stmt(level + 1, w)?;
    }
    writeln!(w, "{}}}", p)?;
    Ok(())
}
