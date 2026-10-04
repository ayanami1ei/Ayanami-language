use super::*;

impl<'a> Emitter<'a> {
    pub(super) fn emit_vtable_globals(&mut self) {
        // Emit wrapper functions for vtable entries (value types need ptr→load wrapper)
        self.emit_vtable_wrappers();

        // Emit vtable globals
        for vt in &self.prog.vtables {
            let elem_count = vt.fn_ids.len();
            self.wln_fmt(format_args!(
                "@{} = private unnamed_addr constant [{} x ptr] [",
                vt.name, elem_count
            ));
            self.indent += 1;
            for (i, &fn_id) in vt.fn_ids.iter().enumerate() {
                let comma = if i < elem_count - 1 { "," } else { "" };
                // usize::MAX is truncated to u32::MAX during serialization
                if fn_id.0 == usize::MAX || fn_id.0 == u32::MAX as usize {
                    self.wln_fmt(format_args!("ptr null{}", comma));
                } else {
                    let fn_name = &self.prog.fn_names[&fn_id];
                    let vt_name_str = &vt.name;
                    let wrapper_name = format!("{}_{}_wrap", fn_name, vt_name_str);
                    if self.fn_needs_vtable_wrapper(fn_id) {
                        self.wln_fmt(format_args!("ptr @{}{}", wrapper_name, comma));
                    } else {
                        self.wln_fmt(format_args!("ptr @{}{}", fn_name, comma));
                    }
                }
            }
            self.indent -= 1;
            self.wln("]");
            self.wln("");
        }
    }

    pub(super) fn fn_needs_vtable_wrapper(&self, fn_id: FnId) -> bool {
        self.prog.functions.iter().find(|f| f.fn_id == fn_id).map(|f| {
            f.params.first().map(|(_, t)| {
                let inner = match t {
                    HirType::Unique(inner) => inner.as_ref(),
                    other => other,
                };
                matches!(inner, HirType::Int | HirType::Float | HirType::Char | HirType::Bool | HirType::IntN { .. })
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    pub(super) fn fn_ret_type(&self, fn_id: FnId) -> String {
        self.prog.functions.iter()
            .find(|f| f.fn_id == fn_id)
            .map(|f| self.llvm_type(&f.return_type))
            .unwrap_or_else(|| "void".to_string())
    }

    pub(super) fn fn_first_param_unwrapped(&self, fn_id: FnId) -> String {
        self.prog.functions.iter()
            .find(|f| f.fn_id == fn_id)
            .and_then(|f| f.params.first().map(|(_, t)| {
                let inner = match t {
                    HirType::Unique(inner) => inner.as_ref(),
                    other => other,
                };
                self.llvm_type(inner)
            }))
            .unwrap_or_else(|| "i64".to_string())
    }

    /// Get LLVM types and parameter indices for extra params (after the first one).
    /// Returns `(param_decls, param_refs)` where decls are like `"i64 %p1, double %p2"`
    /// and refs are like `"i64 %p1, double %p2"` usable in call arguments.
    pub(super) fn fn_extra_params(&self, fn_id: FnId) -> (Vec<String>, Vec<String>) {
        let mut decls = Vec::new();
        let mut refs = Vec::new();
        let func = self.prog.functions.iter().find(|f| f.fn_id == fn_id);
        if let Some(f) = func {
            for (i, (_, t)) in f.params.iter().enumerate().skip(1) {
                let llvm_ty = self.llvm_type(t);
                decls.push(format!("{} %p{}", llvm_ty, i));
                refs.push(format!("{} %p{}", llvm_ty, i));
            }
        }
        (decls, refs)
    }

    pub(super) fn emit_vtable_wrappers(&mut self) {
        for vt in &self.prog.vtables {
            for &fn_id in &vt.fn_ids {
                if fn_id.0 == usize::MAX { continue; }
                if !self.fn_needs_vtable_wrapper(fn_id) { continue; }
                let fn_name = &self.prog.fn_names[&fn_id];
                let wrapper_name = format!("{}_{}_wrap", fn_name, vt.name);
                let ret_ty = self.fn_ret_type(fn_id);
                let param_ty = self.fn_first_param_unwrapped(fn_id);
                let (extra_decls, extra_refs) = self.fn_extra_params(fn_id);
                let all_params = {
                    let mut p = vec![format!("ptr %data")];
                    p.extend(extra_decls.iter().cloned());
                    p.join(", ")
                };
                let all_args = {
                    let mut a = vec![format!("{} %val", param_ty)];
                    a.extend(extra_refs.iter().cloned());
                    a.join(", ")
                };
                self.wln_fmt(format_args!(
                    "define {} @{}({}) {{",
                    ret_ty, wrapper_name, all_params
                ));
                self.indent += 1;
                self.wln_fmt(format_args!(
                    "%val = load {}, ptr %data, align 8",
                    param_ty
                ));
                if ret_ty == "void" {
                    self.wln_fmt(format_args!(
                        "call void @{}({})",
                        fn_name, all_args
                    ));
                    self.wln("ret void");
                } else {
                    let tmp = self.tmp();
                    self.wln_fmt(format_args!(
                        "%e{} = call {} @{}({})",
                        tmp, ret_ty, fn_name, all_args
                    ));
                    self.wln_fmt(format_args!("ret {} %e{}", ret_ty, tmp));
                }
                self.indent -= 1;
                self.wln("}");
                self.wln("");
            }
        }
    }

}
