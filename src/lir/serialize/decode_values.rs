//! 字面量/值解码（从 decode.rs 拆出，保持文件 ≤300 行）。

use super::*;

impl<'a> Reader<'a> {
    pub(super) fn literal(&mut self) -> Result<HirLiteral> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(HirLiteral::Int(self.u64()? as i64)),
            1 => { let b = self.read(8)?; Ok(HirLiteral::Float(f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))) }
            2 => Ok(HirLiteral::Char(char::from_u32(self.u32()?).unwrap_or('\0'))),
            3 => Ok(HirLiteral::String(self.str()?)),
            4 => Ok(HirLiteral::Bool(self.read(1)?[0] != 0)),
            5 => {
                let n = self.u32()? as usize;
                let mut v = Vec::with_capacity(n);
                for _ in 0..n { v.push(self.literal()?); }
                Ok(HirLiteral::Array(v))
            }
            6 => {
                let n = self.u32()? as usize;
                let mut v = Vec::with_capacity(n);
                for _ in 0..n { v.push(self.literal()?); }
                Ok(HirLiteral::Struct(v))
            }
            _ => Err(Error::Serialize(format!("unknown literal tag: {}", tag))),
        }
    }
    pub(super) fn value(&mut self) -> Result<LirValue> {
        let tag = self.read(1)?[0];
        match tag {
            0 => Ok(LirValue::Var(VarId(self.u32()? as usize))),
            1 => Ok(LirValue::Tmp(self.u64()?)),
            2 => Ok(LirValue::Param(self.u64()?)),
            3 => { let l = self.literal()?; let t = self.ty()?; Ok(LirValue::Literal(l, t)) }
            _ => Err(Error::Serialize(format!("unknown value tag: {}", tag))),
        }
    }
}
