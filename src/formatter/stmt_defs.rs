//! 定义类语句（struct/union/enum/interface）格式化（从 stmt.rs 拆出）。
//!
//! #179：字段/变体/接口方法处的注释回插（前置整行 + 行尾）。

use super::*;
use super::helpers::*;
use super::types::write_type;
use super::expr::vis_str;

pub(super) fn write_def(out: &mut String, stmt: &Stmt, level: usize) {
    match stmt {
        StructDef { attrs, vis, name, generic_params, is_union, fields, field_attrs, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let vis_str = vis_str(vis);
            let kw = if *is_union { "union" } else { "struct" };
            let _ = write!(out, "{}{}{} {}", i, vis_str, kw, name);
            write_generic_params(out, generic_params);
            if fields.is_empty() {
                let _ = writeln!(out, ";");
            } else {
                let _ = writeln!(out, " {{");
                for (idx, (fname, fty)) in fields.iter().enumerate() {
                    // #179：字段注释回插（前置整行 / 行尾）
                    super::comments::flush_before(out, fty.span().start_line, level + 1);
                    if let Some(fa) = field_attrs.get(idx) {
                        write_attrs(out, fa, level + 1);
                    }
                    let _ = writeln!(out, "{}{} {}", indent(level + 1), write_type(fty), fname);
                    super::comments::flush_trailing(out, fty.span().end_line, fty.span().end_col);
                }
                let _ = writeln!(out, "{}}}", i);
            }
        }
        EnumDef { attrs, vis, name, generic_params, variants, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let vis_str = vis_str(vis);
            let _ = write!(out, "{}{}enum {}", i, vis_str, name);
            write_generic_params(out, generic_params);
            let _ = writeln!(out, " {{");
            for v in variants {
                // #179：变体注释回插（载荷类型 span 定位；无载荷变体顺延）
                let vline = match &v.fields {
                    crate::parser::ast::stmt::EnumFields::Named(fs) => fs.first().map(|(_, t)| t.span().start_line),
                    crate::parser::ast::stmt::EnumFields::Tuple(ts) => ts.first().map(|t| t.span().start_line),
                    crate::parser::ast::stmt::EnumFields::None => None,
                };
                if let Some(l) = vline { super::comments::flush_before(out, l, level + 1); }
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
                let vend = match &v.fields {
                    crate::parser::ast::stmt::EnumFields::Named(fs) => fs.last().map(|(_, t)| (t.span().end_line, t.span().end_col)),
                    crate::parser::ast::stmt::EnumFields::Tuple(ts) => ts.last().map(|t| (t.span().end_line, t.span().end_col)),
                    crate::parser::ast::stmt::EnumFields::None => None,
                };
                if let Some((l, c)) = vend { super::comments::flush_trailing(out, l, c); }
            }
            let _ = writeln!(out, "{}}}", i);
        }
        InterfaceDef { attrs, name, generic_params, methods, .. } => {
            write_attrs(out, attrs, level);
            let i = indent(level);
            let _ = write!(out, "{}interface {}", i, name);
            write_generic_params(out, generic_params);
            let _ = writeln!(out, " {{");
            for m in methods {
                // #179：接口方法注释回插（返回类型 span 定位；Default 跳过）
                if !matches!(m.return_type, crate::parser::ast::Type::Default) {
                    super::comments::flush_before(out, m.return_type.span().start_line, level + 1);
                }
                write_attrs(out, &m.attrs, level + 1);
                let _ = write!(out, "{}fn {}", indent(level + 1), m.name);
                // self 关键字往返：self / ref self / ref mut self（refmut 必须拆开写）
                let self_str = match m.self_keyword.as_str().as_str() {
                    "ref" => "ref self",
                    "refmut" => "ref mut self",
                    _ => "self",
                };
                let _ = write!(out, "({}", self_str);
                for (pn, pt) in &m.params {
                    let _ = write!(out, ", {} {}", write_type(pt), pn);
                }
                let _ = write!(out, ")");
                write_return_type(out, &m.return_type);
                let _ = writeln!(out, ";");
                if !matches!(m.return_type, crate::parser::ast::Type::Default) {
                    super::comments::flush_trailing(out, m.return_type.span().end_line, m.return_type.span().end_col);
                }
            }
            let _ = writeln!(out, "{}}}", i);
        }
        _ => {}
    }
}
