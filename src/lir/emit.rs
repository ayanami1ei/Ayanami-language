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
            HirType::Array(_) => "ptr".into(),
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
        if self.prog.struct_defs.contains_key(name) {
            Some(format!("%struct.{}", sanitize_name(&name.as_str())))
        } else {
            None
        }
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
    //  Instruction emission
    // ----------------------------------------------------------------

    fn emit_inst(&mut self, inst: &LirInst) {
        match inst {
            LirInst::Alloca(vid, ty) => {
                let llvm_ty = self.llvm_type(ty);
                self.wln_fmt(format_args!(
                    "%v{} = alloca {}, align 8",
                    vid.0, llvm_ty
                ));
            }
            LirInst::Store { dest, src, ty } => {
                let llvm_ty = self.llvm_type(ty);
                let src_str = match src {
                    LirValue::Param(i) => format!("%{}", i),
                    LirValue::Tmp(t) => format!("%t{}", t),
                    LirValue::Var(v) => {
                        let tmp = self.tmp();
                        self.wln_fmt(format_args!(
                        "%e{} = load {}, ptr %v{}, align 8",
                        tmp, llvm_ty, v.0
                        ));
                        format!("%e{}", tmp)
                    }
                    LirValue::Literal(lit, _) => lit_to_string(lit, ty),
                };
                self.wln_fmt(format_args!(
                    "store {} {}, ptr %v{}, align 8",
                    llvm_ty, src_str, dest.0
                ));
            }
            LirInst::Load { dest, src, ty } => {
                let llvm_ty = self.llvm_type(ty);
                self.wln_fmt(format_args!(
                    "%t{} = load {}, ptr %v{}, align 8",
                    dest, llvm_ty, src.0
                ));
            }
            LirInst::BinOp {
                dest,
                op,
                lhs,
                rhs,
                ty,
                result_ty: _,
            } => {
                let l = self.value_ref(lhs, ty);
                let r = self.value_ref(rhs, ty);
                // Detect pointer comparison: one side is "null" and the other is a temp
                let is_ptr = l == "null" || r == "null" || l.contains("ptr") || r.contains("ptr");

                let icmp_llvm = |ty: &HirType| -> &str {
                    match ty {
                        HirType::Char => "i8",
                        HirType::Bool => "i1",
                        HirType::Int => "i64",
                        _ => {
                            // Pointers and other types use ptr-sized integer
                            if is_ptr { "ptr" } else { "i64" }
                        }
                    }
                };

                match (op, ty) {
                    (BinaryOp::Add, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = add i64 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Sub, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = sub i64 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Mul, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = mul i64 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Div, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = sdiv i64 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Mod, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = srem i64 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Add, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fadd double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Sub, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fsub double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Mul, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fmul double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Div, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fdiv double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Mod, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = frem double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Add, HirType::Char) => {
                        self.wln_fmt(format_args!(
                            "%t{} = add i8 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Sub, HirType::Char) => {
                        self.wln_fmt(format_args!(
                            "%t{} = sub i8 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Eq, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp eq ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp eq {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Neq, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp ne ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp ne {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Lt, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp ult ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp slt {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Gt, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp ugt ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp sgt {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Le, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp ule ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp sle {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Ge, _) if *ty != HirType::Float => {
                        let llvm_int = icmp_llvm(ty);
                        if is_ptr {
                            self.wln_fmt(format_args!("%t{} = icmp uge ptr {}, {}", dest, l, r));
                        } else {
                            self.wln_fmt(format_args!("%t{} = icmp sge {} {}, {}", dest, llvm_int, l, r));
                        }
                    }
                    (BinaryOp::Eq, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp oeq double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Neq, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp one double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Lt, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp olt double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Gt, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp ogt double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Le, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp ole double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Ge, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fcmp oge double {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::And, _) => {
                        self.wln_fmt(format_args!(
                            "%t{} = and i1 {}, {}",
                            dest, l, r
                        ));
                    }
                    (BinaryOp::Or, _) => {
                        self.wln_fmt(format_args!(
                            "%t{} = or i1 {}, {}",
                            dest, l, r
                        ));
                    }
                    _ => {}
                }
            }
            LirInst::UnaryOp { dest, op, src, ty } => {
                let s = self.value_ref(src, ty);
                match (op, ty) {
                    (UnaryOp::Neg, HirType::Int) => {
                        self.wln_fmt(format_args!(
                            "%t{} = sub i64 0, {}",
                            dest, s
                        ));
                    }
                    (UnaryOp::Neg, HirType::Float) => {
                        self.wln_fmt(format_args!(
                            "%t{} = fsub double -0.0, {}",
                            dest, s
                        ));
                    }
                    (UnaryOp::Not, _) => {
                        self.wln_fmt(format_args!(
                            "%t{} = xor i1 1, {}",
                            dest, s
                        ));
                    }
                    _ => {}
                }
            }
            LirInst::Call {
                dest,
                fn_id,
                args,
                ret_ty,
            } => {
                let fn_name = &self.prog.fn_names[fn_id];
                let mut arg_strs = Vec::new();
                for (val, aty) in args {
                    let llvm_ty = self.llvm_type(aty);
                    let val_str = self.value_ref(val, aty);
                    arg_strs.push(format!("{} {}", llvm_ty, val_str));
                }
                let is_void = matches!(ret_ty, HirType::Void);
                let dest_str = match (dest, is_void) {
                    (Some(d), false) => format!("%t{} = ", d),
                    _ => String::new(),
                };
                let ret_llvm = self.llvm_type(ret_ty);
                self.wln_fmt(format_args!(
                    "{}call {} @{}({})",
                    dest_str, ret_llvm, fn_name, arg_strs.join(", ")
                ));
            }
            LirInst::StrGlobal { dest, str_idx } => {
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr inbounds i8, ptr @__str_{}, i64 0",
                    dest, str_idx
                ));
            }
            LirInst::Conv { dest, alloca_tmp, malloc_tmp, src, kind: _, src_ty, ty } => {
                let src_val = self.value_ref(src, src_ty);
                let inner_ty = match &ty {
                    HirType::Unique(i) | HirType::Shared(i) | HirType::Weak(i) => i.as_ref(),
                    _ => &ty,
                };
                let size = llvm_type_size(inner_ty);

                // If source is a struct value (not a pointer), store to alloca first
                let src_ptr = if matches!(src_ty, HirType::Named(s) if self.prog.struct_defs.contains_key(s)) {
                    let src_llvm = self.llvm_type(&src_ty);
                    self.wln_fmt(format_args!(
                        "%t{} = alloca {}, align 8",
                        alloca_tmp, src_llvm
                    ));
                    self.wln_fmt(format_args!(
                        "store {} {}, ptr %t{}",
                        src_llvm, src_val, alloca_tmp
                    ));
                    format!("%t{}", alloca_tmp)
                } else {
                    src_val.clone()
                };

                self.wln_fmt(format_args!(
                    "%l{} = call i8* @malloc(i64 {})",
                    malloc_tmp, size
                ));
                self.wln_fmt(format_args!(
                    "call void @llvm.memcpy.p0.p0.i64(i8* %l{}, ptr {}, i64 {}, i1 false)",
                    malloc_tmp, src_ptr, size
                ));
                self.wln_fmt(format_args!(
                    "%t{} = bitcast i8* %l{} to {}",
                    dest, malloc_tmp, self.llvm_type(&ty)
                ));
            }
            LirInst::DropValue(vid, ty) => {
                if needs_heap_ops(&ty) {
                    let tmp = self.tmp();
                    let llvm_ty = self.llvm_type(&ty);
                    self.wln_fmt(format_args!(
                        "%c{} = load {}, ptr %v{}, align 8",
                        tmp, llvm_ty, vid.0
                    ));
                    self.wln_fmt(format_args!("call void @free(i8* %c{})", tmp));
                }
            }
            LirInst::RetainValue(vid, ty) => {
                if needs_heap_ops(&ty) {
                    let tmp = self.tmp();
                    let llvm_ty = self.llvm_type(&ty);
                    self.wln_fmt(format_args!(
                        "%c{} = load {}, ptr %v{}, align 8",
                        tmp, llvm_ty, vid.0
                    ));
                    self.wln_fmt(format_args!(
                        "call void @__ayanami_shared_retain(i8* %c{})",
                        tmp
                    ));
                }
            }
            LirInst::ReleaseValue(vid, ty) => {
                if needs_heap_ops(&ty) {
                    let tmp = self.tmp();
                    let llvm_ty = self.llvm_type(&ty);
                    self.wln_fmt(format_args!(
                        "%c{} = load {}, ptr %v{}, align 8",
                        tmp, llvm_ty, vid.0
                    ));
                    self.wln_fmt(format_args!(
                        "call void @__ayanami_shared_release(i8* %c{})",
                        tmp
                    ));
                }
            }
            LirInst::Br(label) => {
                self.wln_fmt(format_args!("br label %{}", label));
            }
            LirInst::BrCond {
                cond,
                true_block,
                false_block,
            } => {
                let c = self.value_ref(cond, &HirType::Bool);
                self.wln_fmt(format_args!(
                    "br i1 {}, label %{}, label %{}",
                    c, true_block, false_block
                ));
            }
            LirInst::MakeFatPtr { dest, malloc_tmp, bc_tmp, vtable_gep_tmp, iv_tmp, value_src, value_ty, vtable_name, .. } => {
                // Allocate heap space for the value
                let size = llvm_type_size(value_ty);
                self.wln_fmt(format_args!(
                    "%t{} = call i8* @malloc(i64 {})",
                    malloc_tmp, size
                ));
                // Bitcast to the value type pointer and store
                let val_llvm = self.llvm_type(value_ty);
                self.wln_fmt(format_args!(
                    "%t{} = bitcast i8* %t{} to ptr",
                    bc_tmp, malloc_tmp
                ));
                let src_str = self.value_ref(value_src, value_ty);
                self.wln_fmt(format_args!(
                    "store {} {}, ptr %t{}, align 8",
                    val_llvm, src_str, bc_tmp
                ));
                // Get vtable pointer
                let vtable_elem_count = self.prog.vtables.iter()
                    .find(|v| v.name == *vtable_name)
                    .map(|v| v.fn_ids.len())
                    .unwrap_or(1);
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr [{} x ptr], ptr @{}, i64 0, i64 0",
                    vtable_gep_tmp, vtable_elem_count, vtable_name
                ));
                // Construct fat pointer struct {ptr, ptr}
                self.wln_fmt(format_args!(
                    "%t{} = insertvalue {{ ptr, ptr }} zeroinitializer, ptr %t{}, 0",
                    iv_tmp, malloc_tmp
                ));
                self.wln_fmt(format_args!(
                    "%t{} = insertvalue {{ ptr, ptr }} %t{}, ptr %t{}, 1",
                    dest, iv_tmp, vtable_gep_tmp
                ));
            }
            LirInst::VirtualCall { fn_dest, receiver_tmp, data_tmp, vtable_tmp, gep_tmp, fn_ptr_tmp, method_index, args, ret_ty } => {
                // Extract data and vtable from fat pointer
                self.wln_fmt(format_args!(
                    "%t{} = extractvalue {{ ptr, ptr }} %t{}, 0",
                    data_tmp, receiver_tmp
                ));
                self.wln_fmt(format_args!(
                    "%t{} = extractvalue {{ ptr, ptr }} %t{}, 1",
                    vtable_tmp, receiver_tmp
                ));
                // Index into vtable (skip slot 0 = destructor)
                let slot_idx = 1 + method_index;
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr ptr, ptr %t{}, i32 {}",
                    gep_tmp, vtable_tmp, slot_idx
                ));
                self.wln_fmt(format_args!(
                    "%t{} = load ptr, ptr %t{}",
                    fn_ptr_tmp, gep_tmp
                ));
                // Build call arguments: data ptr first, then method args
                let ret_llvm = self.llvm_type(ret_ty);
                let mut call_args = vec![format!("ptr %t{}", data_tmp)];
                for (val, aty) in args {
                    let llvm_ty = self.llvm_type(aty);
                    let val_str = self.value_ref(val, aty);
                    call_args.push(format!("{} {}", llvm_ty, val_str));
                }
                let dest_str = match (fn_dest, ret_ty) {
                    (Some(_), HirType::Void) | (None, _) => String::new(),
                    (Some(d), _) => format!("%t{} = ", d),
                };
                self.wln_fmt(format_args!(
                    "{}call {} %t{}({})",
                    dest_str, ret_llvm, fn_ptr_tmp, call_args.join(", ")
                ));
            }
            LirInst::FieldAccess { dest, gep_tmp, src, field_index, field_ty, struct_ty } => {
                // Strip ownership wrappers to get the inner Named type
                let inner = match struct_ty {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                let struct_name = match inner {
                    HirType::Named(n) => n,
                    _ => unreachable!(),
                };
                let struct_llvm = self.struct_llvm_name(struct_name)
                    .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
                let src_str = self.value_ref(src, &struct_ty);
                // If the struct is wrapped in Shared/Unique/Weak (heap pointer), use GEP + load
                // Otherwise (plain struct value), use extractvalue
                if matches!(struct_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)) {
                    self.wln_fmt(format_args!(
                        "%t{} = getelementptr {}, ptr {}, i32 0, i32 {}",
                        gep_tmp, struct_llvm, src_str, field_index
                    ));
                    self.wln_fmt(format_args!(
                        "%t{} = load {}, ptr %t{}",
                        dest, self.llvm_type(field_ty), gep_tmp
                    ));
                } else {
                    self.wln_fmt(format_args!(
                        "%t{} = extractvalue {} {}, {}",
                        dest, struct_llvm, src_str, field_index
                    ));
                }
            }
            LirInst::StructLit { dest, alloca_tmp, field_geps, fields, struct_name, struct_ty: _ } => {
                let struct_llvm = format!("%struct.{}", sanitize_name(&struct_name.as_str()));
                self.wln_fmt(format_args!(
                    "%t{} = alloca {}, align 8",
                    alloca_tmp, struct_llvm
                ));
                for (i, ((val, fty), gep_tmp)) in fields.iter().zip(field_geps.iter()).enumerate() {
                    let val_str = self.value_ref(val, fty);
                    let field_llvm = self.llvm_type(fty);
                    self.wln_fmt(format_args!(
                        "%t{} = getelementptr {}, ptr %t{}, i32 0, i32 {}",
                        gep_tmp, struct_llvm, alloca_tmp, i
                    ));
                    self.wln_fmt(format_args!(
                        "store {} {}, ptr %t{}",
                        field_llvm, val_str, gep_tmp
                    ));
                }
                self.wln_fmt(format_args!(
                    "%t{} = load {}, ptr %t{}",
                    dest, struct_llvm, alloca_tmp
                ));
            }
            LirInst::ArraySized { dest, malloc_tmp, count_tmp, size_tmp, elem_count, elem_size, .. } => {
                let count_str = self.value_ref(elem_count, &HirType::Int);
                self.wln_fmt(format_args!(
                    "%t{} = add i64 0, {}",
                    count_tmp, count_str
                ));
                self.wln_fmt(format_args!(
                    "%t{} = mul i64 %t{}, {}",
                    size_tmp, count_tmp, elem_size
                ));
                self.wln_fmt(format_args!(
                    "%t{} = call i8* @malloc(i64 %t{})",
                    malloc_tmp, size_tmp
                ));
                self.wln_fmt(format_args!(
                    "%t{} = bitcast i8* %t{} to ptr",
                    dest, malloc_tmp
                ));
                self.wln_fmt(format_args!(
                    "call void @llvm.memset.p0.i64(ptr %t{}, i8 0, i64 %t{}, i1 false)",
                    dest, size_tmp
                ));
            }
            LirInst::ArrayLit { dest, malloc_tmp, elem_geps, elems, elem_ty, ty: _ } => {
                let num_elems = elems.len();
                let elem_llvm = self.llvm_type(elem_ty);
                let elem_size = llvm_type_size(elem_ty).parse::<u64>().unwrap_or(8);
                let total_size = num_elems as u64 * elem_size;
                self.wln_fmt(format_args!(
                    "%t{} = call i8* @malloc(i64 {})",
                    malloc_tmp, total_size
                ));
                self.wln_fmt(format_args!(
                    "%t{} = bitcast i8* %t{} to ptr",
                    dest, malloc_tmp
                ));
                for (i, ((val, _fty), gep_tmp)) in elems.iter().zip(elem_geps.iter()).enumerate() {
                    let val_str = self.value_ref(val, elem_ty);
                    self.wln_fmt(format_args!(
                        "%t{} = getelementptr {}, ptr %t{}, i64 {}",
                        gep_tmp, elem_llvm, dest, i
                    ));
                    self.wln_fmt(format_args!(
                        "store {} {}, ptr %t{}",
                        elem_llvm, val_str, gep_tmp
                    ));
                }
            }
            LirInst::IndexAccess { dest, gep_tmp, load_tmp, arr, index, elem_ty, ty } => {
                let elem_llvm = self.llvm_type(elem_ty);
                let arr_str = self.value_ref(arr, ty);
                let idx_str = self.value_ref(index, &HirType::Int);
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr {}, ptr {}, i64 {}",
                    gep_tmp, elem_llvm, arr_str, idx_str
                ));
                self.wln_fmt(format_args!(
                    "%t{} = load {}, ptr %t{}",
                    load_tmp, elem_llvm, gep_tmp
                ));
                self.wln_fmt(format_args!(
                    "%t{} = bitcast {} %t{} to {}",
                    dest, elem_llvm, load_tmp, self.llvm_type(ty)
                ));
            }
            LirInst::FieldStore { dest, var_id, gep_tmp, iv_tmp, src, field_index, field_ty, struct_ty } => {
                let inner = match struct_ty {
                    HirType::Shared(inner) | HirType::Unique(inner) | HirType::Weak(inner) => inner.as_ref(),
                    other => other,
                };
                let struct_name = match inner {
                    HirType::Named(n) => n,
                    _ => unreachable!(),
                };
                let struct_llvm = self.struct_llvm_name(struct_name)
                    .unwrap_or_else(|| panic!("unknown struct type `{}`", struct_name));
                let src_str = self.value_ref(src, field_ty);
                let field_llvm = self.llvm_type(field_ty);
                if matches!(struct_ty, HirType::Shared(_) | HirType::Unique(_) | HirType::Weak(_)) {
                    // Heap pointer: GEP + store
                    self.wln_fmt(format_args!(
                        "%t{} = getelementptr {}, ptr %t{}, i32 0, i32 {}",
                        gep_tmp, struct_llvm, dest, field_index
                    ));
                    self.wln_fmt(format_args!(
                        "store {} {}, ptr %t{}",
                        field_llvm, src_str, gep_tmp
                    ));
                } else {
                    // Value type: insertvalue + store back to alloca
                    let var_ty = self.llvm_type(&struct_ty);
                    self.wln_fmt(format_args!(
                        "%t{} = insertvalue {} %t{}, {} {}, {}",
                        iv_tmp, struct_llvm, dest, field_llvm, src_str, field_index
                    ));
                    let store_var = var_id.expect("FieldStore: value type needs var_id");
                    self.wln_fmt(format_args!(
                        "store {} %t{}, ptr %v{}, align 8",
                        var_ty, iv_tmp, store_var.0
                    ));
                }
            }
            LirInst::IndexStore { dest, gep_tmp, src, index, elem_ty, array_ty: _ } => {
                let elem_llvm = self.llvm_type(elem_ty);
                let src_str = self.value_ref(src, elem_ty);
                let idx_str = self.value_ref(index, &HirType::Int);
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr {}, ptr %t{}, i64 {}",
                    gep_tmp, elem_llvm, dest, idx_str
                ));
                self.wln_fmt(format_args!(
                    "store {} {}, ptr %t{}",
                    elem_llvm, src_str, gep_tmp
                ));
            }
            LirInst::Asm { dest, template, output_constraints, input_operands, input_constraints, ret_ty } => {
                let ret_llvm = self.llvm_type(ret_ty);
                let constraint_str = {
                    let mut all = output_constraints.clone();
                    all.extend(input_constraints.iter().cloned());
                    all.join(",")
                };
                let args_str: Vec<String> = input_operands.iter()
                    .map(|(v, t)| format!("{} {}", self.llvm_type(t), self.value_ref(v, t)))
                    .collect();
                if let Some(d) = dest {
                    self.wln_fmt(format_args!(
                        "%t{} = call {} asm sideeffect \"{}\", \"{}\"({})",
                        d, ret_llvm, template, constraint_str, args_str.join(", ")
                    ));
                } else {
                    let args = if args_str.is_empty() { String::from("()") } else { format!("({})", args_str.join(", ")) };
                    self.wln_fmt(format_args!(
                        "call void asm sideeffect \"{}\", \"{}\"{}",
                        template, constraint_str, args
                    ));
                }
            }
            LirInst::RefInst { dest, var_id, mutable: _, ty: _ } => {
                // The alloca pointer IS the reference value
                self.wln_fmt(format_args!(
                    "%t{} = getelementptr i8, ptr %v{}, i32 0",
                    dest, var_id.0
                ));
            }
            LirInst::Ret(val) => match val {
                Some((v, ty)) => {
                    let s = self.value_ref(v, ty);
                    let llvm_ty = self.llvm_type(&self.current_fn_ret_ty);
                    self.wln_fmt(format_args!("ret {} {}", llvm_ty, s));
                }
                None => {
                    self.wln("ret void");
                }
            },
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

