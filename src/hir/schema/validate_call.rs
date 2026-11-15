//! SCall 签名校验（从 validate.rs 拆出，保持文件 ≤300 行）。

use serde_json::Value;
use std::collections::HashMap;

use crate::error::Result;

use super::validate::FnSig;

pub(super) fn validate_call(
    call: &serde_json::Map<String, Value>,
    id: u64,
    sigs: &HashMap<u64, FnSig>,
    path: &str,
) -> Result<()> {
    let Some(sig) = sigs.get(&id) else { return Ok(()) };
    if sig.extern_c {
        return Ok(());
    }
    let empty = Vec::new();
    let args = call.get("args").and_then(Value::as_array).unwrap_or(&empty);
    if args.len() != sig.params.len() {
        return Err(err(format!(
            "{}: call FnId {} has {} arg(s), signature takes {}",
            path, id, args.len(), sig.params.len()
        )));
    }
    for (i, arg) in args.iter().enumerate() {
        let Some(arg_ty) = arg.get("ty") else { continue };
        if !types_compatible(arg_ty, &sig.params[i]) {
            return Err(err(format!(
                "{}: call FnId {} arg {} type mismatch (arg {}, param {})",
                path, id, i, arg_ty, sig.params[i]
            )));
        }
    }
    if let Some(call_ty) = call.get("ty") {
        let is_never = sig.ret.get("kind").and_then(Value::as_str) == Some("never");
        if !is_never && !types_compatible(call_ty, &sig.ret) {
            return Err(err(format!(
                "{}: call FnId {} return type mismatch (node {}, function {})",
                path, id, call_ty, sig.ret
            )));
        }
    }
    Ok(())
}

fn types_compatible(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    if actual.get("kind").and_then(Value::as_str) == Some("never") {
        return true;
    }
    if actual.get("kind").and_then(Value::as_str) == Some("ref")
        && expected.get("kind").and_then(Value::as_str) == Some("ref")
        && actual.get("inner") == expected.get("inner")
    {
        return true;
    }
    false
}

fn err(message: impl std::fmt::Display) -> crate::error::Error {
    crate::error::Error::Hir(format!("transform HIR: {}", message))
}
