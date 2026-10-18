use super::*;
use super::helpers::{indent, write_match_body};
use super::types::write_type;

pub(super) fn write_expr(expr: &Expr) -> String {
    write_expr_at(expr, 0)
}


/// 二元运算符优先级（数值越大结合越紧；与解析器/文法一致）
fn binop_prec(op: &BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 3,
        BinaryOp::And => 4,
        BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => 5,
        BinaryOp::BitOr => 6,
        BinaryOp::BitXor => 7,
        BinaryOp::BitAnd => 8,
        BinaryOp::Shl | BinaryOp::Shr => 9,
        BinaryOp::Add | BinaryOp::Sub => 10,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 11,
    }
}

/// 表达式优先级（原子/后缀 14；一元 13；`as` 12；二元见 binop_prec）
fn expr_prec(e: &Expr) -> u8 {
    match e {
        Expr::Binary { op, .. } => binop_prec(op),
        Expr::Cast { .. } => 12,
        Expr::Unary { .. } | Expr::Move(..) | Expr::Clone(..) | Expr::ToUnique(..) | Expr::Ref(..) => 13,
        _ => 14,
    }
}

fn maybe_paren(cond: bool, e: &Expr, level: usize) -> String {
    let s = write_expr_at(e, level);
    if cond { format!("({})", s) } else { s }
}

