use super::*;

impl<'a> Emitter<'a> {
    pub(super) fn llvm_type(&self, ty: &HirType) -> String {
        match ty {
            HirType::Int => "i64".into(),
            HirType::Float => "double".into(),
            HirType::F32 => "float".into(),
            HirType::Char => "i8".into(),
            HirType::Bool => "i1".into(),
            HirType::Void | HirType::Never => "void".into(),
            HirType::IntN { bits, .. } => format!("i{}", bits),
            HirType::Named(s) => {
                if self.prog.struct_defs.contains_key(s) {
                    return format!("%struct.{}", sanitize_name(&s.as_str()));
                }
                // 泛型实例名（Result<int,int>）回退到基名（Result）
                let name = s.as_str();
                if let Some(pos) = name.find('<').or_else(|| name.find('[')) {
                    let base = crate::intern::Symbol::intern(&name[..pos]);
                    if self.prog.struct_defs.contains_key(&base) {
                        return format!("%struct.{}", sanitize_name(&name[..pos]));
                    }
                }
                "i8*".into()
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
            HirType::Closure(..) => "{ ptr, ptr }".into(),
            HirType::Ref(_, _) => "ptr".into(),
        }
    }

    // ----------------------------------------------------------------
    //  Entry
    // ----------------------------------------------------------------

}
