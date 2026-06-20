// ============================================================
//  LLVM IR 发射器
//  将 LIR（三地址码）转换为 LLVM IR 文本（.ll 格式），主要工作：
//  1. 生成 LLVM 模块和类型声明
//  2. 为每个函数生成 LLVM 函数定义（含基本块和指令）
//  3. 生成全局字符串常量和虚函数表
//  4. 处理内存管理指令（DropValue/RetainValue/ReleaseValue）
//  5. 处理虚函数调用（通过 vtable 间接调用）
//  6. 结构体和数组的内存布局与存取
// ============================================================

use crate::intern::Symbol;
use crate::hir::ir::{FnId, HirLiteral, HirType};
use crate::parser::ast::{BinaryOp, UnaryOp};

use super::ir::*;

/// 将 LIR 程序发射为 LLVM IR 文本
pub fn emit_program(prog: &LirProgram) -> String {
    let mut e = Emitter::new(prog);
    e.emit();
    e.finish()
}

// ----------------------------------------------------------------
//  Emitter
// ----------------------------------------------------------------

struct Emitter<'a> {
    out: String,
    prog: &'a LirProgram,
    indent: usize,
    load_tmp: u64,
    current_fn_ret_ty: HirType,
}

impl<'a> Emitter<'a> {
    fn new(prog: &'a LirProgram) -> Self {
        Self {
            out: String::new(),
            prog,
            indent: 0,
            load_tmp: 0,
            current_fn_ret_ty: HirType::Void,
        }
    }

    fn finish(self) -> String {
        self.out
    }

    fn wln(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn wln_fmt(&mut self, fmt: std::fmt::Arguments<'_>) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(&format!("{}", fmt));
        self.out.push('\n');
    }

    fn llvm_type(&self, ty: &HirType) -> String {
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
            HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
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

    fn emit(&mut self) {
        // Header
        self.wln("; ModuleID = 'ayanami'");
        self.wln("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"");
        self.wln("target triple = \"x86_64-unknown-linux-gnu\"");
        self.wln("");

        // String globals
        self.emit_string_globals();

        // Struct type definitions
        self.emit_struct_defs();

        // Vtable globals
        self.emit_vtable_globals();

        // Runtime declarations
        self.wln("declare void @free(i8*)");
        self.wln("declare i8* @malloc(i64)");
        self.wln("declare i8* @__ayanami_shared_alloc(i64)");
        self.wln("declare void @__ayanami_shared_retain(i8*)");
        self.wln("declare void @__ayanami_shared_release(i8*)");
        self.wln("declare void @llvm.memcpy.p0.p0.i64(i8*, i8*, i64, i1)");
        self.wln("declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)");
        self.wln("declare i32 @putchar(i32)");
        self.wln("declare i32 @printf(i8*, ...)");
        self.wln("");

        // Extern declarations for imported functions
        for &fn_id in &self.prog.imported_fn_ids {
            if let Some(fn_name) = self.prog.fn_names.get(&fn_id) {
                // Use void return and ... params as a generic declaration
                self.wln_fmt(format_args!("declare i64 @{}()", fn_name));
            }
        }
        if !self.prog.imported_fn_ids.is_empty() {
            self.wln("");
        }

        // Functions
        for func in &self.prog.functions {
            self.emit_fn(func);
        }
    }

    fn emit_vtable_globals(&mut self) {
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

    fn fn_needs_vtable_wrapper(&self, fn_id: FnId) -> bool {
        self.prog.functions.iter().find(|f| f.fn_id == fn_id).map(|f| {
            f.params.first().map(|(_, t)| {
                let inner = match t {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                matches!(inner, HirType::Int | HirType::Float | HirType::Char | HirType::Bool)
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn fn_ret_type(&self, fn_id: FnId) -> String {
        self.prog.functions.iter()
            .find(|f| f.fn_id == fn_id)
            .map(|f| self.llvm_type(&f.return_type))
            .unwrap_or_else(|| "void".to_string())
    }

    fn fn_first_param_unwrapped(&self, fn_id: FnId) -> String {
        self.prog.functions.iter()
            .find(|f| f.fn_id == fn_id)
            .and_then(|f| f.params.first().map(|(_, t)| {
                let inner = match t {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                self.llvm_type(inner)
            }))
            .unwrap_or_else(|| "i64".to_string())
    }

    /// Get LLVM types and parameter indices for extra params (after the first one).
    /// Returns `(param_decls, param_refs)` where decls are like `"i64 %p1, double %p2"`
    /// and refs are like `"i64 %p1, double %p2"` usable in call arguments.
    fn fn_extra_params(&self, fn_id: FnId) -> (Vec<String>, Vec<String>) {
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

    fn emit_vtable_wrappers(&mut self) {
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

    fn emit_struct_defs(&mut self) {
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

    fn struct_llvm_name(&self, name: &Symbol) -> Option<String> {
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

    fn emit_string_globals(&mut self) {
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

    fn emit_fn(&mut self, f: &LirFn) {
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

    fn emit_inst(&mut self, inst: &LirNodeBox) {
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

    fn tmp(&mut self) -> u64 {
        let t = self.load_tmp;
        self.load_tmp += 1;
        t
    }

    fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String {
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
