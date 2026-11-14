use super::{from_json, roundtrip, to_json};
use crate::compiler::CompilerPipeline;

fn hir_for(source: &str) -> crate::hir::HirProgram {
    let mut pipeline = CompilerPipeline::new(source);
    pipeline
        .lex()
        .expect("lex")
        .parse()
        .expect("parse")
        .lower_hir()
        .expect("hir");
    pipeline.hir_program().expect("hir").clone()
}

#[test]
fn identity_roundtrip_basic_program() {
    let source = r#"
struct Point {
    int x
    int y
}
enum Maybe { Yes(int), No }
#[pure]
fn add(int a, int b) -> int { return a + b }
fn main() -> int {
    p = Point { x = 1, y = 2 }
    m = Maybe::Yes(3)
    if add(p.x, p.y) == 3 {
        return 0
    }
    return 1
}
"#;
    let hir = hir_for(source);
    let encoded = to_json(&hir).expect("encode");
    let decoded = from_json(&encoded).expect("decode");
    let reencoded = to_json(&decoded).expect("reencode");
    assert_eq!(encoded, reencoded);
    let again = roundtrip(&hir).expect("roundtrip");
    assert_eq!(reencoded, to_json(&again).expect("third"));
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let hir = hir_for("fn main() -> int { return 0 }");
    let json = to_json(&hir).expect("encode");
    let json = json.replace("\"schema\":1", "\"schema\":2");
    assert!(from_json(&json).is_err());
}

fn find_mut<'a>(value: &'a mut serde_json::Value, kind: &str) -> Option<&'a mut serde_json::Value> {
    if value.get("kind").and_then(serde_json::Value::as_str) == Some(kind) {
        return Some(value);
    }
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                if let Some(found) = find_mut(item, kind) {
                    return Some(found);
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            for child in map.values_mut() {
                if let Some(found) = find_mut(child, kind) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

#[test]
fn invalid_fn_id_is_rejected_before_decode() {
    let source = "fn add(int a, int b) -> int { return a + b }\nfn main() -> int { return add(1, 2) }";
    let hir = hir_for(source);
    let mut value: serde_json::Value = serde_json::from_str(&to_json(&hir).unwrap()).unwrap();
    let call = find_mut(&mut value, "call").expect("call node");
    call.as_object_mut().unwrap().insert("fn_id".into(), serde_json::json!(999999));
    assert!(from_json(&value.to_string()).is_err());
}

#[test]
fn invalid_var_id_is_rejected_before_decode() {
    let source = "fn add(int a, int b) -> int { return a + b }\nfn main() -> int { return add(1, 2) }";
    let hir = hir_for(source);
    let mut value: serde_json::Value = serde_json::from_str(&to_json(&hir).unwrap()).unwrap();
    let var = find_mut(&mut value, "var").expect("var node");
    var.as_object_mut().unwrap().insert("var".into(), serde_json::json!(999999));
    assert!(from_json(&value.to_string()).is_err());
}
