//! M6.1b：常量值的 .lcl 文本编解码。
//!
//! 编码为 `(类型串, 值串)`：Int/IntN 十进制、Float 最短往返、Char 码点、
//! Bool `true|false`。不引入引号/转义（字符串常量见 M6.3b）。
use crate::hir::ir::{HirLiteral, HirType};
use crate::intern::Symbol;

/// HirType → 类型串（仅标量；不支持的返回 None）
pub fn type_str(ty: &HirType) -> Option<String> {
    Some(match ty {
        HirType::Int => "int".into(),
        HirType::Float => "float".into(),
        HirType::F32 => "f32".into(),
        HirType::Char => "char".into(),
        HirType::Bool => "bool".into(),
        HirType::IntN { bits, signed } => {
            format!("{}{}", if *signed { "i" } else { "u" }, bits)
        }
        // M6.2c：数组类型（标量元素）编码 `[T;N]` / `[T]`
        HirType::Unique(inner) => type_str(inner)?,
        HirType::ArraySized(elem, n) => format!("[{};{}]", type_str(elem)?, n),
        HirType::Array(elem) => format!("[{}]", type_str(elem)?),
        // M6.2c：结构体（命名类型）——导入侧按 .lcl struct_defs 解析布局
        HirType::Named(n) => n.as_str(),
        _ => return None,
    })
}

/// 类型串 → HirType（仅标量）
pub fn type_from_str(s: &str) -> Option<HirType> {
    // M6.2c：数组类型
    if let Some(rest) = s.strip_prefix('[') {
        let inner = rest.strip_suffix(']')?;
        if let Some((e, n)) = inner.split_once(';') {
            let elem = type_from_str(e)?;
            let n: usize = n.parse().ok()?;
            return Some(HirType::Unique(Box::new(HirType::ArraySized(Box::new(elem), n))));
        }
        let elem = type_from_str(inner)?;
        return Some(HirType::Unique(Box::new(HirType::Array(Box::new(elem)))));
    }
    Some(match s {
        "int" => HirType::Int,
        "float" => HirType::Float,
        "f32" => HirType::F32,
        "char" => HirType::Char,
        "bool" => HirType::Bool,
        "i8" => HirType::IntN { bits: 8, signed: true },
        "i16" => HirType::IntN { bits: 16, signed: true },
        "i32" => HirType::IntN { bits: 32, signed: true },
        "i64" => HirType::IntN { bits: 64, signed: true },
        "i128" => HirType::IntN { bits: 128, signed: true },
        "u8" => HirType::IntN { bits: 8, signed: false },
        "u16" => HirType::IntN { bits: 16, signed: false },
        "u32" => HirType::IntN { bits: 32, signed: false },
        "u64" => HirType::IntN { bits: 64, signed: false },
        "u128" => HirType::IntN { bits: 128, signed: false },
        // M6.2c：命名类型（结构体 static；布局随 .lcl struct_defs 导入）
        _ => HirType::Named(Symbol::intern(s)),
    })
}

/// 值 → 文本
pub fn lit_str(lit: &HirLiteral) -> Option<String> {
    Some(match lit {
        HirLiteral::Int(i) => i.to_string(),
        HirLiteral::Float(f) => f.to_string(),
        HirLiteral::Char(c) => (*c as u32).to_string(),
        HirLiteral::Bool(b) => b.to_string(),
        HirLiteral::String(_) | HirLiteral::Array(_) | HirLiteral::Struct(_) => return None,
    })
}

/// 文本 → 值（按类型解析）
pub fn lit_from_str(ty: &HirType, s: &str) -> Option<HirLiteral> {
    Some(match ty {
        HirType::Int | HirType::IntN { .. } => HirLiteral::Int(s.parse().ok()?),
        HirType::Float | HirType::F32 => HirLiteral::Float(s.parse().ok()?),
        HirType::Char => HirLiteral::Char(char::from_u32(s.parse().ok()?)?),
        HirType::Bool => HirLiteral::Bool(s == "true"),
        _ => return None,
    })
}

/// 编码 `(类型, 值)` → 两个文本字段
pub fn encode(ty: &HirType, lit: &HirLiteral) -> Option<(String, String)> {
    Some((type_str(ty)?, lit_str(lit)?))
}

/// 解码类型串 + 值串 → `(HirType, HirLiteral)`
pub fn decode(ty: &str, value: &str) -> Option<(HirType, HirLiteral)> {
    let t = type_from_str(ty)?;
    let l = lit_from_str(&t, value)?;
    Some((t, l))
}
