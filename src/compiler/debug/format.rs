use super::*;

pub(super) fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

pub(super) fn format_type(ty: &Type) -> String {
    match ty {
        Type::Default | Type::FnPtr(..) => "???".into(),
        Type::Closure(..) => "Fn(...)".into(),
        Type::Int(_) => "Int".into(),
        Type::Float(_) => "Float".into(),
        Type::Char(_) => "Char".into(),
        Type::Bool(_) => "Bool".into(),
        Type::Void(_) => "Void".into(),
        Type::Never(_) => "Never".into(),
        Type::Named(s, _) => format!("Named({})", s),
        Type::Array(inner, _) => format!("[{}]", format_type(inner)),
        Type::ArraySized(inner, n, _) => format!("[{}; {}]", format_type(inner), n),
        Type::Unique(inner, _) => format!("unique {}", format_type(inner)),
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

pub(super) fn format_op(op: &BinaryOp) -> &str {
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
        BinaryOp::BitAnd => "BitAnd",
        BinaryOp::BitOr => "BitOr",
        BinaryOp::BitXor => "BitXor",
        BinaryOp::Shl => "Shl",
        BinaryOp::Shr => "Shr",
    }
}

pub(super) fn format_unary(op: &UnaryOp) -> &str {
    match op {
        UnaryOp::Neg => "Neg",
        UnaryOp::Not => "Not",
        UnaryOp::BitNot => "BitNot",
    }
}

pub(super) fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => format!("Int({})", n),
        Literal::Float(n, _) => format!("Float({})", n),
        Literal::Char(c, _) => format!("Char('{}')", c),
        Literal::String(s, _) => format!("String(\"{}\")", s),
        Literal::Bool(b, _) => format!("Bool({})", b),
    }
}
