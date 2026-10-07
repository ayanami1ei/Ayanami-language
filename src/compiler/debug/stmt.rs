use super::*;
use super::expr::write_expr;
use super::format::*;

fn write_block(block: &Block, level: usize, w: &mut impl Write) {
    writeln!(w, "{}Block {{", pad(level)).unwrap();
    for stmt in &block.stmts {
        write_stmt(stmt, level + 1, w);
    }
    writeln!(w, "{}}}", pad(level)).unwrap();
}

pub(super) fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write) {
    let p = pad(level);
    match stmt {
        Stmt::Block(b) => {
            writeln!(w, "{}Block ({} stmts)", p, b.stmts.len()).unwrap();
        }
        Stmt::Unsafe { block, .. } => {
            writeln!(w, "{}Unsafe ({} stmts)", p, block.stmts.len()).unwrap();
        }
        Stmt::FnDecl {
            name,
            params,
            return_type,
            body,
            ..
        } => {
            writeln!(
                w,
                "{}FnDecl {{ name: {}, return: {} }}",
                p,
                name,
                format_type(return_type)
            )
            .unwrap();
            for (n, t) in params {
                writeln!(w, "{}    {}: {}", p, n, format_type(t)).unwrap();
            }
            writeln!(w, "{}  body:", p).unwrap();
            write_block(body, level + 1, w);
        }
        Stmt::Assign { name, value, .. } => {
            writeln!(w, "{}Assign {{ name: {} }}", p, name).unwrap();
            writeln!(w, "{}  value:", p).unwrap();
            write_expr(value, level + 1, w);
        }
        Stmt::FieldAssign {
            object, field, value, ..
        } => {
            writeln!(w, "{}FieldAssign {{ field: {} }}", p, field).unwrap();
            writeln!(w, "{}  object:", p).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  value:", p).unwrap();
            write_expr(value, level + 1, w);
        }
        Stmt::IndexAssign {
            object, index, value, ..
        } => {
            writeln!(w, "{}IndexAssign", p).unwrap();
            writeln!(w, "{}  object:", p).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  index:", p).unwrap();
            write_expr(index, level + 1, w);
            writeln!(w, "{}  value:", p).unwrap();
            write_expr(value, level + 1, w);
        }
        Stmt::Return { value, .. } => {
            writeln!(w, "{}Return", p).unwrap();
            if let Some(val) = value {
                write_expr(val, level + 1, w);
            }
        }
        Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            writeln!(w, "{}If", p).unwrap();
            writeln!(w, "{}  cond:", p).unwrap();
            write_expr(cond, level + 1, w);
            writeln!(w, "{}  then:", p).unwrap();
            write_block(then_block, level + 1, w);
            for (c, b) in elifs {
                writeln!(w, "{}  elif:", p).unwrap();
                write_expr(c, level + 1, w);
                write_block(b, level + 1, w);
            }
            if let Some(b) = else_block {
                writeln!(w, "{}  else:", p).unwrap();
                write_block(b, level + 1, w);
            }
        }
        Stmt::For {
            iterator,
            start,
            end,
            step,
            body,
            ..
        } => {
            writeln!(w, "{}For {{ iterator: {} }}", p, iterator).unwrap();
            write_expr(start, level + 1, w);
            write_expr(end, level + 1, w);
            if let Some(s) = step {
                write_expr(s, level + 1, w);
            }
            write_block(body, level + 1, w);
        }
        Stmt::While { cond, body, .. } => {
            writeln!(w, "{}While", p).unwrap();
            write_expr(cond, level + 1, w);
            write_block(body, level + 1, w);
        }
        Stmt::ForIn { iterator, iterable, body, .. } => {
            writeln!(w, "{}ForIn {{ iterator: {} }}", p, iterator).unwrap();
            write_expr(iterable, level + 1, w);
            write_block(body, level + 1, w);
        }
        Stmt::Match { .. } => {
            writeln!(w, "{}Match", p).unwrap();
        }
        Stmt::Namespace { name, items, .. } => {
            writeln!(w, "{}Namespace {{ name: {} }}", p, name).unwrap();
            for item in items {
                write_stmt(item, level + 1, w);
            }
        }
        Stmt::ExprStmt { expr, .. } => {
            writeln!(w, "{}ExprStmt", p).unwrap();
            write_expr(expr, level + 1, w);
        }
        Stmt::StructDef { name, fields, .. } => {
            writeln!(w, "{}StructDef {{ name: {} }}", p, name).unwrap();
            for (fname, fty) in fields {
                writeln!(w, "{}  {}: {}", p, fname, format_type(fty)).unwrap();
            }
        }
        Stmt::InterfaceDef { name, methods, .. } => {
            writeln!(w, "{}InterfaceDef {{ name: {} }}", p, name).unwrap();
            for m in methods {
                let params: Vec<String> = m
                    .params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, format_type(t)))
                    .collect();
                writeln!(
                    w,
                    "{}  fn {}({}) -> {}",
                    p,
                    m.name,
                    params.join(", "),
                    format_type(&m.return_type)
                )
                .unwrap();
            }
        }
            Stmt::EnumDef { .. } => {}
        Stmt::ImplBlock {
            type_name, methods, ..
        } => {
            writeln!(w, "{}ImplBlock {{ name: {} }}", p, type_name).unwrap();
            for m in methods {
                write_stmt(m, level + 1, w);
            }
        }
        Stmt::Attributed { attrs, stmt, .. } => {
            writeln!(w, "{}Attributed ({} attrs)", p, attrs.len()).unwrap();
            write_stmt(stmt, level + 1, w);
        }
        Stmt::Import { path, .. } => {
            writeln!(w, "{}Import {{ path: {} }}", p, path).unwrap();
        }
        Stmt::ConstDecl { name, ty, value, .. } => {
            writeln!(w, "{}ConstDecl {{ name: {}, ty: {:?}, value: {:?} }}", p, name, ty, value).unwrap();
        }
        Stmt::StaticDecl { name, is_mut, ty, value, .. } => {
            writeln!(w, "{}StaticDecl {{ name: {}, mut: {}, ty: {:?}, value: {:?} }}", p, name, is_mut, ty, value).unwrap();
        }
        Stmt::Break { .. } => {
            writeln!(w, "{}Break", p).unwrap();
        }
        Stmt::Continue { .. } => {
            writeln!(w, "{}Continue", p).unwrap();
        }
    }
}
