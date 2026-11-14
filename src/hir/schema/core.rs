//! HIR schema 的基础类型编解码。

use serde_json::{json, Map, Value};

use crate::error::{Error, Result};
use crate::hir::ty::{FnId, VarId};
use crate::intern::Symbol;
use crate::span::Span;

pub trait ToJson {
    fn to_json(&self) -> Value;
}

pub trait FromJson: Sized {
    fn from_json(value: &Value) -> Result<Self>;
}

pub(crate) fn err(message: impl std::fmt::Display) -> Error {
    Error::Hir(format!("HIR schema: {}", message))
}

pub(crate) fn object<'a>(value: &'a Value, ctx: &str) -> Result<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| err(format!("{} must be an object", ctx)))
}

pub(crate) fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    object(value, "object")?
        .get(key)
        .ok_or_else(|| err(format!("missing field '{}'", key)))
}

pub(crate) fn str_field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    field(value, key)?
        .as_str()
        .ok_or_else(|| err(format!("field '{}' must be a string", key)))
}

pub(crate) fn u64_field(value: &Value, key: &str) -> Result<u64> {
    field(value, key)?.as_u64().ok_or_else(|| err(format!("field '{}' must be an integer", key)))
}

pub(crate) fn bool_field(value: &Value, key: &str) -> Result<bool> {
    field(value, key)?.as_bool().ok_or_else(|| err(format!("field '{}' must be a boolean", key)))
}

pub(crate) fn array<'a>(value: &'a Value, ctx: &str) -> Result<&'a Vec<Value>> {
    value.as_array().ok_or_else(|| err(format!("{} must be an array", ctx)))
}

impl ToJson for bool {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for usize {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for u64 {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for i64 {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for String {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for str {
    fn to_json(&self) -> Value { json!(self) }
}
impl<T: ToJson + ?Sized> ToJson for &T {
    fn to_json(&self) -> Value { (*self).to_json() }
}
impl ToJson for u32 {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for i32 {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for f64 {
    fn to_json(&self) -> Value { json!(self) }
}
impl ToJson for Value {
    fn to_json(&self) -> Value { self.clone() }
}
impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> Value {
        Value::Array(self.iter().map(ToJson::to_json).collect())
    }
}
impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Value {
        match self { Some(v) => v.to_json(), None => Value::Null }
    }
}
impl<A: ToJson, B: ToJson> ToJson for (A, B) {
    fn to_json(&self) -> Value { Value::Array(vec![self.0.to_json(), self.1.to_json()]) }
}
impl<A: ToJson, B: ToJson, C: ToJson> ToJson for (A, B, C) {
    fn to_json(&self) -> Value {
        Value::Array(vec![self.0.to_json(), self.1.to_json(), self.2.to_json()])
    }
}

impl FromJson for bool {
    fn from_json(v: &Value) -> Result<Self> { v.as_bool().ok_or_else(|| err("expected bool")) }
}
impl FromJson for usize {
    fn from_json(v: &Value) -> Result<Self> {
        v.as_u64().map(|n| n as usize).ok_or_else(|| err("expected usize"))
    }
}
impl FromJson for u64 {
    fn from_json(v: &Value) -> Result<Self> { v.as_u64().ok_or_else(|| err("expected u64")) }
}
impl FromJson for i64 {
    fn from_json(v: &Value) -> Result<Self> { v.as_i64().ok_or_else(|| err("expected i64")) }
}
impl FromJson for String {
    fn from_json(v: &Value) -> Result<Self> { v.as_str().map(str::to_string).ok_or_else(|| err("expected string")) }
}
impl<T: FromJson> FromJson for Vec<T> {
    fn from_json(v: &Value) -> Result<Self> {
        array(v, "array")?.iter().map(T::from_json).collect()
    }
}
impl<T: FromJson> FromJson for Option<T> {
    fn from_json(v: &Value) -> Result<Self> {
        if v.is_null() { Ok(None) } else { Ok(Some(T::from_json(v)?)) }
    }
}
impl<A: FromJson, B: FromJson> FromJson for (A, B) {
    fn from_json(v: &Value) -> Result<Self> {
        let a = array(v, "tuple")?;
        if a.len() != 2 { return Err(err("expected 2-tuple")); }
        Ok((A::from_json(&a[0])?, B::from_json(&a[1])?))
    }
}
impl<A: FromJson, B: FromJson, C: FromJson> FromJson for (A, B, C) {
    fn from_json(v: &Value) -> Result<Self> {
        let a = array(v, "tuple")?;
        if a.len() != 3 { return Err(err("expected 3-tuple")); }
        Ok((A::from_json(&a[0])?, B::from_json(&a[1])?, C::from_json(&a[2])?))
    }
}

impl ToJson for Symbol {
    fn to_json(&self) -> Value { json!(self.as_str()) }
}
impl FromJson for Symbol {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(Symbol::intern(v.as_str().ok_or_else(|| err("expected symbol string"))?))
    }
}
impl ToJson for FnId {
    fn to_json(&self) -> Value { json!(self.0) }
}
impl FromJson for FnId {
    fn from_json(v: &Value) -> Result<Self> { Ok(FnId(usize::from_json(v)?)) }
}
impl ToJson for VarId {
    fn to_json(&self) -> Value { json!(self.0) }
}
impl FromJson for VarId {
    fn from_json(v: &Value) -> Result<Self> { Ok(VarId(usize::from_json(v)?)) }
}

impl ToJson for Span {
    fn to_json(&self) -> Value {
        json!({
            "start_line": self.start_line,
            "start_col": self.start_col,
            "end_line": self.end_line,
            "end_col": self.end_col,
            "start_byte": self.start_byte,
            "end_byte": self.end_byte,
        })
    }
}
impl FromJson for Span {
    fn from_json(v: &Value) -> Result<Self> {
        Ok(Span {
            start_line: usize::from_json(field(v, "start_line")?)?,
            start_col: usize::from_json(field(v, "start_col")?)?,
            end_line: usize::from_json(field(v, "end_line")?)?,
            end_col: usize::from_json(field(v, "end_col")?)?,
            start_byte: usize::from_json(field(v, "start_byte")?)?,
            end_byte: usize::from_json(field(v, "end_byte")?)?,
        })
    }
}
