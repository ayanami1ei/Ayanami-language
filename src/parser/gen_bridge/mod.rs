// ── Bridge: Asuka generated parser Node → typed AST enums ──

use crate::error::{Error, Result};
use asuka::runtime::{Node, Value};
use crate::parser::ast::*;
use crate::parser::ast::vis::Visibility;
use crate::intern::Symbol;
use crate::span::Span;

pub fn node_to_program(node: &Node) -> Result<crate::parser::ast::block::Block> {
    let items = node.children("items");
    let mut stmts = Vec::new();
    for item in &items {
        stmts.push(node_to_stmt(item)?);
    }
    Ok(crate::parser::ast::block::Block::new(stmts, default_span()))
}


mod expr;
mod stmt;
mod ty;
pub use expr::node_to_expr;
pub use stmt::node_to_stmt;
pub use ty::type_from_node;

fn get_str(node: &Node, field: &str) -> Result<String> {
    node.get(field).and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .ok_or_else(|| Error::Parse(format!("missing field '{}' in node '{}'", field, node.kind)))
}

fn str_to_binop(s: &str) -> Result<BinaryOp> {
    match s {
        "+" => Ok(BinaryOp::Add), "-" => Ok(BinaryOp::Sub),
        "*" => Ok(BinaryOp::Mul), "/" => Ok(BinaryOp::Div), "%" => Ok(BinaryOp::Mod),
        "==" => Ok(BinaryOp::Eq), "!=" => Ok(BinaryOp::Neq),
        "<" => Ok(BinaryOp::Lt), ">" => Ok(BinaryOp::Gt),
        "<=" => Ok(BinaryOp::Le), ">=" => Ok(BinaryOp::Ge),
        "&&" => Ok(BinaryOp::And), "||" => Ok(BinaryOp::Or),
        _ => Err(Error::Parse(format!("unknown operator: {}", s))),
    }
}

fn params_from_node(node: &Node) -> Result<Vec<(Symbol, Type)>> {
    let mut params = Vec::new();
    for child in node.children("items") {
        let ty = type_from_node(child.child("type").ok_or_else(|| Error::Parse("missing param type".into()))?)?;
        let name = get_str(child, "name")?;
        params.push((Symbol::intern(&name), ty));
    }
    Ok(params)
}

fn block_from_node(node: &Node) -> Result<crate::parser::ast::block::Block> {
    let stmts = node.children("stmts").iter().map(|n| node_to_stmt(*n)).collect::<std::result::Result<_, _>>()?;
    Ok(crate::parser::ast::block::Block::new(stmts, default_span()))
}

fn default_span() -> Span {
    Span::default()
}
