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

pub(super) fn types_compatible(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    let ak = actual.get("kind").and_then(Value::as_str);
    let ek = expected.get("kind").and_then(Value::as_str);
    if ak == Some("never") {
        return true;
    }
    let scalar = |k: Option<&str>| matches!(k, Some("int"|"bool"|"char"|"float"|"f32"|"int_n"));
    if scalar(ak) && scalar(ek) {
        return true;
    }
    match (ak, ek) {
        (Some("ref"), Some("ref")) => {
            types_compatible(actual.get("inner").unwrap_or(&Value::Null), expected.get("inner").unwrap_or(&Value::Null))
        }
        (Some("unique"), Some("unique")) => {
            let ai = actual.get("inner").unwrap_or(&Value::Null);
            let ei = expected.get("inner").unwrap_or(&Value::Null);
            let ak = ai.get("kind").and_then(Value::as_str);
            let ek = ei.get("kind").and_then(Value::as_str);
            if matches!(ak, Some("array") | Some("array_sized"))
                && matches!(ek, Some("array") | Some("array_sized"))
            {
                true
            } else {
                types_compatible(ai, ei)
            }
        }
        (Some("array"), Some("array"))
        | (Some("array_sized"), Some("array_sized"))
        | (Some("raw_ptr"), Some("raw_ptr")) => {
            types_compatible(actual.get("inner").unwrap_or(&Value::Null), expected.get("inner").unwrap_or(&Value::Null))
        }
        // HIR 会用 array_sized 表示具体尺寸；调用边界允许与 array 互操作。
        (Some("array_sized"), Some("array")) | (Some("array"), Some("array_sized")) => {
            types_compatible(actual.get("inner").unwrap_or(&Value::Null), expected.get("inner").unwrap_or(&Value::Null))
        }
        // unique<T> 在 HIR 调用边界可能对应 array/array_sized<T>；比较 T 与容器元素层。
        (Some("unique"), Some("array")) | (Some("unique"), Some("array_sized")) => {
            types_compatible(actual.get("inner").unwrap_or(&Value::Null), expected)
        }
        (Some("array"), Some("unique")) | (Some("array_sized"), Some("unique")) => {
            types_compatible(actual, expected.get("inner").unwrap_or(&Value::Null))
        }
        (Some("closure"), Some("closure")) => {
            let ap = actual.get("params").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
            let ep = expected.get("params").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
            ap.len() == ep.len()
                && ap.iter().zip(ep).all(|(a, b)| types_compatible(a, b))
                && types_compatible(actual.get("return").unwrap_or(&Value::Null), expected.get("return").unwrap_or(&Value::Null))
        }
        (Some("named"), Some("named")) => {
            let an = actual.get("name").and_then(Value::as_str).unwrap_or("");
            let en = expected.get("name").and_then(Value::as_str).unwrap_or("");
            an == en || named_base(an) == named_base(en)
        }
        _ => false,
    }
}

fn named_base(name: &str) -> &str {
    name.split(['<', '[']).next().unwrap_or(name)
}

fn err(message: impl std::fmt::Display) -> crate::error::Error {
    crate::error::Error::Hir(format!("transform HIR: {}", message))
}

#[cfg(test)]
mod tests {
    use super::types_compatible;

    #[test]
    fn unique_sized_array_matches_array() {
        let actual = serde_json::json!({
            "kind": "unique",
            "inner": {"kind": "array_sized", "size": 3, "inner": {"kind": "char"}}
        });
        let expected = serde_json::json!({
            "kind": "unique",
            "inner": {"kind": "array", "inner": {"kind": "char"}}
        });
        assert!(types_compatible(&actual, &expected));
    }

    #[test]
    fn unique_sized_array_matches_plain_array() {
        let actual = serde_json::json!({
            "kind": "unique",
            "inner": {"kind": "array_sized", "size": 3, "inner": {"kind": "char"}}
        });
        let expected = serde_json::json!({
            "kind": "array",
            "inner": {"kind": "char"}
        });
        assert!(types_compatible(&actual, &expected));
    }
}
