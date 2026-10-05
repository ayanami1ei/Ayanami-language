use super::*;

impl crate::hir::lower::Ctx {
    /// M1.5：带后缀字面量（`1u8` / `1.5f32`），含溢出范围检查
    pub(crate) fn lower_suffixed(&mut self, lit: &Literal, suffix: &Symbol, span: &Span) -> Result<HirNodeBox> {
        let name = suffix.as_str();
        let ty = match fixed_width_type(&name) {
            Some(t) => t,
            None => return Err(Error::Hir(format!(
                "unknown literal suffix `{}` (at {}:{})", name, span.start_line, span.start_col
            ))),
        };
        match lit {
            Literal::Int(n, _) => {
                if matches!(ty, HirType::Float | HirType::F32) {
                    return Ok(SConst { val: HirLiteral::Float(*n as f64), ty }.into());
                }
                check_literal_range(*n, &ty, span)?;
                Ok(SConst { val: HirLiteral::Int(*n), ty }.into())
            }
            Literal::Float(f, _) => {
                if !matches!(ty, HirType::Float | HirType::F32) {
                    return Err(Error::Hir(format!(
                        "invalid suffix `{}` for floating-point literal (at {}:{})",
                        name, span.start_line, span.start_col
                    )));
                }
                Ok(SConst { val: HirLiteral::Float(*f), ty }.into())
            }
            _ => Ok(self.lower_literal(lit)?),
        }
    }

    pub(crate) fn lower_literal(&mut self, lit: &Literal) -> Result<HirNodeBox> {
        match lit {
            Literal::Int(n, _) => Ok(SConst { val: HirLiteral::Int(*n), ty: HirType::Int }.into()),
            Literal::Float(n, _) => Ok(SConst { val: HirLiteral::Float(*n), ty: HirType::Float }.into()),
            Literal::Char(c, _) => Ok(SConst { val: HirLiteral::Char(*c), ty: HirType::Char }.into()),
            Literal::String(s, _) => Ok(SConst { val: HirLiteral::String(s.clone()), ty: HirType::Named(Symbol::intern("String")) }.into()),
            Literal::Bool(b, _) => Ok(SConst { val: HirLiteral::Bool(*b), ty: HirType::Bool }.into()),
        }
    }
}

/// M1.5：后缀整数常量范围检查（超出目标类型 → 报错，Rust 风格）
fn check_literal_range(n: i64, ty: &HirType, span: &Span) -> Result<()> {
    let HirType::IntN { bits, signed } = ty else { return Ok(()); };
    let bits = *bits as u32;
    let ok = if *signed {
        if bits >= 64 { true } else {
            let max = (1i64 << (bits - 1)) - 1;
            let min = -(1i64 << (bits - 1));
            n >= min && n <= max
        }
    } else if bits >= 64 {
        n >= 0
    } else {
        n >= 0 && n < (1i64 << bits)
    };
    if !ok {
        return Err(Error::Hir(format!(
            "literal out of range for `{}` (at {}:{})",
            intn_name(bits as u8, *signed), span.start_line, span.start_col
        )));
    }
    Ok(())
}
