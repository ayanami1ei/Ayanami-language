//! HIR 表达式节点的 schema 编解码。

use serde_json::{Map, Value};

use crate::error::Result;
use crate::hir::node::{HirNodeBox, HirSchemaNode};
use crate::hir::*;

use super::core::{field, str_field, FromJson, ToJson};

macro_rules! node {
    ($ty:ty, $tag:literal, { $($field:ident),* $(,)? }) => {
        impl HirSchemaNode for $ty {
            fn schema_json(&self) -> Value {
                let mut map = Map::new();
                map.insert("kind".to_string(), Value::String($tag.to_string()));
                $(map.insert(stringify!($field).to_string(), self.$field.to_json());)*
                Value::Object(map)
            }
        }
    };
}

node!(SBin, "bin", { op, lhs, rhs, ty });
node!(SUn, "un", { op, arg, ty });
node!(SCall, "call", { fn_id, args, ty });
node!(SConst, "const", { val, ty });
node!(SFileArg, "file_arg", { path, ty });
node!(SBoolToNum, "bool_to_num", { expr, ty });
node!(SGlobal, "global", { name, ty, mutable });
node!(SVar, "var", { var, ty });
node!(SMove, "move", { expr, ty });
node!(SClone, "clone", { expr, ty });
node!(SToUnique, "to_unique", { expr, ty });
node!(SCast, "cast", { expr, ty });
node!(SReinterpret, "reinterpret", { expr, ty });
node!(SPtrOffset, "ptr_offset", { ptr, index, ty });
node!(SVolatileLoad, "volatile_load", { ptr, ty });
node!(SVolatileStore, "volatile_store", { ptr, value, ty });
node!(SField, "field", { object, field, field_index, ty });
node!(SStruct, "struct", { type_name, fields, ty });
node!(SArrLit, "array", { elems, ty });
node!(SArrSz, "array_sized", { count, elem_ty, ty });
node!(SAsm, "asm", { template, outputs, inputs, clobbers, ty });
node!(SRef, "ref", { expr, mutable, ty });
node!(SDeref, "deref", { expr, ty });
node!(SIdx, "index", { object, index, ty });
node!(SVCall, "vcall", { receiver, interface, method_index, args, concrete_type, ty });
node!(SMFP, "mfp", { value, concrete_type, interface_name, ty });
node!(SEnumC, "enum_construct", { enum_name, variant_name, variant_struct, args, ty });
node!(SEnumM, "enum_match", { value, arms, ty });
node!(SFnPtr, "fn_ptr", { fn_id, ty });
node!(SCallP, "call_ptr", { fn_ptr, args, ty });

impl ToJson for HirNodeBox {
    fn to_json(&self) -> Value { self.0.schema_json() }
}

impl FromJson for HirNodeBox {
    fn from_json(value: &Value) -> Result<Self> {
        let kind = str_field(value, "kind")?;
        let node = match kind {
            "bin" => SBin {
                op: FromJson::from_json(field(value, "op")?)?,
                lhs: FromJson::from_json(field(value, "lhs")?)?,
                rhs: FromJson::from_json(field(value, "rhs")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "un" => SUn {
                op: FromJson::from_json(field(value, "op")?)?,
                arg: FromJson::from_json(field(value, "arg")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "call" => SCall {
                fn_id: FromJson::from_json(field(value, "fn_id")?)?,
                args: FromJson::from_json(field(value, "args")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "const" => SConst {
                val: FromJson::from_json(field(value, "val")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "file_arg" => SFileArg {
                path: FromJson::from_json(field(value, "path")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "bool_to_num" => SBoolToNum {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "global" => SGlobal {
                name: FromJson::from_json(field(value, "name")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
                mutable: FromJson::from_json(field(value, "mutable")?)?,
            }.into(),
            "var" => SVar {
                var: FromJson::from_json(field(value, "var")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "move" => SMove {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "clone" => SClone {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "to_unique" => SToUnique {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "cast" => SCast {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "reinterpret" => SReinterpret {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "ptr_offset" => SPtrOffset {
                ptr: FromJson::from_json(field(value, "ptr")?)?,
                index: FromJson::from_json(field(value, "index")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "volatile_load" => SVolatileLoad {
                ptr: FromJson::from_json(field(value, "ptr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "volatile_store" => SVolatileStore {
                ptr: FromJson::from_json(field(value, "ptr")?)?,
                value: FromJson::from_json(field(value, "value")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "field" => SField {
                object: FromJson::from_json(field(value, "object")?)?,
                field: FromJson::from_json(field(value, "field")?)?,
                field_index: FromJson::from_json(field(value, "field_index")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "struct" => SStruct {
                type_name: FromJson::from_json(field(value, "type_name")?)?,
                fields: FromJson::from_json(field(value, "fields")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "array" => SArrLit {
                elems: FromJson::from_json(field(value, "elems")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "array_sized" => SArrSz {
                count: FromJson::from_json(field(value, "count")?)?,
                elem_ty: FromJson::from_json(field(value, "elem_ty")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "asm" => SAsm {
                template: FromJson::from_json(field(value, "template")?)?,
                outputs: FromJson::from_json(field(value, "outputs")?)?,
                inputs: FromJson::from_json(field(value, "inputs")?)?,
                clobbers: FromJson::from_json(field(value, "clobbers")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "ref" => SRef {
                expr: FromJson::from_json(field(value, "expr")?)?,
                mutable: FromJson::from_json(field(value, "mutable")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "deref" => SDeref {
                expr: FromJson::from_json(field(value, "expr")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "index" => SIdx {
                object: FromJson::from_json(field(value, "object")?)?,
                index: FromJson::from_json(field(value, "index")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "vcall" => SVCall {
                receiver: FromJson::from_json(field(value, "receiver")?)?,
                interface: FromJson::from_json(field(value, "interface")?)?,
                method_index: FromJson::from_json(field(value, "method_index")?)?,
                args: FromJson::from_json(field(value, "args")?)?,
                concrete_type: FromJson::from_json(field(value, "concrete_type")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "mfp" => SMFP {
                value: FromJson::from_json(field(value, "value")?)?,
                concrete_type: FromJson::from_json(field(value, "concrete_type")?)?,
                interface_name: FromJson::from_json(field(value, "interface_name")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "enum_construct" => SEnumC {
                enum_name: FromJson::from_json(field(value, "enum_name")?)?,
                variant_name: FromJson::from_json(field(value, "variant_name")?)?,
                variant_struct: FromJson::from_json(field(value, "variant_struct")?)?,
                args: FromJson::from_json(field(value, "args")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "enum_match" => SEnumM {
                value: FromJson::from_json(field(value, "value")?)?,
                arms: FromJson::from_json(field(value, "arms")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "fn_ptr" => SFnPtr {
                fn_id: FromJson::from_json(field(value, "fn_id")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            "call_ptr" => SCallP {
                fn_ptr: FromJson::from_json(field(value, "fn_ptr")?)?,
                args: FromJson::from_json(field(value, "args")?)?,
                ty: FromJson::from_json(field(value, "ty")?)?,
            }.into(),
            other => {
                return Err(crate::error::Error::Hir(format!(
                    "HIR schema: unknown expression node '{}'",
                    other
                )))
            }
        };
        Ok(node)
    }
}
