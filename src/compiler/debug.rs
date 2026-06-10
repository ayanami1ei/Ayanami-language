use std::fmt::Write;

use crate::parser::ast::*;

fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::Int(_) => "Int".into(),
        Type::Float(_) => "Float".into(),
        Type::Char(_) => "Char".into(),
        Type::Bool(_) => "Bool".into(),
        Type::Void(_) => "Void".into(),
        Type::Named(s, _) => format!("Named({})", s),
        Type::Array(inner, _) => format!("[{}]", format_type(inner)),
        Type::Unique(inner, _) => format!("unique {}", format_type(inner)),
        Type::Shared(inner, _) => format!("shared {}", format_type(inner)),
        Type::Weak(inner, _) => format!("weak {}", format_type(inner)),
        Type::Generic(name, args, _) => format!(
            "{}[{}]",
            name,
            args.iter()
                .map(|a| format_type(a))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Type::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", format_type(inner))
            } else {
                format!("ref {}", format_type(inner))
            }
        }
        Type::Self_(_) => "Self".into(),
    }
}

fn format_op(op: &BinaryOp) -> &str {
    match op {
        BinaryOp::Add => "Add",
        BinaryOp::Sub => "Sub",
        BinaryOp::Mul => "Mul",
        BinaryOp::Div => "Div",
        BinaryOp::Mod => "Mod",
        BinaryOp::Eq => "Eq",
        BinaryOp::Neq => "Neq",
        BinaryOp::Lt => "Lt",
        BinaryOp::Gt => "Gt",
        BinaryOp::Le => "Le",
        BinaryOp::Ge => "Ge",
        BinaryOp::And => "And",
        BinaryOp::Or => "Or",
    }
}

fn format_unary(op: &UnaryOp) -> &str {
    match op {
        UnaryOp::Neg => "Neg",
        UnaryOp::Not => "Not",
    }
}

fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => format!("Int({})", n),
        Literal::Float(n, _) => format!("Float({})", n),
        Literal::Char(c, _) => format!("Char('{}')", c),
        Literal::String(s, _) => format!("String(\"{}\")", s),
        Literal::Bool(b, _) => format!("Bool({})", b),
    }
}

/// Debug-format a program AST as a tree.
pub fn format_program(program: &Program) -> String {
    let mut s = String::new();
    writeln!(s, "Program").unwrap();
    for stmt in &program.stmts {
        write_stmt(stmt, 1, &mut s);
    }
    s
}

fn write_expr(expr: &Expr, level: usize, w: &mut impl Write) {
    match expr {
        Expr::Literal(lit) => {
            writeln!(w, "{}{}", pad(level), format_literal(lit)).unwrap();
        }
        Expr::Ident(name, _) => {
            writeln!(w, "{}Ident({})", pad(level), name).unwrap();
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            writeln!(w, "{}Binary {{ op: {} }}", pad(level), format_op(op)).unwrap();
            writeln!(w, "{}  lhs:", pad(level)).unwrap();
            write_expr(lhs, level + 1, w);
            writeln!(w, "{}  rhs:", pad(level)).unwrap();
            write_expr(rhs, level + 1, w);
        }
        Expr::Unary { op, arg, .. } => {
            writeln!(w, "{}Unary {{ op: {} }}", pad(level), format_unary(op)).unwrap();
            write_expr(arg, level + 1, w);
        }
        Expr::FnCall { name, args, .. } => {
            writeln!(w, "{}FnCall {{ name: {} }}", pad(level), name).unwrap();
            writeln!(w, "{}  args:", pad(level)).unwrap();
            for arg in args {
                write_expr(arg, level + 1, w);
            }
        }
        Expr::Move(expr, _) => {
            writeln!(w, "{}Move", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::Clone(expr, _) => {
            writeln!(w, "{}Clone", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToUnique(expr, _) => {
            writeln!(w, "{}ToUnique", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToShared(expr, _) => {
            writeln!(w, "{}ToShared", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToWeak(expr, _) => {
            writeln!(w, "{}ToWeak", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::MethodCall {
            object, method, args, ..
        } => {
            writeln!(w, "{}MethodCall {{ method: {} }}", pad(level), method).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            for arg in args {
                write_expr(arg, level + 1, w);
            }
        }
        Expr::FieldAccess { object, field, .. } => {
            writeln!(w, "{}FieldAccess {{ field: {} }}", pad(level), field).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
        }
        Expr::StructLiteral {
            type_name, fields, ..
        } => {
            writeln!(
                w,
                "{}StructLiteral {{ type: {} }}",
                pad(level),
                type_name
            )
            .unwrap();
            for (name, val) in fields {
                writeln!(w, "{}  {} =", pad(level), name).unwrap();
                write_expr(val, level + 1, w);
            }
        }
        Expr::ArrayLiteral(elems, _) => {
            writeln!(w, "{}ArrayLiteral", pad(level)).unwrap();
            for e in elems {
                write_expr(e, level + 1, w);
            }
        }
        Expr::ArraySized {
            elem_type, count, ..
        } => {
            writeln!(
                w,
                "{}ArraySized {{ elem_type: {} }}",
                pad(level),
                format_type(elem_type)
            )
            .unwrap();
            writeln!(w, "{}  count:", pad(level)).unwrap();
            write_expr(count, level + 1, w);
        }
        Expr::Null(_) => writeln!(w, "{}Null", pad(level)).unwrap(),
        Expr::Ref(expr, mutable, _) => {
            let m = if *mutable { "mut " } else { "" };
            writeln!(w, "{}Ref({})", pad(level), m).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::Index { object, index, .. } => {
            writeln!(w, "{}Index", pad(level)).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  index:", pad(level)).unwrap();
            write_expr(index, level + 1, w);
        }
        Expr::Asm { template, .. } => {
            writeln!(w, "{}Asm(\"{}\")", pad(level), template).unwrap();
        }
        Expr::CallExpr { target, args, .. } => {
            writeln!(w, "{}CallExpr", pad(level)).unwrap();
            writeln!(w, "{}  target:", pad(level)).unwrap();
            write_expr(target, level + 1, w);
            for arg in args {
                write_expr(arg, level + 1, w);
            }
        }
        Expr::TryOp(inner, _) => {
            writeln!(w, "{}TryOp", pad(level)).unwrap();
            write_expr(inner, level + 1, w);
        }
        Expr::Match { .. } => {
            writeln!(w, "{}Match", pad(level)).unwrap();
        }
        Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, .. } => {
            writeln!(w, "{}EnumConstruct {}.{}", pad(level), enum_name, variant_name).unwrap();
            for e in tuple_args { write_expr(e, level + 1, w); }
            for (_, e) in named_args { write_expr(e, level + 1, w); }
        }
    }
}

fn write_block(block: &Block, level: usize, w: &mut impl Write) {
    writeln!(w, "{}Block {{", pad(level)).unwrap();
    for stmt in &block.stmts {
        write_stmt(stmt, level + 1, w);
    }
    writeln!(w, "{}}}", pad(level)).unwrap();
}

fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write) {
    let p = pad(level);
    match stmt {
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
        Stmt::Import { path, .. } => {
            writeln!(w, "{}Import {{ path: {} }}", p, path).unwrap();
        }
        Stmt::Break { .. } => {
            writeln!(w, "{}Break", p).unwrap();
        }
        Stmt::Continue { .. } => {
            writeln!(w, "{}Continue", p).unwrap();
        }
    }
}
