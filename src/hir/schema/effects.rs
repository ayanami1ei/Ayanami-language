//! HIR 效应声明 schema 编解码。

use serde_json::{json, Value};
use std::collections::BTreeSet;
use crate::error::Result;
use crate::hir::effects::infer::EffectSet;
use crate::hir::effects::{EffectDecl, ThrowsDecl};
use super::core::{bool_field, err, field, str_field, FromJson, ToJson};

impl ToJson for EffectDecl {
    fn to_json(&self) -> Value {
        json!({
            "effects": self.effects.to_json(),
            "pure": self.pure,
            "no_error": self.no_error,
            "throws": self.throws.to_json(),
        })
    }
}
impl FromJson for EffectDecl {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(EffectDecl {
            effects: Vec::from_json(field(v, "effects")?)?,
            pure: bool_field(v, "pure")?,
            no_error: bool_field(v, "no_error")?,
            throws: Option::from_json(field(v, "throws")?)?,
        })
    }
}
impl ToJson for ThrowsDecl {
    fn to_json(&self) -> Value {
        match self {
            ThrowsDecl::OpenEmpty => json!({"kind": "open"}),
            ThrowsDecl::Unknown => json!({"kind": "unknown"}),
            ThrowsDecl::Types(types) => json!({"kind": "types", "types": types.to_json()}),
        }
    }
}
impl FromJson for ThrowsDecl {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(match str_field(v, "kind")? {
            "open" => ThrowsDecl::OpenEmpty,
            "unknown" => ThrowsDecl::Unknown,
            "types" => ThrowsDecl::Types(Vec::from_json(field(v, "types")?)?),
            other => return Err(err(format!("unknown throws decl '{}'", other))),
        })
    }
}
impl ToJson for EffectSet {
    fn to_json(&self) -> Value {
        json!({
            "throws": self.throws.iter().cloned().collect::<Vec<_>>(),
            "effects": self.effects.iter().cloned().collect::<Vec<_>>(),
            "may_unknown_effects": self.may_unknown_effects,
            "may_unknown_errors": self.may_unknown_errors,
        })
    }
}
impl FromJson for EffectSet {
    fn from_json(v: &Value) -> Result<Self> {
        let to_set = |key: &str| -> Result<BTreeSet<String>> {
            Ok(Vec::<String>::from_json(field(v, key)?)?.into_iter().collect())
        };
        Ok(EffectSet {
            throws: to_set("throws")?,
            effects: to_set("effects")?,
            may_unknown_effects: bool_field(v, "may_unknown_effects")?,
            may_unknown_errors: bool_field(v, "may_unknown_errors")?,
        })
    }
}
