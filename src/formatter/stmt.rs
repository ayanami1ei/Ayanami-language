use super::*;
use super::helpers::*;
use super::expr::*;
use super::types::write_type;

/// #179：语句写入（前置整行注释 + 行尾注释回插）。
pub(super) fn write_stmt(out: &mut String, stmt: &Stmt, level: usize) {
    let span = stmt.span();
    super::comments::flush_before(out, span.start_line, level);
    write_stmt_inner(out, stmt, level);
    super::comments::flush_trailing(out, span.end_line, span.end_col);
}

fn write_stmt_inner(out: &mut String, stmt: &Stmt, level: usize) {
    match stmt {
        FnDecl { attrs, vis, is_inline, extern_c, is_unsafe, name, generic_params, params, param_attrs, return_type, body, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let unsafe_kw = if *is_unsafe { "unsafe " } else { "" };
            if *extern_c {
                let _ = write!(out, "{}{}extern \"C\" fn {}", i, unsafe_kw, name);
            } else {
                if *is_inline {
                    let _ = write!(out, "{}{}inline fn {}", i, unsafe_kw, name);
                } else {
                    let vis_str = vis_str(vis);
                    let _ = write!(out, "{}{}{}fn {}", i, unsafe_kw, vis_str, name);
                }
            }
            write_generic_params(out, generic_params);
            write_params(out, params, param_attrs);
            write_return_type(out, return_type);
            if *extern_c && body.stmts.is_empty() {
                let _ = writeln!(out, ";");
            } else {
                let _ = write!(out, " ");
                write_block_same_line(out, body, level);
                let _ = writeln!(out);
            }
        }
        Block(block) => {
            write_block_same_line(out, block, level);
            let _ = writeln!(out);
        }
        Unsafe { block, .. } => {
            let _ = write!(out, "{}unsafe ", indent(level));
            write_block_same_line(out, block, level);
            let _ = writeln!(out);
        }
        Assign { name, is_mut, value, .. } => {
            let i = indent(level);
            if *is_mut {
                let _ = writeln!(out, "{}mut {} = {};", i, name, write_expr_at(value, level));
            } else {
                let _ = writeln!(out, "{}{} = {};", i, name, write_expr_at(value, level));
            }
        }
        FieldAssign { object, field, value, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{}.{} = {};", i, write_expr_at(object, level), field, write_expr_at(value, level));
        }
        IndexAssign { object, index, value, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{}[{}] = {};", i, write_expr_at(object, level), write_expr_at(index, level), write_expr_at(value, level));
        }
        DerefAssign { target, value, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}*{} = {};", i, write_expr_at(target, level), write_expr_at(value, level));
        }
        Return { value, .. } => {
            let i = indent(level);
            match value {
                Some(v) => { let _ = writeln!(out, "{}return {};", i, write_expr_at(v, level)); }
                None => { let _ = writeln!(out, "{}return;", i); }
            }
        }
        If { cond, then_block, elifs, else_block, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}if ", i);
            let _ = write!(out, "{} ", write_expr_at(cond, level));
            write_block_same_line(out, then_block, level);
            for (ec, eb) in elifs {
                // 语法是 `elif`，不是 `else if`
                let _ = write!(out, " elif {} ", write_expr_at(ec, level));
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
        ForIn { iterator, iterable, body, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}for {} in {} ", i, iterator, write_expr_at(iterable, level));
            write_block_same_line(out, body, level);
            let _ = writeln!(out);
        }
        For { iterator, start, end, step, body, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}for {} in (", i, iterator);
            let _ = write!(out, "{}", write_expr_at(start, level));
            let _ = write!(out, ", ");
            let _ = write!(out, "{}", write_expr_at(end, level));
            if let Some(s) = step {
                let _ = write!(out, ", {}", write_expr_at(s, level));
            }
            let _ = write!(out, ") ");
            write_block_same_line(out, body, level);
            let _ = writeln!(out);
        }
        While { cond, body, .. } => {
            let i = indent(level);
            let _ = write!(out, "{}while {} ", i, write_expr_at(cond, level));
            write_block_same_line(out, body, level);
            let _ = writeln!(out);
        }
        Match { value, arms, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}match {} {{", i, write_expr_at(value, level));
            for arm in arms {
                let _ = write!(out, "{}{}", indent(level + 1), arm.pattern.display());
                if let Some(g) = &arm.guard {
                    let _ = write!(out, " if {}", write_expr_at(g, level + 1));
                }
                let _ = writeln!(out, " => {},", super::helpers::write_match_body(&arm.body, level + 1));
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
        // 定义类（struct/enum/interface）见 stmt_defs.rs（#179 注释回插 + 保持 ≤300 行）
        StructDef { .. } | EnumDef { .. } | InterfaceDef { .. } => {
            super::stmt_defs::write_def(out, stmt, level);
        }
        ImplBlock { attrs, type_name, methods, generic_params, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let gp_str = if generic_params.is_empty() {
                String::new()
            } else {
                let params: Vec<String> = generic_params.iter()
                    .map(|(n, c)| if c.is_empty() {
                        n.to_string()
                    } else {
                        // #159：导出源码需可被解析器重解析（约束内部为 `A<B>`，统一写 `A[B]`）
                        let text: Vec<String> = c.iter()
                            .map(|x| x.as_str().replace('<', "[").replace('>', "]"))
                            .collect();
                        format!("{}: {}", n, text.join(" + "))
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
                // impl 方法默认公开（解析器标为 Public）；格式化不显式写 pub
                let mut m2 = m.clone();
                if let FnDecl { vis, .. } = &mut m2 {
                    *vis = Visibility::Private;
                }
                write_stmt(out, &m2, level + 1);
            }
            let _ = writeln!(out, "{}}}", i);
        }
        Attributed { attrs, stmt, .. } => {
            write_attrs(out, attrs, level);
            write_stmt(out, stmt, level);
        }
        StaticDecl { attrs, vis, is_mut, name, ty, value, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let m = if *is_mut { "mut " } else { "" };
            let ann = ty.as_ref().map(|t| format!(": {}", write_type(t))).unwrap_or_default();
            let _ = writeln!(out, "{}{}static {}{}{} = {};", i, vis_str(vis), m, name, ann, write_expr(value));
        }
        ConstDecl { attrs, vis, name, ty, value, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let ann = ty.as_ref().map(|t| format!(": {}", write_type(t))).unwrap_or_default();
            let _ = writeln!(out, "{}{}const {}{} = {};", i, vis_str(vis), name, ann, write_expr(value));
        }
        Import { path, macros, .. } => {
            let i = indent(level);
            if macros.is_empty() {
                let _ = writeln!(out, "{}import \"{}\";", i, path);
            } else {
                let list: Vec<String> = macros.iter().map(|m| m.as_str()).collect();
                let _ = writeln!(out, "{}import \"{}\" {{ {} }};", i, path, list.join(", "));
            }
        }
        ExprStmt { expr, .. } => {
            let i = indent(level);
            let _ = writeln!(out, "{}{};", i, write_expr_at(expr, level));
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
