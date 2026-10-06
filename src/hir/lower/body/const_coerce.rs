use super::*;

/// 常量类型适配：按标注类型转换/检查字面量（含 IntN 范围检查）
pub(super) fn coerce_literal(lit_ty: &mut HirType, lit: &mut HirLiteral, want: &HirType, span: &Span) -> Result<()> {
    // M6.2c：数组常量（标量元素）——标注 `[T]` / `[T; N]`
    if let HirLiteral::Array(vals) = lit {
        let inner = match want {
            HirType::Unique(i) => i.as_ref(),
            other => other,
        };
        let (want_elem, want_n) = match inner {
            HirType::ArraySized(e, n) => (e.as_ref(), Some(*n)),
            HirType::Array(e) => (e.as_ref(), None),
            _ => {
                return Err(Error::Hir(format!(
                    "const type mismatch: expected {}, found array (at {}:{})",
                    hir_type_display(want), span.start_line, span.start_col
                )))
            }
        };
        if let Some(n) = want_n {
            if n != vals.len() {
                return Err(Error::Hir(format!(
                    "const array length mismatch: expected {}, found {} (at {}:{})",
                    n, vals.len(), span.start_line, span.start_col
                )));
            }
        }
        for v in vals.iter_mut() {
            let mut vt = match v {
                HirLiteral::Int(_) => HirType::Int,
                HirLiteral::Float(_) => HirType::Float,
                HirLiteral::Char(_) => HirType::Char,
                HirLiteral::Bool(_) => HirType::Bool,
                HirLiteral::String(_) | HirLiteral::Array(_) => {
                    return Err(Error::Hir(format!(
                        "nested array constants are not supported yet (at {}:{})",
                        span.start_line, span.start_col
                    )))
                }
            };
            coerce_literal(&mut vt, v, want_elem, span)?;
        }
        *lit_ty = HirType::Unique(Box::new(HirType::ArraySized(
            Box::new(want_elem.clone()), vals.len())));
        return Ok(());
    }
    match (want, &*lit_ty, &*lit) {
        (HirType::Int, HirType::Int, HirLiteral::Int(_)) => {}
        (HirType::Float, HirType::Int, HirLiteral::Int(v)) => {
            *lit = HirLiteral::Float(*v as f64);
            *lit_ty = HirType::Float;
        }
        (HirType::Float, HirType::Float, HirLiteral::Float(_)) => {}
        (HirType::F32, HirType::Int, HirLiteral::Int(v)) => {
            *lit = HirLiteral::Float(*v as f64);
            *lit_ty = HirType::F32;
        }
        (HirType::F32, HirType::Float, HirLiteral::Float(_)) => {
            *lit_ty = HirType::F32;
        }
        (HirType::IntN { bits, signed }, HirType::Int, HirLiteral::Int(v)) => {
            check_intn_range(*v, *bits, *signed, want, span)?;
            *lit_ty = want.clone();
        }
        (HirType::Char, HirType::Char, HirLiteral::Char(_)) => {}
        (HirType::Bool, HirType::Bool, HirLiteral::Bool(_)) => {}
        _ => {
            return Err(Error::Hir(format!(
                "const type mismatch: expected {}, found {} (at {}:{})",
                hir_type_display(want), hir_type_display(lit_ty), span.start_line, span.start_col
            )))
        }
    }
    Ok(())
}

/// 定宽整数范围检查（字面量以 i64 存储）
pub(super) fn check_intn_range(v: i64, bits: u8, signed: bool, want: &HirType, span: &Span) -> Result<()> {
    let ok = if signed {
        let min = -(1i128 << (bits - 1));
        let max = (1i128 << (bits - 1)) - 1;
        (v as i128) >= min && (v as i128) <= max
    } else {
        let max = if bits >= 64 { u64::MAX as u128 } else { (1u128 << bits) - 1 };
        v >= 0 && (v as u128) <= max
    };
    if !ok {
        return Err(Error::Hir(format!(
            "literal out of range for `{}` (at {}:{})",
            hir_type_display(want), span.start_line, span.start_col
        )));
    }
    Ok(())
}
