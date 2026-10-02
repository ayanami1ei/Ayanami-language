use super::*;
use super::helpers::*;
use super::expr::*;

pub(super) fn write_stmt(out: &mut String, stmt: &Stmt, level: usize) {
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
        Match { value, arms, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}match {} {{", i, write_expr(value));
            for arm in arms {
                let _ = write!(out, "{}{}", indent(level + 1), arm.variant_name);
                if !arm.bindings.is_empty() {
                    let _ = write!(out, "({})", arm.bindings.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "));
                }
                let _ = writeln!(out, " => {},", write_expr(&arm.body));
            }
            let _ = writeln!(out, "{}}}", i);
        }
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
