use super::*;

impl<'a> Emitter<'a> {
    pub(super) fn llvm_type(&self, ty: &HirType) -> String {
        match ty {
            HirType::Int => "i64".into(),
            HirType::Float => "double".into(),
            HirType::Char => "i8".into(),
            HirType::Bool => "i1".into(),
            HirType::Void => "void".into(),
            HirType::Named(s) => {
                if self.prog.struct_defs.contains_key(s) {
                    format!("%struct.{}", sanitize_name(&s.as_str()))
                } else {
                    "i8*".into()
                }
            }
            HirType::Unique(inner) => {
                if matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. }) {
                    "ptr".into()
                } else {
                    self.llvm_type(inner)
                }
            }
            HirType::FatPtr { .. } => "{ ptr, ptr }".into(),
            HirType::Array(_) | HirType::ArraySized(_, _) => "ptr".into(),
            HirType::FnPtr(..) => "ptr".into(),
            HirType::Ref(_, _) => "ptr".into(),
        }
    }

    // ----------------------------------------------------------------
    //  Entry
    // ----------------------------------------------------------------

}
