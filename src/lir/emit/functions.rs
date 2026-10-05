use super::*;

/// 形参是否含指针/引用/拥有堆值（解引用会读内存）。
pub(super) fn is_ptr_like(t: &HirType) -> bool {
    matches!(
        t,
        HirType::Ref(..) | HirType::Unique(_) | HirType::Array(_)
            | HirType::ArraySized(_, _) | HirType::FatPtr { .. }
    )
}

/// 标注 + 效应摘要 → LLVM 函数属性。默认信任（ADR-3）：误标后果自负。
/// A3b：显式空集 `#[throws()]` → nounwind；`#[eff()]`/`#[pure]` → memory(none)。
pub(super) fn llvm_attr_suffix(
    attrs: &[LirAttr],
    is_inline: bool,
    effects: LirEffects,
    has_ptr_params: bool,
) -> String {
    let mut s = String::new();
    if is_inline {
        // 旧 inline 关键字：保持强制内联
        s.push_str(" alwaysinline");
    } else if let Some(a) = attrs.iter().find(|a| a.name == "inline") {
        if a.args.iter().any(|x| x == "always") {
            s.push_str(" alwaysinline");
        } else {
            s.push_str(" inlinehint");
        }
    }
    for a in attrs {
        match a.name.as_str() {
            "cold" => s.push_str(" cold"),
            "noreturn" => s.push_str(" noreturn"),
            "pure" => s.push_str(" memory(none)"),
            "readonly" => s.push_str(" memory(read)"),
            "nounwind" => s.push_str(" nounwind"),
            "willreturn" => s.push_str(" willreturn"),
            _ => {}
        }
    }
    if effects.no_throws && !s.contains("nounwind") {
        s.push_str(" nounwind");
    }
    if effects.no_effects && !s.contains("memory(") {
        // 自动 no_effects ≠ 不访问内存：解引用引用/拥有指针形参仍会读内存。
        // 若标 memory(none)，LLVM 可跨调用 CSE 只读方法（如 pop 后 len 仍返回旧值）。
        s.push_str(if has_ptr_params { " memory(read)" } else { " memory(none)" });
    }
    s
}

/// 形参标注 → LLVM 参数属性（A1b：仅 noalias/nonnull）。
pub(super) fn llvm_param_attrs(attrs: &[LirAttr]) -> String {
    let mut s = String::new();
    for a in attrs {
        match a.name.as_str() {
            "noalias" => s.push_str(" noalias"),
            "nonnull" => s.push_str(" nonnull"),
            _ => {}
        }
    }
    s
}

impl<'a> Emitter<'a> {
    pub(super) fn emit_struct_defs(&mut self) {
        // 稳定输出：HashMap 迭代顺序不定，按名称排序保证可复现
        let mut defs: Vec<_> = self.prog.struct_defs.iter().collect();
        defs.sort_by(|a, b| a.0.as_str().cmp(&b.0.as_str()));
        for (name, fields) in defs {
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


    /// M6.2：发射全局变量 `@name = global <ty> <init>`
    pub(super) fn emit_global_defs(&mut self) {
        for g in &self.prog.globals {
            let ty = self.llvm_type(&g.ty);
            let lit = lit_to_string(&g.value, &g.ty);
            self.wln_fmt(format_args!("@{} = global {} {}", g.name.as_str(), ty, lit));
        }
        if !self.prog.globals.is_empty() {
            self.wln("");
        }
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
        let mut params_str: Vec<String> = Vec::new();
        for (i, (_, t)) in f.params.iter().enumerate() {
            let attrs = f.param_attrs.get(i).map(|v| llvm_param_attrs(v)).unwrap_or_default();
            params_str.push(format!("{}{}", self.llvm_type(t), attrs));
        }
        let param_list = params_str.join(", ");
        let has_ptr_params = f.params.iter().any(|(_, t)| is_ptr_like(t));
        let inline_attr = llvm_attr_suffix(&f.attrs, f.is_inline, f.effects, has_ptr_params);

        self.current_fn_ret_ty = f.return_type.clone();

        let linkage = if self.prog.specialized_fns.contains(&f.fn_id) || f.is_inline {
            "linkonce_odr "
        } else {
            ""
        };
        self.wln_fmt(format_args!(
            "define {}{} @{}({}){} {{",
            linkage, ret_ty, fn_name, param_list, inline_attr
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

}

// ----------------------------------------------------------------
//  Utilities
// ----------------------------------------------------------------



fn escape_llvm_string(s: &str) -> String {
    // #122：按 UTF-8 字节转义（此前按 char 取低 8 位，非 ASCII 会截断且字节数不符）；
    // LLVM c"..." 中非可打印字节用大写十六进制 \XX
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'"' => out.push_str("\\22"),
            b'\\' => out.push_str("\\5c"),
            b'\n' => out.push_str("\\0a"),
            b'\r' => out.push_str("\\0d"),
            b'\t' => out.push_str("\\09"),
            0x20..=0x7e => out.push(b as char),
            other => out.push_str(&format!("\\{:02X}", other)),
        }
    }
    out
}
