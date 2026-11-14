//! A5e/B0：第三方 `#[transform]` 的 HIR JSON schema（版本 1）。
//!
//! 编码目标不是 Rust 内存布局，而是稳定、可跨插件传输的 JSON：
//! - symbol 用字符串，解码时重新驻留；
//! - `FnId` / `VarId` 用整数，插件可保持或重新编号；
//! - attrs 以源码文本传输，解码时重新解析；
//! - schema 版本写在顶层，插件与编译器必须一致。

mod core;
mod effects;
mod nodes;
mod program;
mod stmt;
mod syntax;
mod types;
mod validate;

pub use core::{FromJson, ToJson};
pub use program::{decode_program, encode_program};

use crate::error::{Error, Result};
use crate::hir::HirProgram;

/// HIR JSON schema 版本；任何字段/节点语义变化都必须递增。
pub const HIR_SCHEMA_VERSION: u32 = 1;

/// HIR → JSON 文本。
pub fn to_json(program: &HirProgram) -> Result<String> {
    serde_json::to_string(&encode_program(program))
        .map_err(|e| Error::Hir(format!("HIR schema encode: {}", e)))
}

/// JSON 文本 → HIR；版本或结构不匹配时返回明确诊断。
pub fn from_json(text: &str) -> Result<HirProgram> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| Error::Hir(format!("HIR schema decode: invalid JSON: {}", e)))?;
    validate::validate(&value)?;
    decode_program(&value)
}

/// identity roundtrip：编码 → 解码 → 再编码，确保 schema 稳定。
pub fn roundtrip(program: &HirProgram) -> Result<HirProgram> {
    let first = to_json(program)?;
    let decoded = from_json(&first)?;
    let second = to_json(&decoded)?;
    if first != second {
        return Err(Error::Hir(
            "HIR schema roundtrip is not stable (encode→decode→encode differs)".into(),
        ));
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests;
