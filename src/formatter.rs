use std::fmt::Write;
use crate::parser::ast::*;
use crate::parser::ast::Stmt::*;
use crate::parser::ast::vis::Visibility;
use crate::intern::Symbol;

const INDENT: &str = "    ";

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for (i, stmt) in program.stmts.iter().enumerate() {
        if i > 0 {
            write_stmt_separator(&mut out, stmt);
        }
        write_stmt(&mut out, stmt, 0);
    }
    let s = out.trim_end().to_string();
    if s.is_empty() { s } else { s + "\n" }
}

fn write_stmt_separator(out: &mut String, stmt: &Stmt) {
    match stmt {
        FnDecl { .. } | StructDef { .. } | EnumDef { .. } | InterfaceDef { .. }
        | ImplBlock { .. } | Namespace { .. } | Import { .. } => {
            out.push_str("\n");
        }
        _ => {}
    }
}

fn indent(level: usize) -> String {
    INDENT.repeat(level)
}

fn write_stmt(out: &mut String, stmt: &Stmt, level: usize) {
    match stmt {
        FnDecl { vis, is_inline, extern_c, name, generic_params, params, return_type, body, .. } => {
            let i = indent(level);
            if *extern_c {
                let _ = write!(out, "{}extern \"C\" fn {}", i, name);
            } else {
                if *is_inline {
                    let _ = write!(out, "{}inline fn {}", i, name);
                } else {
                    let vis_str = vis_str(vis);
                    let _ = write!(out, "{}{}fn {}", i, vis_str, name);
                }
            }
            write_generic_params(out, generic_params);
            write_params(out, params);
            write_return_type(out, return_type);
            if *extern_c && body.stmts.is_empty() {
                let _ = writeln!(out, ";");
            } else {
                let _ = write!(out, " ");
                write_block_same_line(out, body, level);
                let _ = writeln!(out);
            }
        }
        Assign { name, is_mut, value, .. } => {
            let i = indent(level);
            if *is_mut {
                let _ = writeln!(out, "{}mut {} = {};", i, name, write_expr(value));
            } else {
                let _ = writeln!(out, "{}{} = {};", i, name, write_expr(value));
            }
        }
        FieldAssign { object, field, value, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{}.{} = {};", i, write_expr(object), field, write_expr(value));
        }
        IndexAssign { object, index, value, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{}[{}] = {};", i, write_expr(object), write_expr(index), write_expr(value));
        }
        Return { value, .. } => {
            let i = indent(level);
            match value {
                Some(v) => { let _ = writeln!(out, "{}return {};", i, write_expr(v)); }
                None => { let _ = writeln!(out, "{}return;", i); }
            }
        }
        If { cond, then_block, elifs, else_block, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}if ", i);
            let _ = write!(out, "{} ", write_expr(cond));
            write_block_same_line(out, then_block, level);
            for (ec, eb) in elifs {
                let _ = write!(out, " else if {} ", write_expr(ec));
                write_block_same_line(out, eb, level);
            }
            if let Some(eb) = else_block {
                if eb.stmts.is_empty() {
                    let _ = writeln!(out, " else {{}}");
                } else {
                    let _ = write!(out, " else ");
                    write_block_same_line(out, eb, level);
                    let _ = writeln!(out);
                }
            } else {
                let _ = writeln!(out);
            }
        }
        For { iterator, start, end, step, body, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}for {} in (", i, iterator);
            let _ = write!(out, "{}", write_expr(start));
            let _ = write!(out, ", ");
            let _ = write!(out, "{}", write_expr(end));
            if let Some(s) = step {
                let _ = write!(out, ", {}", write_expr(s));
            }
            let _ = write!(out, ") ");
            write_block_same_line(out, body, level);
            let _ = writeln!(out);
        }
        While { cond, body, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}while {} ", i, write_expr(cond));
            write_block_same_line(out, body, level);
            let _ = writeln!(out);
        }
        Match { .. } => todo!(),
        Namespace { vis, name, items, .. } => {
            let i = indent(level);
            let vis_str = vis_str(vis);
            let _ = writeln!(out, "{}{}namespace {} {{", i, vis_str, name);
            for item in items {
                write_stmt(out, item, level + 1);
            }
            let _ = writeln!(out, "{}}}", i);
        }
        StructDef { vis, name, generic_params, fields, .. } => {
            let i = indent(level);
            let vis_str = vis_str(vis);
            let _ = write!(out, "{}{}struct {}", i, vis_str, name);
            write_generic_params(out, generic_params);
            if fields.is_empty() {
                let _ = writeln!(out, ";");
            } else {
                let _ = writeln!(out, " {{");
                for (fname, fty) in fields {
                    let _ = writeln!(out, "{}{} {}", indent(level + 1), write_type(fty), fname);
                }
                let _ = writeln!(out, "{}}}", i);
            }
        }
        EnumDef { vis, name, generic_params, variants, .. } => {
            let i = indent(level);
            let vis_str = vis_str(vis);
            let _ = write!(out, "{}{}enum {}", i, vis_str, name);
            write_generic_params(out, generic_params);
            let _ = writeln!(out, " {{");
            for v in variants {
                let _ = write!(out, "{}{}", indent(level + 1), v.name);
                match &v.fields {
                    crate::parser::ast::stmt::EnumFields::Named(fields) => {
                        let _ = writeln!(out, " {{");
                        for (fname_e, fty_e) in fields {
                            let _ = writeln!(out, "{}{} {}", indent(level + 2), write_type(fty_e), fname_e);
                        }
                        let _ = write!(out, "{}}}", indent(level + 1));
                    }
                    crate::parser::ast::stmt::EnumFields::Tuple(tys) => {
                        let _ = write!(out, "({})", tys.iter().map(write_type).collect::<Vec<_>>().join(", "));
                    }
                    crate::parser::ast::stmt::EnumFields::None => {}
                }
                let _ = writeln!(out, ",");
            }
            let _ = writeln!(out, "{}}}", i);
        }
        InterfaceDef { name, generic_params, methods, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}interface {}", i, name);
            write_generic_params(out, generic_params);
            let _ = writeln!(out, " {{");
            for m in methods {
                let _ = write!(out, "{}fn {}", indent(level + 1), m.name);
                let _ = write!(out, "({} self", m.self_keyword);
                for (pn, pt) in &m.params {
                    let _ = write!(out, ", {} {}", write_type(pt), pn);
                }
                let _ = write!(out, ")");
                write_return_type(out, &m.return_type);
                let _ = writeln!(out, ";");
            }
            let _ = writeln!(out, "{}}}", i);
        }
        ImplBlock { type_name, methods, generic_params, .. } => {
            let i = indent(level);
            let gp_str = if generic_params.is_empty() {
                String::new()
            } else {
                let params: Vec<String> = generic_params.iter()
                    .map(|(n, c)| if let Some(constraint) = c {
                        format!("{}: {}", n, constraint)
                    } else {
                        n.to_string()
                    })
                    .collect();
                format!("[{}]", params.join(", "))
            };
            let type_str = if generic_params.is_empty() {
                type_name.to_string()
            } else {
                let args: Vec<String> = generic_params.iter()
                    .map(|(n, _)| n.to_string())
                    .collect();
                format!("{}[{}]", type_name, args.join(", "))
            };
            let _ = writeln!(out, "{}impl{} {} {{", i, gp_str, type_str);
            for m in methods {
                write_stmt(out, m, level + 1);
            }
            let _ = writeln!(out, "{}}}", i);
        }
        Import { path, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}import \"{}\";", i, path);
        }
        ExprStmt { expr, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{};", i, write_expr(expr));
        }
        Break { .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}break;", i);
        }
        Continue { .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}continue;", i);
        }
    }
}

