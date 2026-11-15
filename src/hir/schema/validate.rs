//! A5e/B2c：transform 返回 JSON 的结构校验。
//!
//! JSON schema 解码能保证字段形状，但插件仍可能返回越界 FnId/VarId、未知结构体
//! 或非法 vtable。这里在 HIR 重建前做一层引用检查，避免后续 pass panic。

use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::error::{Error, Result};

use super::validate_call::validate_call;
use super::validate_fields::{validate_enum_node, validate_field_node, validate_struct_node};

pub(crate) fn validate(root: &Value) -> Result<()> {
    let schema = root.get("schema").and_then(Value::as_u64).unwrap_or(0);
    if schema != super::HIR_SCHEMA_VERSION as u64 {
        return Err(Error::Hir(format!(
            "transform HIR schema: unsupported version {}",
            schema
        )));
    }

    let mut fn_ids = HashSet::new();
    let mut structs = HashSet::new();
    let mut sigs = HashMap::new();
    let mut field_map = HashMap::new();
    collect_program(root, &mut fn_ids, &mut structs, &mut sigs, &mut field_map)?;
    validate_vtables(root, &fn_ids)?;
    validate_items(root, &fn_ids, &structs, &sigs, &field_map)?;
    Ok(())
}

#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct FieldSig {
    pub(super) name: String,
    pub(super) ty: Value,
}

#[derive(Clone)]
pub(super) struct FnSig {
    pub(super) params: Vec<Value>,
    pub(super) ret: Value,
    pub(super) extern_c: bool,
}

fn collect_program(
    root: &Value,
    fn_ids: &mut HashSet<u64>,
    structs: &mut HashSet<String>,
    sigs: &mut HashMap<u64, FnSig>,
    field_map: &mut HashMap<String, Vec<FieldSig>>,
) -> Result<()> {
    let items = root.get("items").and_then(Value::as_array)
        .ok_or_else(|| err("missing items array"))?;
    collect_items(items, fn_ids, structs, sigs)?;
    if let Some(imported) = root.get("imported_fns").and_then(Value::as_array) {
        for item in imported {
            if let Some(id) = item.get("fn_id").and_then(Value::as_u64) {
                fn_ids.insert(id);
                if let Some(sig) = fn_sig(item) {
                    sigs.entry(id).or_insert(sig);
                }
            }
        }
    }
    if let Some(defs) = root.get("struct_defs").and_then(Value::as_array) {
        for def in defs {
            if let Some(name) = def.get(0).and_then(Value::as_str) {
                structs.insert(name.to_string());
                let fields = def.get(1).and_then(Value::as_array).cloned().unwrap_or_default();
                let sigs = fields.iter().filter_map(|f| {
                    let name = f.get("name")?.as_str()?.to_string();
                    let ty = f.get("ty")?.clone();
                    Some(FieldSig { name, ty })
                }).collect();
                field_map.insert(name.to_string(), sigs);
            }
        }
    }
    Ok(())
}

