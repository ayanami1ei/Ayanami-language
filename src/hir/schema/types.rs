//! HIR 类型/字面量 schema 编解码。

use serde_json::{json, Value};
use crate::error::Result;
use crate::hir::ty::{HirLiteral, HirType};
use crate::intern::Symbol;
use super::core::{bool_field, err, field, str_field, u64_field, FromJson, ToJson};

impl ToJson for HirType {
    fn to_json(&self) -> Value {
        match self {
            HirType::Int => json!({"kind": "int"}),
            HirType::Float => json!({"kind": "float"}),
            HirType::F32 => json!({"kind": "f32"}),
            HirType::Char => json!({"kind": "char"}),
            HirType::Void => json!({"kind": "void"}),
            HirType::Never => json!({"kind": "never"}),
            HirType::Bool => json!({"kind": "bool"}),
            HirType::Unique(inner) => json!({"kind": "unique", "inner": inner.to_json()}),
            HirType::Named(name) => json!({"kind": "named", "name": name.to_json()}),
            HirType::FatPtr { name, kind } => json!({"kind": "fat_ptr", "name": name.to_json(), "ownership": kind.to_json()}),
            HirType::Array(inner) => json!({"kind": "array", "inner": inner.to_json()}),
            HirType::ArraySized(inner, size) => json!({"kind": "array_sized", "inner": inner.to_json(), "size": size}),
            HirType::Ref(inner, mutable) => json!({"kind": "ref", "inner": inner.to_json(), "mutable": mutable}),
            HirType::FnPtr(params, ret) => json!({"kind": "fn_ptr", "params": params.to_json(), "return": ret.to_json()}),
            HirType::Closure(params, ret, owns_env, once) => json!({
                "kind": "closure", "params": params.to_json(), "return": ret.to_json(),
                "owns_env": owns_env, "once": once
            }),
            HirType::IntN { bits, signed } => json!({"kind": "int_n", "bits": bits, "signed": signed}),
            HirType::RawPtr(inner) => json!({"kind": "raw_ptr", "inner": inner.to_json()}),
        }
    }
}
impl FromJson for HirType {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match str_field(v, "kind")? {
            "int" => HirType::Int,
            "float" => HirType::Float,
            "f32" => HirType::F32,
            "char" => HirType::Char,
            "void" => HirType::Void,
            "never" => HirType::Never,
            "bool" => HirType::Bool,
            "unique" => HirType::Unique(Box::new(HirType::from_json(field(v, "inner")?)?)),
            "named" => HirType::Named(Symbol::from_json(field(v, "name")?)?),
            "fat_ptr" => HirType::FatPtr {
                name: Symbol::from_json(field(v, "name")?)?,
                kind: Box::new(HirType::from_json(field(v, "ownership")?)?),
            },
            "array" => HirType::Array(Box::new(HirType::from_json(field(v, "inner")?)?)),
            "array_sized" => HirType::ArraySized(
                Box::new(HirType::from_json(field(v, "inner")?)?),
                usize::from_json(field(v, "size")?)?,
            ),
            "ref" => HirType::Ref(
                Box::new(HirType::from_json(field(v, "inner")?)?),
                bool::from_json(field(v, "mutable")?)?,
            ),
            "fn_ptr" => HirType::FnPtr(
                Vec::from_json(field(v, "params")?)?,
                Box::new(HirType::from_json(field(v, "return")?)?),
            ),
            "closure" => HirType::Closure(
                Vec::from_json(field(v, "params")?)?,
                Box::new(HirType::from_json(field(v, "return")?)?),
                bool::from_json(field(v, "owns_env")?)?,
                bool::from_json(field(v, "once")?)?,
            ),
            "int_n" => HirType::IntN {
                bits: u64_field(v, "bits")? as u8,
                signed: bool_field(v, "signed")?,
            },
            "raw_ptr" => HirType::RawPtr(Box::new(HirType::from_json(field(v, "inner")?)?)),
            other => return Err(err(format!("unknown HIR type '{}'", other))),
        })
    }
}

impl ToJson for HirLiteral {
    fn to_json(&self) -> Value {
        match self {
            HirLiteral::Int(v) => json!({"kind": "int", "value": v}),
            HirLiteral::Float(v) => json!({"kind": "float", "value": v}),
            HirLiteral::Char(v) => json!({"kind": "char", "value": v.to_string()}),
            HirLiteral::String(v) => json!({"kind": "string", "value": v}),
            HirLiteral::Bool(v) => json!({"kind": "bool", "value": v}),
            HirLiteral::Array(vs) => json!({"kind": "array", "values": vs.to_json()}),
            HirLiteral::Struct(vs) => json!({"kind": "struct", "values": vs.to_json()}),
        }
    }
}
impl FromJson for HirLiteral {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match str_field(v, "kind")? {
            "int" => HirLiteral::Int(i64::from_json(field(v, "value")?)?),
            "float" => HirLiteral::Float(field(v, "value")?.as_f64().ok_or_else(|| err("float literal"))?),
            "char" => HirLiteral::Char(
                str_field(v, "value")?.chars().next().ok_or_else(|| err("char literal"))?,
            ),
            "string" => HirLiteral::String(String::from_json(field(v, "value")?)?),
            "bool" => HirLiteral::Bool(bool::from_json(field(v, "value")?)?),
            "array" => HirLiteral::Array(Vec::from_json(field(v, "values")?)?),
            "struct" => HirLiteral::Struct(Vec::from_json(field(v, "values")?)?),
            other => return Err(err(format!("unknown HIR literal '{}'", other))),
        })
    }
}
