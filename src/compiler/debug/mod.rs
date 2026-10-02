use std::fmt::Write;

use crate::parser::ast::*;

mod expr;
mod format;
mod stmt;

use stmt::write_stmt;


/// Debug-format a program AST as a tree.
pub fn format_program(program: &Program) -> String {
    let mut s = String::new();
    writeln!(s, "Program").unwrap();
    for stmt in &program.stmts {
        write_stmt(stmt, 1, &mut s);
    }
    s
}
