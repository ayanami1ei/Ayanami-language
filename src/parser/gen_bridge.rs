// ── Bridge: Asuka generated parser Node → typed AST enums ──

use asuka::runtime::{Node, Value};
use crate::parser::ast::*;
use crate::intern::Symbol;
use crate::span::Span;

pub fn node_to_expr(node: &Node) -> Result<Expr, String> {
    match node.kind.as_str() {
        "IntLiteral" => {
            let val = node.get("value").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            let n = val.parse::<i64>().map_err(|_| "invalid int".to_string())?;
            Ok(Expr::Literal(Literal::Int(n, default_span())))
        }
        "FloatLiteral" => {
            let val = node.get("value").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            let n = val.parse::<f64>().map_err(|_| "invalid float".to_string())?;
            Ok(Expr::Literal(Literal::Float(n, default_span())))
        }
        "StringLiteral" => {
            let s = node.get("value").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            Ok(Expr::Literal(Literal::String(s, default_span())))
        }
        "BoolLiteral" => {
            let val = node.get("value").and_then(|v| if let Value::String(s) = v { Some(s == "true") } else { None })
                .unwrap_or(false);
            Ok(Expr::Literal(Literal::Bool(val, default_span())))
        }
        "CharLiteral" => {
            let val = node.get("value").and_then(|v| if let Value::String(s) = v { s.chars().next() } else { None })
                .unwrap_or('\0');
            Ok(Expr::Literal(Literal::Char(val, default_span())))
        }
        "Ident" => {
            let name = node.get("name").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            Ok(Expr::Ident(Symbol::intern(&name), default_span()))
        }
        "BinaryExpr" => {
            let op_str = node.get("op").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            let lhs = node.child("lhs").ok_or("missing lhs")?;
            let rhs = node.child("rhs").ok_or("missing rhs")?;
            let lhs_expr = node_to_expr(lhs)?;
            let rhs_expr = node_to_expr(rhs)?;
            let op = match op_str.as_str() {
                "+" => BinaryOp::Add, "-" => BinaryOp::Sub, "*" => BinaryOp::Mul,
                "/" => BinaryOp::Div, "%" => BinaryOp::Mod,
                "==" => BinaryOp::Eq, "!=" => BinaryOp::Neq,
                "<" => BinaryOp::Lt, ">" => BinaryOp::Gt, "<=" => BinaryOp::Le, ">=" => BinaryOp::Ge,
                "&&" => BinaryOp::And, "||" => BinaryOp::Or,
                _ => return Err(format!("unknown operator: {}", op_str)),
            };
            Ok(Expr::Binary { op, lhs: Box::new(lhs_expr), rhs: Box::new(rhs_expr), span: default_span() })
        }
        "UnaryExpr" => {
            let op_str = node.get("op").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_default();
            let arg = node.child("arg").ok_or("missing arg")?;
            let arg_expr = node_to_expr(arg)?;
            let op = match op_str.as_str() {
                "-" => UnaryOp::Neg, "!" => UnaryOp::Not,
                _ => return Err(format!("unknown unary op: {}", op_str)),
            };
            Ok(Expr::Unary { op, arg: Box::new(arg_expr), span: default_span() })
        }
        "FnCallExpr" | "CallExpr" => {
            let is_fncall = node.kind == "FnCallExpr";
            let name = if is_fncall {
                node.get("name").and_then(|v| if let Value::String(s) = v { Some(Symbol::intern(s)) } else { None })
            } else {
                None
            };
            let target = node.child("target");
            let args = node.children("args");
            let hir_args: Vec<Expr> = args.iter().map(|a| node_to_expr(a)).collect::<Result<_, _>>()?;
            if let Some(n) = name {
                Ok(Expr::FnCall { name: n, args: hir_args, span: default_span() })
            } else if let Some(t) = target {
                let t_expr = node_to_expr(t)?;
                Ok(Expr::CallExpr { target: Box::new(t_expr), args: hir_args, span: default_span() })
            } else {
                Err("call target not found".to_string())
            }
        }
        kind => Err(format!("unknown expr kind: {}", kind)),
    }
}

fn default_span() -> Span {
    Span::default()
}
