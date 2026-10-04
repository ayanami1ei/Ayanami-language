use super::*;
use super::helpers::indent;

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
    write_expr_at(expr, 0)
}

pub(super) fn write_expr_at(expr: &Expr, level: usize) -> String {
    match expr {
        Expr::Literal(lit) => write_literal(lit),
        // 解析器把 `A::B` 合并为 `A.B`；格式化时恢复 `::`（`.` 只用于字段/方法）
        Expr::Ident(name, _) => name.as_str().replace('.', "::"),
        Expr::Binary { op, lhs, rhs, .. } => {
            format!("{} {} {}", write_expr_at(lhs, level), write_bin_op(op), write_expr_at(rhs, level))
        }
        Expr::Unary { op, arg, .. } => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            };
            format!("{}{}", op_str, write_expr_at(arg, level))
        }
        Expr::FnCall { name, args, generic_args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            let generic_str = if generic_args.is_empty() {
                String::new()
            } else {
                format!("[{}]", generic_args.iter().map(write_type).collect::<Vec<_>>().join(", "))
            };
            let name_str = name.as_str().replace('.', "::");
            format!("{}{}({})", name_str, generic_str, args_str.join(", "))
        }
        Expr::Move(inner, _) => format!("move {}", write_expr_at(inner, level)),
        Expr::Clone(inner, _) => format!("clone {}", write_expr_at(inner, level)),
        Expr::ToUnique(inner, _) => format!("unique {}", write_expr_at(inner, level)),
        Expr::MethodCall { object, method, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            format!("{}.{}({})", write_expr_at(object, level), method, args_str.join(", "))
        }
        Expr::FieldAccess { object, field, .. } => {
            format!("{}.{}", write_expr_at(object, level), field)
        }
        Expr::StructLiteral { type_name, generic_args, fields, .. } => {
            let generic_str = if generic_args.is_empty() {
                String::new()
            } else {
                let args_str: Vec<String> = generic_args.iter().map(|a| write_type(a)).collect();
                format!("[{}]", args_str.join(", "))
            };
            let fields_str: Vec<String> = fields.iter()
                .map(|(n, v)| format!("{} = {}", n, write_expr_at(v, level)))
                .collect();
            format!("{}{} {{ {} }}", type_name, generic_str, fields_str.join(", "))
        }
        Expr::ArrayLiteral(elems, _) => {
            let elems_str: Vec<String> = elems.iter().map(|e| write_expr_at(e, level)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        Expr::ArraySized { elem_type, count, .. } => {
            format!("[{}; {}]", write_type(elem_type), write_expr_at(count, level))
        }
        Expr::Null(_) => "null".into(),
        Expr::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", write_expr_at(inner, level))
            } else {
                format!("ref {}", write_expr_at(inner, level))
            }
        }
        Expr::Asm { template, outputs, inputs, .. } => {
            let mut parts = Vec::new();
            // 约束还原为源码关键字 `reg`（解析器把 out 存成 "=r"、in 存成 "r"）；
            // 输出在前，与 LLVM 操作数编号（输出 → 输入）一致
            for (c, e) in outputs {
                parts.push(format!("out({}) {}", asm_constraint(c), write_expr_at(e, level)));
            }
            for (c, e) in inputs {
                parts.push(format!("in({}) {}", asm_constraint(c), write_expr_at(e, level)));
            }
            let extra = if parts.is_empty() { String::new() } else { format!(", {}", parts.join(", ")) };
            format!("asm(\"{}\"{})", template, extra)
        }
        Expr::Index { object, index, .. } => {
            format!("{}[{}]", write_expr_at(object, level), write_expr_at(index, level))
        }
        Expr::CallExpr { target, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            format!("{}({})", write_expr_at(target, level), args_str.join(", "))
        }
        Expr::TryOp(inner, _) => {
            format!("{}?", write_expr_at(inner, level))
        }
        Expr::Match { value, arms, .. } => {
            let mut out = format!("match {} {{", write_expr_at(value, level));
            for arm in arms {
                let binds = if arm.bindings.is_empty() {
                    String::new()
                } else {
                    format!("({})", arm.bindings.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))
                };
                out.push_str(&format!("\n{}{}{} => {},", indent(level + 1), arm.variant_name, binds, write_expr_at(&arm.body, level + 1)));
            }
            out.push_str(&format!("\n{}}}", indent(level)));
            out
        }
        Expr::EnumConstruct { enum_name, variant_name, tuple_args, named_args, .. } => {
            if !tuple_args.is_empty() {
                format!("{}::{}({})", enum_name, variant_name, tuple_args.iter().map(|e| write_expr_at(e, level)).collect::<Vec<_>>().join(", "))
            } else if !named_args.is_empty() {
                format!("{}::{} {{ {} }}", enum_name, variant_name, named_args.iter().map(|(n, v)| format!("{} = {}", n, write_expr_at(v, level))).collect::<Vec<_>>().join(", "))
            } else {
                format!("{}::{}", enum_name, variant_name)
            }
        }
        Expr::MacroCall { name, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            format!("#{}({})", name.as_str().replace('.', "::"), args_str.join(", "))
        }
        Expr::Lambda { params, return_type, body, .. } => {
            let params_str: Vec<String> = params.iter().map(|(n, t)| format!("{} {}", write_type(t), n)).collect();
            let mut out = format!("({}) -> {} {{", params_str.join(", "), write_type(return_type));
            for st in body {
                out.push('\n');
                write_stmt(&mut out, st, level + 1);
            }
            while out.ends_with('\n') { out.pop(); }
            out.push_str(&format!("\n{}}}", indent(level)));
            out
        }
    }
}

/// asm 约束 → 源码关键字（当前仅支持 reg）
fn asm_constraint(c: &str) -> &str {
    if c.contains('r') { "reg" } else { c }
}

pub(super) fn write_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => n.to_string(),
        Literal::Float(n, _) => {
            // 保证浮点形态（Rust 的 3.0.to_string() == "3"，会变成 int 字面量）
            let s = n.to_string();
            if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
                s
            } else {
                format!("{}.0", s)
            }
        }
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
        Literal::String(s, _) => {
            let mut out = String::from("\"");
            for c in s.chars() {
                match c {
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    '\r' => out.push_str("\\r"),
                    '\0' => out.push_str("\\0"),
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    c if c.is_ascii_graphic() || c == ' ' => out.push(c),
                    c => out.push_str(&format!("\\x{:02x}", c as u32)),
                }
            }
            out.push('"');
            out
        }
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
