use super::*;
use super::helpers::*;

pub(super) fn write_type(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::FnPtr(params, ret, _) => {
            let p: Vec<String> = params.iter().map(|p| write_type(p)).collect();
            if matches!(ret.as_ref(), Type::Void(_)) {
                format!("fn({})", p.join(","))
            } else {
                format!("fn({})->{}", p.join(","), write_type(ret))
            }
        },
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Array(inner, _) => format!("[{}]", write_type(inner)),
        Type::Unique(inner, _) => format!("unique {}", write_type(inner)),
        Type::Shared(inner, _) => format!("shared {}", write_type(inner)),
        Type::Weak(inner, _) => format!("weak {}", write_type(inner)),
        Type::Generic(name, args, _) => {
            let args_str: Vec<String> = args.iter().map(|a| write_type(a)).collect();
            format!("{}[{}]", name, args_str.join(", "))
        }
        Type::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", write_type(inner))
            } else {
                format!("ref {}", write_type(inner))
            }
        }
        Type::Self_(_) => "Self".into(),
    }
}

pub(super) fn write_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(lit) => write_literal(lit),
        Expr::Ident(name, _) => name.as_str().to_string(),
        Expr::Binary { op, lhs, rhs, .. } => {
            format!("{} {} {}", write_expr(lhs), write_bin_op(op), write_expr(rhs))
        }
        Expr::Unary { op, arg, .. } => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            format!("{}{}", op_str, write_expr(arg))
        }
        Expr::FnCall { name, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr(a)).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        Expr::Move(inner, _) => format!("move {}", write_expr(inner)),
        Expr::Clone(inner, _) => format!("clone {}", write_expr(inner)),
        Expr::ToUnique(inner, _) => format!("unique {}", write_expr(inner)),
        Expr::ToShared(inner, _) => format!("shared {}", write_expr(inner)),
        Expr::ToWeak(inner, _) => format!("weak {}", write_expr(inner)),
        Expr::MethodCall { object, method, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr(a)).collect();
            format!("{}.{}({})", write_expr(object), method, args_str.join(", "))
        }
        Expr::FieldAccess { object, field, .. } => {
            format!("{}.{}", write_expr(object), field)
        }
        Expr::StructLiteral { type_name, generic_args, fields, .. } => {
            let generic_str = if generic_args.is_empty() {
                String::new()
            } else {
                let args_str: Vec<String> = generic_args.iter().map(|a| write_type(a)).collect();
                format!("[{}]", args_str.join(", "))
            };
            let fields_str: Vec<String> = fields.iter()
                .map(|(n, v)| format!("{} = {}", n, write_expr(v)))
                .collect();
            format!("{}{} {{ {} }}", type_name, generic_str, fields_str.join(", "))
        }
        Expr::ArrayLiteral(elems, _) => {
            let elems_str: Vec<String> = elems.iter().map(|e| write_expr(e)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        Expr::ArraySized { elem_type, count, .. } => {
            format!("[{}; {}]", write_type(elem_type), write_expr(count))
        }
        Expr::Null(_) => "null".into(),
        Expr::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", write_expr(inner))
            } else {
                format!("ref {}", write_expr(inner))
            }
        }
        Expr::Asm { template, outputs, inputs, .. } => {
            let mut parts = Vec::new();
            for (c, e) in outputs {
                parts.push(format!("out({}) {}", c, write_expr(e)));
            }
            for (c, e) in inputs {
                parts.push(format!("in({}) {}", c, write_expr(e)));
            }
            let extra = if parts.is_empty() { String::new() } else { format!(", {}", parts.join(", ")) };
            format!("asm(\"{}\"{})", template, extra)
        }
        Expr::Index { object, index, .. } => {
            format!("{}[{}]", write_expr(object), write_expr(index))
        }
        Expr::CallExpr { target, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr(a)).collect();
            format!("{}({})", write_expr(target), args_str.join(", "))
        }
        Expr::TryOp(inner, _) => {
            format!("{}?", write_expr(inner))
        }
        Expr::Match { .. } => {
            String::new()
        }
        Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, .. } => {
            if !tuple_args.is_empty() {
                format!("{}::{}({})", enum_name, variant_name, tuple_args.iter().map(|e| write_expr(e)).collect::<Vec<_>>().join(", "))
            } else if !named_args.is_empty() {
                format!("{}::{} {{ {} }}", enum_name, variant_name, named_args.iter().map(|(n, v)| format!("{} = {}", n, write_expr(v))).collect::<Vec<_>>().join(", "))
            } else {
                format!("{}::{}", enum_name, variant_name)
            }
        }
        Expr::Lambda { params, return_type, body, .. } => {
            let params_str: Vec<String> = params.iter().map(|(n, t)| format!("{} {}", write_type(t), n)).collect();
            let mut buf = String::new();
            for s in body { write_stmt(&mut buf, s, 1); }
            let body_str = buf;
            format!("({}) -> {} {{\n{}\n}}", params_str.join(", "), write_type(return_type), body_str)
        }
    }
}

pub(super) fn write_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => n.to_string(),
        Literal::Float(n, _) => n.to_string(),
        Literal::Char(c, _) => {
            let s = match c {
                '\n' => "\\n".into(),
                '\t' => "\\t".into(),
                '\'' => "\\'".into(),
                '\\' => "\\\\".into(),
                c if c.is_ascii_graphic() || *c == ' ' => c.to_string(),
                _ => format!("\\x{:02x}", *c as u8),
            };
            format!("'{}'", s)
        }
        Literal::String(s, _) => format!("\"{}\"", s),
        Literal::Bool(b, _) => b.to_string(),
    }
}

pub(super) fn write_bin_op(op: &BinaryOp) -> &str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Neq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}

pub(super) fn vis_str(vis: &Visibility) -> &str {
    match vis {
        Visibility::Pub => "pub ",
        Visibility::PubCrate => "pub(crate) ",
        Visibility::Private => "",
    }
}
