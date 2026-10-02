use super::*;

impl<'a> Emitter<'a> {
    pub(super) fn emit_struct_defs(&mut self) {
        for (name, fields) in &self.prog.struct_defs {
            let field_types: Vec<String> = fields.iter()
                .map(|(_, ty)| self.llvm_type(ty))
                .collect();
            let safe_name = sanitize_name(&name.as_str());
            self.wln_fmt(format_args!(
                "%struct.{} = type {{ {} }}",
                safe_name, field_types.join(", ")
            ));
        }
        if !self.prog.struct_defs.is_empty() {
            self.wln("");
        }
    }

    pub(super) fn struct_llvm_name(&self, name: &Symbol) -> Option<String> {
        // 尝试完整类型名，再尝试剥离泛型参数后的基名
        if self.prog.struct_defs.contains_key(name) {
            return Some(format!("%struct.{}", sanitize_name(&name.as_str())));
        }
        let s = name.as_str();
        let base = s.find('<').or_else(|| s.find('[')).map(|p| &s[..p]);
        if let Some(base) = base {
            let base_sym = Symbol::intern(base);
            if self.prog.struct_defs.contains_key(&base_sym) {
                return Some(format!("%struct.{}", sanitize_name(base)));
            }
        }
        None
    }

    pub(super) fn emit_string_globals(&mut self) {
        for (i, s) in self.prog.strings.iter().enumerate() {
            let escaped = escape_llvm_string(s);
            let len = s.len();
            self.wln_fmt(format_args!(
                "@__str_{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                i,
                len + 1,
                escaped
            ));
        }
        if !self.prog.strings.is_empty() {
            self.wln("");
        }
    }

    // ----------------------------------------------------------------
    //  Function
    // ----------------------------------------------------------------

    pub(super) fn emit_fn(&mut self, f: &LirFn) {
        let fn_name = self.prog.fn_names[&f.fn_id].clone();
        let ret_ty = self.llvm_type(&f.return_type);
        let params_str: Vec<String> = f
            .params
            .iter()
            .map(|(_, t)| self.llvm_type(t))
            .collect();
        let param_list = params_str.join(", ");
        let inline_attr = if f.is_inline { " alwaysinline" } else { "" };

        self.current_fn_ret_ty = f.return_type.clone();

        self.wln_fmt(format_args!(
            "define {} @{}({}){} {{",
            ret_ty, fn_name, param_list, inline_attr
        ));
        self.indent += 1;

        // Emit all allocas and other instructions per block
        for (idx, block) in f.blocks.iter().enumerate() {
            if idx > 0 {
                self.wln_fmt(format_args!("{}:", block.label));
            }

            for inst in &block.insts {
                self.emit_inst(inst);
            }
        }

        self.indent -= 1;
        self.wln("}");
        self.wln("");
    }

    // ----------------------------------------------------------------
    //  Instruction emission — simplified to trait dispatch
    // ----------------------------------------------------------------

    pub(super) fn emit_inst(&mut self, inst: &LirNodeBox) {
        let mut ctx = LirEmitCtx {
            prog: self.prog,
            load_tmp: self.load_tmp,
            current_fn_ret_ty: self.current_fn_ret_ty.clone(),
        };
        let lines = inst.emit(&mut ctx);
        self.load_tmp = ctx.load_tmp;
        for line in lines {
            self.wln(&line);
        }
    }

    // ----------------------------------------------------------------
    //  Value references
    // ----------------------------------------------------------------

    pub(super) fn tmp(&mut self) -> u64 {
        let t = self.load_tmp;
        self.load_tmp += 1;
        t
    }

    pub(super) fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String {
        match val {
            LirValue::Tmp(t) => format!("%t{}", t),
            LirValue::Param(i) => format!("%{}", i),
            LirValue::Var(v) => format!("%v{}", v.0),
            LirValue::Literal(lit, _) => lit_to_string(lit, expected_ty),
        }
    }
}

// ----------------------------------------------------------------
//  Utilities
// ----------------------------------------------------------------



fn escape_llvm_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\22"),
            '\\' => out.push_str("\\5c"),
            '\n' => out.push_str("\\0a"),
            '\r' => out.push_str("\\0d"),
            '\t' => out.push_str("\\09"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            c => out.push_str(&format!("\\{:02x}", c as u8)),
        }
    }
    out
}
