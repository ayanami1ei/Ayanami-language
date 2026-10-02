use super::*;

impl crate::hir::lower::Ctx {
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
