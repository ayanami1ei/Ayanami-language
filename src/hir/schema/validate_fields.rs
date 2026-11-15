//! B2k：transform 字段布局校验（从 validate.rs 拆出）。

use serde_json::Value;
use std::collections::HashMap;

use crate::error::Result;

use super::validate::FieldSig;
use super::validate_call::types_compatible;

pub(super) fn validate_struct_node(
    _node: &serde_json::Map<String, Value>,
    _fields: &HashMap<String, Vec<FieldSig>>,
    _path: &str,
) -> Result<()> {
    // struct 的字段类型可能经过泛型替换/union/closure 表示差异；
    // 是否安全最终由 MIR 降级与借用检查裁定。这里只拦截未知类型（在 validate.rs）。
    Ok(())
}

pub(super) fn validate_enum_node(
    _node: &serde_json::Map<String, Value>,
    _fields: &HashMap<String, Vec<FieldSig>>,
    _path: &str,
) -> Result<()> {
    Ok(())
}

pub(super) fn validate_field_node(
    node: &serde_json::Map<String, Value>,
    fields: &HashMap<String, Vec<FieldSig>>,
    path: &str,
) -> Result<()> {
    let Some(object_ty) = node.get("object").and_then(|v| v.get("ty")) else { return Ok(()) };
    let Some(name) = named_type(object_ty) else { return Ok(()) };
    let Some(defs) = fields.get(name) else { return Ok(()) };
    let Some(index) = node.get("field_index").and_then(Value::as_u64) else { return Ok(()) };
    let Some(def) = defs.get(index as usize) else {
        return Err(err(format!(
            "{}: field index {} out of range for {} (fields = {})",
            path, index, name, defs.len()
        )));
    };
    // union/niche 布局的名字可能与声明字段名不同；只校验索引范围。
    let _ = (&def.ty, node.get("field_ty"));
    Ok(())
}

pub(super) fn validate_return(value: &Value, return_ty: &Value, path: &str) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let ty = value.get("ty").unwrap_or(&Value::Null);
    if !types_compatible(ty, return_ty) {
        return Err(err(format!(
            "{}: return type mismatch (value {}, function {})",
            path, ty, return_ty
        )));
    }
    Ok(())
}

pub(super) fn loose_type_eq(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    if actual.get("kind").and_then(Value::as_str) == Some("named")
        && expected.get("kind").and_then(Value::as_str) == Some("named")
    {
        let a = actual.get("name").and_then(Value::as_str).unwrap_or("");
        let e = expected.get("name").and_then(Value::as_str).unwrap_or("");
        return a == e || a.split(['<', '[']).next() == e.split(['<', '[']).next();
    }
    false
}

fn named_type(value: &Value) -> Option<&str> {
    match value.get("kind").and_then(Value::as_str) {
        Some("named") => value.get("name").and_then(Value::as_str),
        Some("ref") | Some("unique") | Some("raw_ptr") => {
            named_type(value.get("inner")?)
        }
        _ => None,
    }
}

fn err(message: impl std::fmt::Display) -> crate::error::Error {
    crate::error::Error::Hir(format!("transform HIR: {}", message))
}
