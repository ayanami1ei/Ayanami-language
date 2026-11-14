//! HIR 程序、函数、语句与 item 的 schema 编解码。

use serde_json::Value;

use crate::error::{Error, Result};
use crate::hir::*;

use super::core::{bool_field, field, str_field, u64_field, FromJson, ToJson};
use super::HIR_SCHEMA_VERSION;

macro_rules! jobj {
    ({ $($key:literal : $value:expr),* $(,)? }) => {{
        let mut map = serde_json::Map::new();
        $(map.insert($key.to_string(), ($value).to_json());)*
        Value::Object(map)
    }};
}

impl ToJson for HirLocal {
    fn to_json(&self) -> Value {
        jobj!({"name": self.name.to_json(), "ty": self.ty.to_json(), "mutable": self.mutable.to_json(), "is_result": self.is_result.to_json()})
    }
}
impl FromJson for HirLocal {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirLocal {
            name: FromJson::from_json(field(v, "name")?)?,
            ty: FromJson::from_json(field(v, "ty")?)?,
            mutable: bool_field(v, "mutable")?,
            is_result: bool_field(v, "is_result")?,
        })
    }
}

impl ToJson for HirStructField {
    fn to_json(&self) -> Value { jobj!({"name": self.name.to_json(), "ty": self.ty.to_json()}) }
}
impl FromJson for HirStructField {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirStructField {
            name: FromJson::from_json(field(v, "name")?)?,
            ty: FromJson::from_json(field(v, "ty")?)?,
        })
    }
}

impl ToJson for HirStructDef {
    fn to_json(&self) -> Value {
        jobj!({"name": self.name.to_json(), "fields": self.fields.to_json(), "attrs": self.attrs.to_json()})
    }
}
impl FromJson for HirStructDef {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirStructDef {
            name: FromJson::from_json(field(v, "name")?)?,
            fields: Vec::from_json(field(v, "fields")?)?,
            // B2f 新字段：旧 transform 插件返回的 schema 1 包没有 attrs，按空处理。
            attrs: match field(v, "attrs") {
                Ok(value) => Vec::from_json(value)?,
                Err(_) => Vec::new(),
            },
        })
    }
}

impl ToJson for HirInterfaceMethod {
    fn to_json(&self) -> Value {
        jobj!({"name": self.name.to_json(), "self_keyword": self.self_keyword.to_json(), "params": self.params.to_json(), "return_type": self.return_type.to_json()})
    }
}
impl FromJson for HirInterfaceMethod {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirInterfaceMethod {
            name: FromJson::from_json(field(v, "name")?)?,
            self_keyword: FromJson::from_json(field(v, "self_keyword")?)?,
            params: Vec::from_json(field(v, "params")?)?,
            return_type: FromJson::from_json(field(v, "return_type")?)?,
        })
    }
}

impl ToJson for VtableEntry {
    fn to_json(&self) -> Value {
        jobj!({"concrete_type": self.concrete_type.to_json(), "interface": self.interface.to_json(), "method_fn_ids": self.method_fn_ids.to_json()})
    }
}
impl FromJson for VtableEntry {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(VtableEntry {
            concrete_type: FromJson::from_json(field(v, "concrete_type")?)?,
            interface: FromJson::from_json(field(v, "interface")?)?,
            method_fn_ids: Vec::from_json(field(v, "method_fn_ids")?)?,
        })
    }
}

impl ToJson for ImportedFnSig {
    fn to_json(&self) -> Value {
        jobj!({
            "fn_id": self.fn_id.to_json(), "name": self.name.to_json(), "params": self.params.to_json(),
            "return_type": self.return_type.to_json(), "attrs": self.attrs.to_json(),
            "effects": self.effects.to_json(), "inferred": self.inferred.to_json(), "extern_c": self.extern_c.to_json(),
        })
    }
}
impl FromJson for ImportedFnSig {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(ImportedFnSig {
            fn_id: FromJson::from_json(field(v, "fn_id")?)?,
            name: FromJson::from_json(field(v, "name")?)?,
            params: Vec::from_json(field(v, "params")?)?,
            return_type: FromJson::from_json(field(v, "return_type")?)?,
            attrs: Vec::from_json(field(v, "attrs")?)?,
            effects: FromJson::from_json(field(v, "effects")?)?,
            inferred: FromJson::from_json(field(v, "inferred")?)?,
            extern_c: bool_field(v, "extern_c")?,
        })
    }
}

impl ToJson for HirStatic {
    fn to_json(&self) -> Value {
        jobj!({
            "name": self.name.to_json(), "ty": self.ty.to_json(), "value": self.value.to_json(),
            "is_mut": self.is_mut.to_json(), "is_pub": self.is_pub.to_json(), "is_external": self.is_external.to_json(),
        })
    }
}
impl FromJson for HirStatic {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirStatic {
            name: FromJson::from_json(field(v, "name")?)?,
            ty: FromJson::from_json(field(v, "ty")?)?,
            value: FromJson::from_json(field(v, "value")?)?,
            is_mut: bool_field(v, "is_mut")?,
            is_pub: bool_field(v, "is_pub")?,
            is_external: bool_field(v, "is_external")?,
        })
    }
}

