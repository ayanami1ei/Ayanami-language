use super::*;
use super::format::*;
use super::stmt::write_stmt;

pub(super) fn write_expr(expr: &Expr, level: usize, w: &mut impl Write) {
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
        Expr::Lambda { params, return_type, body, .. } => {
            let params_str: Vec<String> = params.iter().map(|(n, t)| format!("{} {:?}", n, t)).collect();
            writeln!(w, "{}Lambda({}) -> {:?}", pad(level), params_str.join(", "), return_type).unwrap();
            for s in body { write_stmt(s, level + 1, w); }
        }
    }
}
