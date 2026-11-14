//! 运算符/属性 schema 编解码。

use serde_json::{json, Value};
use crate::error::Result;
use crate::parser::ast::{BinaryOp, UnaryOp};
use crate::span::Span;
use super::core::{err, field, str_field, FromJson, ToJson};

impl ToJson for BinaryOp {
    fn to_json(&self) -> Value {
        json!(match self {
            BinaryOp::Add => "add", BinaryOp::Sub => "sub", BinaryOp::Mul => "mul",
            BinaryOp::Div => "div", BinaryOp::Mod => "mod", BinaryOp::Eq => "eq",
            BinaryOp::Neq => "neq", BinaryOp::Lt => "lt", BinaryOp::Gt => "gt",
            BinaryOp::Le => "le", BinaryOp::Ge => "ge", BinaryOp::And => "and",
            BinaryOp::Or => "or", BinaryOp::BitAnd => "bit_and", BinaryOp::BitOr => "bit_or",
            BinaryOp::BitXor => "bit_xor", BinaryOp::Shl => "shl", BinaryOp::Shr => "shr",
        })
    }
}
impl FromJson for BinaryOp {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match v.as_str().ok_or_else(|| err("binary op"))? {
            "add" => BinaryOp::Add, "sub" => BinaryOp::Sub, "mul" => BinaryOp::Mul,
            "div" => BinaryOp::Div, "mod" => BinaryOp::Mod, "eq" => BinaryOp::Eq,
            "neq" => BinaryOp::Neq, "lt" => BinaryOp::Lt, "gt" => BinaryOp::Gt,
            "le" => BinaryOp::Le, "ge" => BinaryOp::Ge, "and" => BinaryOp::And,
            "or" => BinaryOp::Or, "bit_and" => BinaryOp::BitAnd,
            "bit_or" => BinaryOp::BitOr, "bit_xor" => BinaryOp::BitXor,
            "shl" => BinaryOp::Shl, "shr" => BinaryOp::Shr,
            other => return Err(err(format!("unknown binary op '{}'", other))),
        })
    }
}
impl ToJson for UnaryOp {
    fn to_json(&self) -> Value {
        json!(match self {
            UnaryOp::Neg => "neg", UnaryOp::Not => "not",
            UnaryOp::BitNot => "bit_not", UnaryOp::Deref => "deref",
        })
    }
}
impl FromJson for UnaryOp {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match v.as_str().ok_or_else(|| err("unary op"))? {
            "neg" => UnaryOp::Neg, "not" => UnaryOp::Not,
            "bit_not" => UnaryOp::BitNot, "deref" => UnaryOp::Deref,
            other => return Err(err(format!("unknown unary op '{}'", other))),
        })
    }
}

impl ToJson for crate::parser::ast::Attr {
    fn to_json(&self) -> Value {
        let path = self.path_str();
        let text = if self.args.is_empty() {
            path
        } else {
            let args: Vec<String> = self.args.iter()
                .map(crate::formatter::format_attr_arg)
                .collect();
            format!("{}({})", path, args.join(", "))
        };
        json!({"text": text, "span": self.span.to_json()})
    }
}
impl FromJson for crate::parser::ast::Attr {
    fn from_json(v: &Value) -> Result<Self> {
        let text = str_field(v, "text")?;
        let mut attrs = crate::parser::parse_attr_list(&format!("#[{}]", text))?;
        let mut attr = attrs.pop().ok_or_else(|| err("empty attribute text"))?;
        if let Ok(span) = Span::from_json(field(v, "span")?) {
            attr.span = span;
        }
        Ok(attr)
    }
}