/// Sanitize a struct name for use as an LLVM identifier.
fn sanitize_name(name: &str) -> String {
    name.replace('<', "_lt_").replace('>', "_gt_").replace(',', "_c_")
}

fn lit_to_string(lit: &HirLiteral, expected_ty: &HirType) -> String {
    match (lit, expected_ty) {
        (HirLiteral::Int(0), ty) if is_pointer_type(ty) => "null".into(),
        (HirLiteral::Int(n), _) => format!("{}", n),
        (HirLiteral::Float(n), _) => {
            let s = format!("{}", n);
            if !s.contains('.') {
                format!("{}.0", s)
            } else {
                s
            }
        }
        (HirLiteral::Char(c), _) => format!("{}", *c as u8),
        (HirLiteral::Bool(b), _) => {
            if *b {
                "1".into()
            } else {
                "0".into()
            }
        }
        (HirLiteral::String(_), _) => {
            // For string literals, we need a GEP to the global
            // This case is handled specially in the instruction
            format!("null")
        }
    }
}

fn llvm_type_size(ty: &HirType) -> &'static str {
    match ty {
        HirType::Int => "8",
        HirType::Float => "8",
        HirType::Char => "1",
        HirType::Bool => "1",
        HirType::Void => "0",
        HirType::Named(_) | HirType::FatPtr { .. } => "16",
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => llvm_type_size(inner),
        HirType::Array(_) => "16",
        HirType::Ref(_, _) => "16",
    }
}

fn needs_heap_ops(ty: &HirType) -> bool {
    match ty {
        HirType::FatPtr { .. } => true,
        HirType::Unique(inner) | HirType::Shared(inner) | HirType::Weak(inner) => {
            matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. })
        }
        HirType::Named(_) => false,
        HirType::Array(_) => true,
        HirType::Ref(_, _) => false,
        _ => false,
    }
}

fn is_pointer_type(ty: &HirType) -> bool {
    matches!(ty,
        HirType::Named(_) | HirType::FatPtr { .. } | HirType::Array(_)
        | HirType::Unique(_) | HirType::Shared(_) | HirType::Weak(_)
    )
}

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