fn fn_sig(value: &Value) -> Option<FnSig> {
    let params = value.get("params")?.as_array()?
        .iter()
        .map(|p| p.get(1).cloned().unwrap_or(Value::Null))
        .collect();
    Some(FnSig {
        params,
        ret: value.get("return_type")?.clone(),
        extern_c: value.get("extern_c").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn collect_items(
    items: &[Value],
    fn_ids: &mut HashSet<u64>,
    structs: &mut HashSet<String>,
    sigs: &mut HashMap<u64, FnSig>,
) -> Result<()> {
    for item in items {
        let Some(kind) = item.get("kind").and_then(Value::as_str) else { continue };
        match kind {
            "fn" => {
                let value = item.get("value").ok_or_else(|| err("fn item has no value"))?;
                if let Some(id) = value.get("fn_id").and_then(Value::as_u64) {
                    fn_ids.insert(id);
                    if let Some(sig) = fn_sig(value) {
                        sigs.entry(id).or_insert(sig);
                    }
                }
            }
            "struct" => {
                let value = item.get("value").ok_or_else(|| err("struct item has no value"))?;
                if let Some(name) = value.get("name").and_then(Value::as_str) {
                    structs.insert(name.to_string());
                }
            }
            "namespace" => {
                let nested = item.get("items").and_then(Value::as_array)
                    .ok_or_else(|| err("namespace has no items"))?;
                collect_items(nested, fn_ids, structs, sigs)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_vtables(root: &Value, fn_ids: &HashSet<u64>) -> Result<()> {
    let Some(vtables) = root.get("vtables").and_then(Value::as_array) else { return Ok(()) };
    for (i, vtable) in vtables.iter().enumerate() {
        let Some(ids) = vtable.get("method_fn_ids").and_then(Value::as_array) else { continue };
        for id in ids {
            let Some(id) = id.as_i64() else { continue };
            if id == -1 { continue; }
            if id < 0 || !fn_ids.contains(&(id as u64)) {
                return Err(err(format!(
                    "vtable[{}] references unknown FnId {}",
                    i, id
                )));
            }
        }
    }
    Ok(())
}

fn validate_items(
    root: &Value,
    fn_ids: &HashSet<u64>,
    structs: &HashSet<String>,
    sigs: &HashMap<u64, FnSig>,
    field_map: &HashMap<String, Vec<FieldSig>>,
) -> Result<()> {
    let items = root.get("items").and_then(Value::as_array)
        .ok_or_else(|| err("missing items array"))?;
    validate_item_list(items, fn_ids, structs, sigs, field_map)
}

fn validate_item_list(
    items: &[Value],
    fn_ids: &HashSet<u64>,
    structs: &HashSet<String>,
    sigs: &HashMap<u64, FnSig>,
    field_map: &HashMap<String, Vec<FieldSig>>,
) -> Result<()> {
    for item in items {
        let Some(kind) = item.get("kind").and_then(Value::as_str) else { continue };
        match kind {
            "fn" => {
                let value = item.get("value").ok_or_else(|| err("fn item has no value"))?;
                let locals = value.get("locals").and_then(Value::as_array)
                    .ok_or_else(|| err("fn has no locals array"))?;
                let body = value.get("body").ok_or_else(|| err("fn has no body"))?;
                walk_nodes(body, locals, fn_ids, structs, sigs, field_map, "fn.body")?;
            }
            "namespace" => {
                let nested = item.get("items").and_then(Value::as_array)
                    .ok_or_else(|| err("namespace has no items"))?;
                validate_item_list(nested, fn_ids, structs, sigs, field_map)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn walk_nodes(
    value: &Value,
    locals: &[Value],
    fn_ids: &HashSet<u64>,
    structs: &HashSet<String>,
    sigs: &HashMap<u64, FnSig>,
    field_map: &HashMap<String, Vec<FieldSig>>,
    path: &str,
) -> Result<()> {
    match value {
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                walk_nodes(item, locals, fn_ids, structs, sigs, field_map, &format!("{}.{}", path, i))?;
            }
        }
        Value::Object(map) => {
            if let Some(kind) = map.get("kind").and_then(Value::as_str) {
                match kind {
                    "call" | "fn_ptr" => {
                        // fn_ptr 的 -1 是未绑定/哨兵；call 必须指向真实函数。
                        let raw = map.get("fn_id");
                        if kind == "fn_ptr"
                            && (raw.is_none() || raw.and_then(Value::as_i64) == Some(-1))
                        {
                            // 允许未绑定函数指针。
                        } else {
                            let id = raw.and_then(Value::as_u64).ok_or_else(|| {
                                err(format!("{}: {} has invalid fn_id", path, kind))
                            })?;
                            if !fn_ids.contains(&id) {
                                return Err(err(format!(
                                    "{}: {} references unknown FnId {}",
                                    path, kind, id
                                )));
                            }
                            if kind == "call" {
                                validate_call(map, id, sigs, path)?;
                            }
                        }
                    }
                    "var" => {
                        let var = map.get("var").and_then(Value::as_u64)
                            .ok_or_else(|| err(format!("{}: var has invalid id", path)))?;
                        if var >= locals.len() as u64 {
                            return Err(err(format!(
                                "{}: var {} out of range (locals = {})",
                                path, var, locals.len()
                            )));
                        }
                        let local_ty = locals[var as usize].get("ty").unwrap_or(&Value::Null);
                        let node_ty = map.get("ty").unwrap_or(&Value::Null);
                        if !loose_type_eq(node_ty, local_ty) {
                            return Err(err(format!(
                                "{}: var {} type mismatch (node {}, local {})",
                                path, var, node_ty, local_ty
                            )));
                        }
                    }
                    "struct" => {
                        let name = map.get("type_name").and_then(Value::as_str)
                            .ok_or_else(|| err(format!("{}: struct has no type_name", path)))?;
                        if !structs.contains(name) {
                            return Err(err(format!(
                                "{}: struct references unknown type '{}'",
                                path, name
                            )));
                        }
                        validate_struct_node(map, field_map, path)?;
                    }
                    "enum_construct" => {
                        let name = map.get("variant_struct").and_then(Value::as_str)
                            .ok_or_else(|| err(format!("{}: enum_construct has no variant_struct", path)))?;
                        if !structs.contains(name) {
                            return Err(err(format!(
                                "{}: enum variant references unknown struct '{}'",
                                path, name
                            )));
                        }
                        validate_enum_node(map, field_map, path)?;
                    }
                    "field" => validate_field_node(map, field_map, path)?,
                    _ => {}
                }
            }
            for (key, child) in map {
                walk_nodes(child, locals, fn_ids, structs, sigs, field_map, &format!("{}.{}", path, key))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn loose_type_eq(actual: &Value, expected: &Value) -> bool {
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

fn err(message: impl std::fmt::Display) -> Error {
    Error::Hir(format!("transform HIR: {}", message))
}
