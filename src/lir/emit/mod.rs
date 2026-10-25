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

use crate::hir::ir::{FnId, HirType};


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
            // 发射期临时量与 LIR 降级期 dest 编号分属不同命名空间（dest 每函数从 0 起），
            // 从高位起编避免 %tN 冲突
            load_tmp: 1_000_000,
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
        // M-opt.10：release 直接用 libc malloc/free（LLVM 识别分配/释放对 →
        // 跨模块内联后不逃逸的分配可提升为 alloca；debug 保留 runtime 包装以计泄漏）
        if crate::hir::contracts::is_release() {
            let s = s.replace("@__ayanami_unique_alloc", "@malloc")
                     .replace("@__ayanami_unique_free", "@free");
            self.out.push_str(&s);
        } else {
            self.out.push_str(s);
        }
        self.out.push('\n');
    }

    fn wln_fmt(&mut self, fmt: std::fmt::Arguments<'_>) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(&format!("{}", fmt));
        self.out.push('\n');
    }


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

        // M6.2：全局变量定义
        self.emit_global_defs();

        // Vtable globals
        self.emit_vtable_globals();

        // #195：LLVM 21 拒绝同一模块内的重复声明/声明+定义——用户声明或定义的符号
        // （如自定义分配器 __ayanami_unique_alloc）以用户签名为准，跳过内建声明
        let mut user_declared: std::collections::HashSet<&str> = self.prog.extern_decls
            .iter().map(|d| d.name.as_str()).collect();
        for f in &self.prog.functions {
            if let Some(n) = self.prog.fn_names.get(&f.fn_id) {
                user_declared.insert(n.as_str());
            }
        }
        let builtin_decl = |name: &str, line: &'static str| -> Option<&'static str> {
            if !user_declared.contains(name) { Some(line) } else { None }
        };

        // Runtime declarations（M-opt.8：分配器 malloc 语义 noalias/allocsize；
        // 契约失败路径 cold；C 侧无栈展开 nounwind）
        // M-opt.10：release 直接声明 libc malloc/free（堆提升）
        if crate::hir::contracts::is_release() {
            for l in [builtin_decl("malloc", "declare ptr @malloc(i64) nounwind"),
                      builtin_decl("free", "declare void @free(ptr) nounwind")].into_iter().flatten() {
                self.wln(l);
            }
        } else {
            for l in [builtin_decl("__ayanami_unique_alloc", "declare noalias i8* @__ayanami_unique_alloc(i64) allocsize(0) nounwind"),
                      builtin_decl("__ayanami_unique_free", "declare void @__ayanami_unique_free(i8*) nounwind")].into_iter().flatten() {
                self.wln(l);
            }
        }
        self.wln("declare void @llvm.memcpy.p0.p0.i64(i8*, i8*, i64, i1)");
        self.wln("declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)");
        self.wln("declare void @llvm.assume(i1)");
        for l in [builtin_decl("__ayanami_require_fail", "declare void @__ayanami_require_fail(i64, i64) noreturn cold nounwind"),
                  builtin_decl("__ayanami_ensure_fail", "declare void @__ayanami_ensure_fail(i64, i64) noreturn cold nounwind"),
                  builtin_decl("__ayanami_invariant_fail", "declare void @__ayanami_invariant_fail(i64, i64) noreturn cold nounwind")].into_iter().flatten() {
            self.wln(l);
        }
        self.wln("");

        // Extern declarations（真实签名 + LLVM 属性）；#195：同名去重（首个为准）
        let mut emitted_decls: std::collections::HashSet<&str> = std::collections::HashSet::new();
        // d5c/nsr：本地已有定义的同名符号不再发 declare（LLVM 拒绝 declare + internal define）
        let defined_fns: std::collections::HashSet<&str> = self.prog.functions.iter()
            .filter_map(|f| self.prog.fn_names.get(&f.fn_id).map(|n| n.as_str()))
            .collect();
        for d in &self.prog.extern_decls {
            if defined_fns.contains(d.name.as_str()) { continue; }
            if !emitted_decls.insert(d.name.as_str()) {
                continue;
            }
            let ret = self.llvm_type(&d.return_type);
            let params: Vec<String> = d.params.iter().enumerate()
                .map(|(i, t)| {
                    let mut suffix = d.param_attrs.get(i).map(|v| functions::llvm_param_attrs(v)).unwrap_or_default();
                    // M-opt.7：release 导入函数（.lcl）声明按所有权模型推断；
                    // #157：源码 extern "C" 声明除外（C 可能写 ref 形参内存）
                    if crate::hir::contracts::is_release() && !d.extern_c {
                        for a in functions::infer_param_attrs(t).split_whitespace() {
                            if !suffix.contains(a) { suffix.push(' '); suffix.push_str(a); }
                        }
                    }
                    format!("{}{}", self.llvm_type(t), suffix)
                })
                .collect();
            let has_ptr_params = d.params.iter().any(functions::is_ptr_like);
            let mut suffix = functions::llvm_attr_suffix(&d.attrs, false, d.effects, has_ptr_params);
            // M-opt.1：release 对 runtime/FFI 声明也标 nounwind（C 侧无栈展开）
            if crate::hir::contracts::is_release() && !suffix.contains("nounwind") {
                suffix.push_str(" nounwind");
            }
            // M-opt.7：release 返回值属性（extern C 声明除外）
            let ret_attr = if crate::hir::contracts::is_release() && !d.extern_c {
                functions::infer_ret_attr(&d.return_type)
            } else {
                ""
            };
            // mcl.2：宏 ABI 助手（hygiene/诊断）由插件 shim 提供——宏库作为模块编入
            // 可执行文件时允许未定义（extern_weak），避免链接失败（调用仅存在于插件路径）
            let weak = if d.name.starts_with("__ayanami_macro_") { "extern_weak " } else { "" };
            self.wln_fmt(format_args!("declare {}{}{} @{}({}){}", weak, ret_attr, ret, d.name, params.join(", "), suffix));
        }
        if !self.prog.extern_decls.is_empty() {
            self.wln("");
        }

        // Functions
        for func in &self.prog.functions {
            self.emit_fn(func);
        }
    }

}

mod consts;
mod functions;
mod types;
mod vtable;
