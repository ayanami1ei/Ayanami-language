//! HIR 语句/块 schema 编解码。

use serde_json::Value;

use crate::error::{Error, Result};
use crate::hir::*;

use super::core::{field, str_field, FromJson, ToJson};

macro_rules! jobj {
    ({ $($key:literal : $value:expr),* $(,)? }) => {{
        let mut map = serde_json::Map::new();
        $(map.insert($key.to_string(), ($value).to_json());)*
        Value::Object(map)
    }};
}

impl ToJson for HirBlock {
    fn to_json(&self) -> Value { self.stmts.to_json() }
}
impl FromJson for HirBlock {
    fn from_json(v: &Value) -> Result<Self> { Ok(HirBlock { stmts: Vec::from_json(v)? }) }
}

impl ToJson for HirStmt {
    fn to_json(&self) -> Value {
        match self {
            HirStmt::Assign { target, value, span } => jobj!({"kind": "assign", "target": target, "value": value, "span": span}),
            HirStmt::FieldAssign { object, field, field_index, field_ty, value, span } => jobj!({
                "kind": "field_assign", "object": object, "field": field,
                "field_index": field_index, "field_ty": field_ty, "value": value, "span": span,
            }),
            HirStmt::IndexAssign { object, index, value, span } => jobj!({
                "kind": "index_assign", "object": object, "index": index, "value": value, "span": span,
            }),
            HirStmt::DerefAssign { target, value, span } => jobj!({
                "kind": "deref_assign", "target": target, "value": value, "span": span,
            }),
            HirStmt::Return { value, span } => jobj!({"kind": "return", "value": value, "span": span}),
            HirStmt::If { cond, then_block, elifs, else_block, span } => jobj!({
                "kind": "if", "cond": cond, "then_block": then_block,
                "elifs": elifs, "else_block": else_block, "span": span,
            }),
            HirStmt::While { cond, body, span } => jobj!({"kind": "while", "cond": cond, "body": body, "span": span}),
            HirStmt::Break { span } => jobj!({"kind": "break", "span": span}),
            HirStmt::Continue { span } => jobj!({"kind": "continue", "span": span}),
            HirStmt::Expr { expr, span } => jobj!({"kind": "expr", "expr": expr, "span": span}),
            HirStmt::Assume { cond, span } => jobj!({"kind": "assume", "cond": cond, "span": span}),
            HirStmt::Contract { kind, cond, line, col } => jobj!({
                "kind": "contract", "contract": kind.label(), "cond": cond, "line": line, "col": col,
            }),
            HirStmt::Block { stmts, span } => jobj!({"kind": "block", "stmts": stmts, "span": span}),
        }
    }
}
impl FromJson for HirStmt {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match str_field(v, "kind")? {
            "assign" => HirStmt::Assign {
                target: FromJson::from_json(field(v, "target")?)?,
                value: FromJson::from_json(field(v, "value")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "field_assign" => HirStmt::FieldAssign {
                object: FromJson::from_json(field(v, "object")?)?,
                field: FromJson::from_json(field(v, "field")?)?,
                field_index: FromJson::from_json(field(v, "field_index")?)?,
                field_ty: FromJson::from_json(field(v, "field_ty")?)?,
                value: FromJson::from_json(field(v, "value")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "index_assign" => HirStmt::IndexAssign {
                object: FromJson::from_json(field(v, "object")?)?,
                index: FromJson::from_json(field(v, "index")?)?,
                value: FromJson::from_json(field(v, "value")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "deref_assign" => HirStmt::DerefAssign {
                target: FromJson::from_json(field(v, "target")?)?,
                value: FromJson::from_json(field(v, "value")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "return" => HirStmt::Return {
                value: Option::from_json(field(v, "value")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "if" => HirStmt::If {
                cond: FromJson::from_json(field(v, "cond")?)?,
                then_block: FromJson::from_json(field(v, "then_block")?)?,
                elifs: Vec::from_json(field(v, "elifs")?)?,
                else_block: Option::from_json(field(v, "else_block")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "while" => HirStmt::While {
                cond: FromJson::from_json(field(v, "cond")?)?,
                body: FromJson::from_json(field(v, "body")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "break" => HirStmt::Break { span: FromJson::from_json(field(v, "span")?)? },
            "continue" => HirStmt::Continue { span: FromJson::from_json(field(v, "span")?)? },
            "expr" => HirStmt::Expr {
                expr: FromJson::from_json(field(v, "expr")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "assume" => HirStmt::Assume {
                cond: FromJson::from_json(field(v, "cond")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            "contract" => HirStmt::Contract {
                kind: match str_field(v, "contract")? {
                    "require" => ContractKind::Require,
                    "ensure" => ContractKind::Ensure,
                    "invariant" => ContractKind::Invariant,
                    other => return Err(Error::Hir(format!("HIR schema: unknown contract '{}'", other))),
                },
                cond: FromJson::from_json(field(v, "cond")?)?,
                line: FromJson::from_json(field(v, "line")?)?,
                col: FromJson::from_json(field(v, "col")?)?,
            },
            "block" => HirStmt::Block {
                stmts: Vec::from_json(field(v, "stmts")?)?,
                span: FromJson::from_json(field(v, "span")?)?,
            },
            other => return Err(Error::Hir(format!("HIR schema: unknown statement '{}'", other))),
        })
    }
}
