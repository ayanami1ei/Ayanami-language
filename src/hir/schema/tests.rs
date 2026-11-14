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