impl ToJson for HirFn {
    fn to_json(&self) -> Value {
        jobj!({
            "span": self.span.to_json(), "attrs": self.attrs.to_json(), "effects": self.effects.to_json(),
            "fn_id": self.fn_id.to_json(), "name": self.name.to_json(), "is_inline": self.is_inline.to_json(),
            "extern_c": self.extern_c.to_json(), "is_specialized": self.is_specialized.to_json(),
            "is_pub": self.is_pub.to_json(), "is_macro": self.is_macro.to_json(),
            "follow_sources": self.follow_sources.to_json(), "inferred": self.inferred.to_json(),
            "params": self.params.to_json(), "param_attrs": self.param_attrs.to_json(),
            "return_type": self.return_type.to_json(), "locals": self.locals.to_json(), "body": self.body.to_json(),
        })
    }
}
impl FromJson for HirFn {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(HirFn {
            span: FromJson::from_json(field(v, "span")?)?,
            attrs: Vec::from_json(field(v, "attrs")?)?,
            effects: FromJson::from_json(field(v, "effects")?)?,
            fn_id: FromJson::from_json(field(v, "fn_id")?)?,
            name: FromJson::from_json(field(v, "name")?)?,
            is_inline: bool_field(v, "is_inline")?,
            extern_c: bool_field(v, "extern_c")?,
            is_specialized: bool_field(v, "is_specialized")?,
            is_pub: bool_field(v, "is_pub")?,
            is_macro: bool_field(v, "is_macro")?,
            follow_sources: Vec::from_json(field(v, "follow_sources")?)?,
            inferred: FromJson::from_json(field(v, "inferred")?)?,
            params: Vec::from_json(field(v, "params")?)?,
            param_attrs: Vec::from_json(field(v, "param_attrs")?)?,
            return_type: FromJson::from_json(field(v, "return_type")?)?,
            locals: Vec::from_json(field(v, "locals")?)?,
            body: FromJson::from_json(field(v, "body")?)?,
        })
    }
}

impl ToJson for HirItem {
    fn to_json(&self) -> Value {
        match self {
            HirItem::Fn(f) => jobj!({"kind": "fn", "value": f}),
            HirItem::StructDef(d) => jobj!({"kind": "struct", "value": d}),
            HirItem::Namespace { name, items } => jobj!({"kind": "namespace", "name": name, "items": items}),
            HirItem::InterfaceDef { name, generic_params, methods } => jobj!({
                "kind": "interface", "name": name, "generic_params": generic_params, "methods": methods,
            }),
        }
    }
}
impl FromJson for HirItem {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match str_field(v, "kind")? {
            "fn" => HirItem::Fn(FromJson::from_json(field(v, "value")?)?),
            "struct" => HirItem::StructDef(FromJson::from_json(field(v, "value")?)?),
            "namespace" => HirItem::Namespace {
                name: FromJson::from_json(field(v, "name")?)?,
                items: Vec::from_json(field(v, "items")?)?,
            },
            "interface" => HirItem::InterfaceDef {
                name: FromJson::from_json(field(v, "name")?)?,
                generic_params: Vec::from_json(field(v, "generic_params")?)?,
                methods: Vec::from_json(field(v, "methods")?)?,
            },
            other => return Err(Error::Hir(format!("HIR schema: unknown item '{}'", other))),
        })
    }
}

pub fn encode_program(program: &HirProgram) -> Value {
    let mut struct_defs: Vec<_> = program.struct_defs.iter()
        .map(|(name, fields)| (name, fields)).collect();
    struct_defs.sort_by_key(|(name, _)| name.as_str());
    let mut generic_params: Vec<_> = program.generic_struct_params.iter()
        .map(|(name, params)| (name, params)).collect();
    generic_params.sort_by_key(|(name, _)| name.as_str());
    jobj!({
        "schema": HIR_SCHEMA_VERSION,
        "items": program.items,
        "vtables": program.vtables,
        "struct_defs": struct_defs,
        "generic_struct_params": generic_params,
        "imported_fns": program.imported_fns,
        "statics": program.statics,
        "consts": program.consts,
    })
}

pub fn decode_program(value: &Value) -> Result<HirProgram> {
    let schema = u64_field(value, "schema")?;
    if schema != HIR_SCHEMA_VERSION as u64 {
        return Err(Error::Hir(format!(
            "HIR schema: unsupported version {} (compiler supports {})",
            schema, HIR_SCHEMA_VERSION
        )));
    }
    let struct_defs: Vec<(crate::intern::Symbol, Vec<HirStructField>)> =
        Vec::from_json(field(value, "struct_defs")?)?;
    let generic_struct_params: Vec<(crate::intern::Symbol, Vec<(crate::intern::Symbol, Vec<crate::intern::Symbol>)>)> =
        Vec::from_json(field(value, "generic_struct_params")?)?;
    Ok(HirProgram {
        items: Vec::from_json(field(value, "items")?)?,
        vtables: Vec::from_json(field(value, "vtables")?)?,
        struct_defs: struct_defs.into_iter().collect(),
        generic_struct_params: generic_struct_params.into_iter().collect(),
        imported_fns: Vec::from_json(field(value, "imported_fns")?)?,
        statics: Vec::from_json(field(value, "statics")?)?,
        consts: Vec::from_json(field(value, "consts")?)?,
    })
}