fn write_block_same_line(out: &mut String, block: &Block, level: usize) {
    if block.stmts.is_empty() {
        let _ = write!(out, "{{}}");
        return;
    }
    let i = indent(level);
    let _ = writeln!(out, "{{");
    for stmt in &block.stmts {
        write_stmt(out, stmt, level + 1);
    }
    let _ = write!(out, "{}}}", i);
}

fn write_generic_params(out: &mut String, params: &[(Symbol, Option<Symbol>)]) {
    if params.is_empty() { return; }
    let _ = write!(out, "[");
    for (i, (name, constraint)) in params.iter().enumerate() {
        if i > 0 { let _ = write!(out, ", "); }
        let _ = write!(out, "{}", name);
        if let Some(c) = constraint {
            let _ = write!(out, ": {}", c);
        }
    }
    let _ = write!(out, "]");
}

fn write_params(out: &mut String, params: &[(Symbol, Type)]) {
    let _ = write!(out, "(");
    for (i, (name, ty)) in params.iter().enumerate() {
        if i > 0 { let _ = write!(out, ", "); }
        // Format impl method self parameter: shared self / unique self
        if name.as_str() == "self" {
            match ty {
                Type::Shared(inner, _) if matches!(inner.as_ref(), Type::Named(_, _) | Type::Generic(_, _, _)) => {
                    let _ = write!(out, "shared self");
                    continue;
                }
                Type::Unique(inner, _) if matches!(inner.as_ref(), Type::Named(_, _) | Type::Generic(_, _, _)) => {
                    let _ = write!(out, "unique self");
                    continue;
                }
                _ => {}
            }
        }
        let _ = write!(out, "{} {}", write_type(ty), name);
    }
    let _ = write!(out, ")");
}

fn write_return_type(out: &mut String, ty: &Type) {
    match ty {
        Type::Void(_) => {}
        _ => {
            let _ = write!(out, " -> {}", write_type(ty));
        }
    }
}

fn write_type(ty: &Type) -> String {
    match ty {
        Type::Default | Type::FnPtr(..) => "???".into(),
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

fn write_expr(expr: &Expr) -> String {
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
    }
}

fn write_literal(lit: &Literal) -> String {
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

fn write_bin_op(op: &BinaryOp) -> &str {
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

fn vis_str(vis: &Visibility) -> &str {
    match vis {
        Visibility::Pub => "pub ",
        Visibility::PubCrate => "pub(crate) ",
        Visibility::Private => "",
    }
}

/// Format a single file: parse and re-format.
pub fn format_file(code: &str) -> Result<String, String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();
    let filtered: Vec<_> = tokens.into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();
    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()?;
    Ok(format_program(&program))
}