pub(super) fn write_expr_at(expr: &Expr, level: usize) -> String {
    match expr {
        Expr::Literal(lit) => write_literal(lit),
        Expr::Suffixed { lit, suffix, .. } => format!("{}{}", write_literal(lit), suffix.as_str()),
        // 解析器把 `A::B` 合并为 `A.B`；格式化时恢复 `::`（`.` 只用于字段/方法）
        Expr::Ident(name, _) => name.as_str().replace('.', "::"),
        Expr::Binary { op, lhs, rhs, .. } => {
            // #89：按优先级补括号，保证 .lcl 泛型源码导出后重解析语义不变
            let p = binop_prec(op);
            let l = maybe_paren(expr_prec(lhs) < p, lhs, level);
            let r = maybe_paren(expr_prec(rhs) <= p, rhs, level);
            format!("{} {} {}", l, write_bin_op(op), r)
        }
        Expr::If { cond, then_block, elifs, else_block, .. } => {
            let mut out = format!("if {} ", write_expr_at(cond, level));
            super::helpers::write_block_same_line(&mut out, then_block, level);
            for (c, b) in elifs {
                out.push_str(&format!(" elif {} ", write_expr_at(c, level)));
                super::helpers::write_block_same_line(&mut out, b, level);
            }
            if let Some(b) = else_block {
                out.push_str(" else ");
                super::helpers::write_block_same_line(&mut out, b, level);
            }
            out
        }
        Expr::Cast { expr, ty, .. } => {
            let e = maybe_paren(expr_prec(expr) < 12, expr, level);
            format!("{} as {}", e, write_type(ty))
        }
        Expr::Unary { op, arg, .. } => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
                UnaryOp::BitNot => "~",
                UnaryOp::Deref => "*",
            };
            let arg = maybe_paren(expr_prec(arg) < 13, arg, level);
            format!("{}{}", op_str, arg)
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
        Expr::Move(inner, _) => format!("move {}", maybe_paren(expr_prec(inner) < 13, inner, level)),
        Expr::Clone(inner, _) => format!("clone {}", maybe_paren(expr_prec(inner) < 13, inner, level)),
        Expr::ToUnique(inner, _) => format!("unique {}", maybe_paren(expr_prec(inner) < 13, inner, level)),
        Expr::MethodCall { object, method, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            // #190：接收者是低优先级表达式（如 `(a - b).to_string()`）时必须保留括号
            let o = maybe_paren(expr_prec(object) < 14, object, level);
            format!("{}.{}({})", o, method, args_str.join(", "))
        }
        Expr::FieldAccess { object, field, .. } => {
            // #190：同上（`(a - b).x`）
            let o = maybe_paren(expr_prec(object) < 14, object, level);
            format!("{}.{}", o, field)
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
        Expr::ArrayRepeat { value, count, .. } => format!("[{}; {}]", write_expr_at(value, level + 1), write_expr_at(count, level + 1)),
        Expr::ArrayLiteral(elems, _) => {
            let elems_str: Vec<String> = elems.iter().map(|e| write_expr_at(e, level)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        Expr::ArraySized { elem_type, count, .. } => {
            format!("[{}; {}]", write_type(elem_type), write_expr_at(count, level))
        }
        Expr::Null(_) => "null".into(),
        Expr::Ref(inner, mutable, _) => {
            let inner_s = maybe_paren(expr_prec(inner) < 13, inner, level);
            if *mutable {
                format!("ref mut {}", inner_s)
            } else {
                format!("ref {}", inner_s)
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
            // #197：模板按源码转义输出（不展开 `\n` 等）
            format!("asm(\"{}\"{})", escape_string(template), extra)
        }
        Expr::Index { object, index, .. } => {
            // #190：索引对象同接收者（`(a - b)[0]`）
            let o = maybe_paren(expr_prec(object) < 14, object, level);
            format!("{}[{}]", o, write_expr_at(index, level))
        }
        Expr::CallExpr { target, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| write_expr_at(a, level)).collect();
            // #159：字段/低优先级目标必须加括号，否则 `(self.f)(x)` 会被重解析为方法调用
            let t = write_expr_at(target, level);
            let t = if matches!(target.as_ref(), Expr::FieldAccess { .. }) || expr_prec(target) < 14 {
                format!("({})", t)
            } else { t };
            format!("{}({})", t, args_str.join(", "))
        }
        Expr::TryOp(inner, _) => {
            format!("{}?", write_expr_at(inner, level))
        }
        Expr::Match { value, arms, .. } => {
            let mut out = format!("match {} {{", write_expr_at(value, level));
            for arm in arms {
                let guard = arm.guard.as_ref()
                    .map(|g| format!(" if {}", write_expr_at(g, level + 1)))
                    .unwrap_or_default();
                out.push_str(&format!("\n{}{}{} => {},", indent(level + 1), arm.pattern.display(), guard, write_match_body(&arm.body, level + 1)));
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
            let args_str: Vec<String> = args.iter().map(|a| super::helpers::format_macro_arg(a, level)).collect();
            format!("#{}({})", name.as_str().replace('.', "::"), args_str.join(", "))
        }
        Expr::Lambda { params, return_type, body, .. } => {
            let params_str: Vec<String> = params.iter().map(|(n, t)| format!("{} {}", write_type(t), n)).collect();
            // #169：未标注返回类型（哨兵 void）不能写成 `-> void`，否则宏展开/源码 round-trip
            // 后变成显式 void 标注，lambda 返回类型推断被禁用
            let ret = if crate::hir::lower::helpers::lambda_ret_unannotated(return_type) {
                String::new()
            } else {
                format!(" -> {}", write_type(return_type))
            };
            let mut out = format!("({}){} {{", params_str.join(", "), ret);
            for st in &body.stmts {
                out.push('\n');
                write_stmt(&mut out, st, level + 1);
            }
            if let Some(t) = &body.tail {
                out.push('\n');
                out.push_str(&format!("{}{}", indent(level + 1), write_expr_at(t, level + 1)));
            }
            while out.ends_with('\n') { out.pop(); }
            out.push_str(&format!("\n{}}}", indent(level)));
            out
        }
    }
}

/// asm 约束 → 源码关键字（当前仅支持 reg）
fn asm_constraint(c: &str) -> String {
    // #198：early-clobber 输出 `=&r` 还原为 `out(early reg)`
    if c.starts_with("=&") { "early reg".into() }
    else if c.contains('r') { "reg".into() }
    else { c.into() }
}

/// 字符串转义（与源码字符串字面量一致）：fmt 不得展开转义（#197 asm 模板换行）。
/// 仅控制字符用 `\xNN`；非 ASCII 按 UTF-8 原样（`\x` 只吃两位十六进制）。
pub(crate) fn escape_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control() => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
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
        Literal::String(s, _) => format!("\"{}\"", escape_string(s)),
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
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
    }
}

pub(super) fn vis_str(vis: &Visibility) -> &str {
    match vis {
        Visibility::Pub => "pub ",
        Visibility::PubCrate => "pub(crate) ",
        Visibility::Private => "",
    }
}
