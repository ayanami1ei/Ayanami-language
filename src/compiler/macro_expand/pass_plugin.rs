//! A5d-2：MIR pass 插件构建与调用（schema v0 的生成式 C 桥接）。
//!
//! shim 负责：AyaBuf(blob) → 解码为 `std/mir.aya` 的 `MirFunction`（C 结构体布局一致）
//! → 调用用户 pass → 编码回 AyaBuf。Ayanami 侧无需手写编解码。

use std::collections::hash_map::DefaultHasher;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::plugin::{compile_pic, dl, resolve_dep_lcls, AyaBuf, AyaBufList, ABI_VERSION, RTLD_NOW};
use crate::error::{Error, Result};
use crate::intern::Symbol;

/// 与 `compiler/build/passes.rs::SCHEMA_VERSION` 保持一致
const SCHEMA_VERSION: i64 = 2;

/// 加载并构建 MIR 注解插件（pass/check 共用）；返回 (.so 路径, 符号名)
/// `want_mut=true`：pass（`ref mut`）；`false`：check（只读 `ref`）
pub(super) fn build_for(lcl_path: &str, ann_name: &str, want_mut: bool) -> Result<(PathBuf, String)> {
    let (_syms, _src, lir_binary, _tt) = crate::package::load_package(lcl_path)
        .map_err(|e| Error::Compile(format!("MIR annotation package '{}': {}", lcl_path, e)))?;
    let lir = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| Error::Compile(format!("MIR annotation LIR decode: {}", e)))?;
    let f = lir.functions.iter().find(|f| f.name.as_str() == ann_name)
        .ok_or_else(|| Error::Compile(format!("MIR annotation `{}` has no compiled body in `{}`", ann_name, lcl_path)))?;
    let symbol = lir.fn_names.get(&f.fn_id)
        .ok_or_else(|| Error::Compile(format!("MIR annotation `{}` has no symbol", ann_name)))?
        .clone();
    let mir_ty = crate::hir::ty::HirType::Named(Symbol::intern("MirFunction"));
    let sig_ok = f.params.len() == 1
        && matches!(&f.params[0].1, crate::hir::ty::HirType::Ref(inner, m) if **inner == mir_ty && *m == want_mut)
        && f.return_type == crate::hir::ty::HirType::Void;
    if !sig_ok {
        let expected = if want_mut { "fn(ref mut MirFunction) -> void" } else { "fn(ref MirFunction) -> void" };
        return Err(Error::Compile(format!(
            "MIR annotation `{}` must have signature {}, found {} param(s)",
            ann_name, expected, f.params.len()
        )));
    }
    let deps = resolve_dep_lcls(lcl_path)?;
    let so_path = build_pass_plugin(&lir, &symbol, &deps)?;
    Ok((so_path, symbol))
}

pub(super) fn invoke_pass(lcl_path: &str, pass_name: &str, blob: &[u8]) -> Result<Vec<u8>> {
    let (so_path, symbol) = build_for(lcl_path, pass_name, true)?;
    call_pass(&so_path, &symbol, blob)
}

fn build_pass_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, dep_lcls: &[String]) -> Result<PathBuf> {
    let ir = crate::lir::emit_program(lir);
    let mut dep_irs: Vec<String> = Vec::new();
    for dep in dep_lcls {
        let (_s, _src, bin, _tt) = crate::package::load_package(dep)
            .map_err(|e| Error::Compile(format!("pass dep '{}': {}", dep, e)))?;
        let dlir = crate::lir::serialize::program_from_bytes(&bin)
            .map_err(|e| Error::Compile(format!("pass dep LIR decode '{}': {}", dep, e)))?;
        dep_irs.push(crate::lir::emit_program(&dlir));
    }

    let mut hasher = DefaultHasher::new();
    ABI_VERSION.hash(&mut hasher);
    SCHEMA_VERSION.hash(&mut hasher);
    symbol.hash(&mut hasher);
    ir.hash(&mut hasher);
    for d in &dep_irs { d.hash(&mut hasher); }
    let hash = hasher.finish();

    let dir = std::env::temp_dir().join("ayanami-macros");
    std::fs::create_dir_all(&dir).ok();
    let stem = symbol.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let so_path = dir.join(format!("pass_{}_{:x}.so", stem, hash));
    if so_path.exists() {
        return Ok(so_path);
    }

    let ll_path = dir.join(format!("pass_{}_{:x}.ll", stem, hash));
    let obj_path = dir.join(format!("pass_{}_{:x}.o", stem, hash));
    let shim_path = dir.join(format!("pass_{}_{:x}.c", stem, hash));
    std::fs::write(&ll_path, &ir).map_err(|e| Error::Compile(format!("pass IR write: {}", e)))?;

    compile_pic(&ll_path, &obj_path)?;
    let mut link_objs = vec![obj_path.clone()];
    for dep_ir in &dep_irs {
        let mut h = DefaultHasher::new();
        dep_ir.hash(&mut h);
        let dep_obj = dir.join(format!("dep_{:x}.o", h.finish()));
        if !dep_obj.exists() {
            let dep_ll = dir.join(format!("dep_{:x}.ll", h.finish()));
            std::fs::write(&dep_ll, dep_ir)
                .map_err(|e| Error::Compile(format!("pass dep IR write: {}", e)))?;
            compile_pic(&dep_ll, &dep_obj)?;
        }
        link_objs.push(dep_obj);
    }

    std::fs::write(&shim_path, pass_shim(symbol))
        .map_err(|e| Error::Compile(format!("pass shim write: {}", e)))?;

    let runtime = crate::driver::find_runtime_c()?;
    let mut gcc = Command::new("gcc");
    gcc.arg("-shared").arg("-fPIC");
    for o in &link_objs { gcc.arg(o); }
    let status = gcc
        .arg(&shim_path).arg(&runtime)
        .arg("-o").arg(&so_path)
        .status()
        .map_err(|e| Error::Compile(format!("pass gcc: {}", e)))?;
    if !status.success() {
        return Err(Error::Compile(format!("pass plugin link failed for `{}`", symbol)));
    }
    Ok(so_path)
}

/// 生成 schema v0 的 C 桥接（结构体布局与 std/mir.aya 的 MirFunction 一致）
fn pass_shim(symbol: &str) -> String {
    format!(
        "#include <stdlib.h>\n#include <string.h>\n\
         typedef struct {{ char* data; long len; }} AyaBuf;\n\
         typedef struct {{ const AyaBuf* items; long count; }} AyaBufList;\n\
         typedef struct {{\n\
             long long pure, no_error, node_count;\n\
             const long long *kinds, *is_call, *callee_pure, *is_alloc, *is_asm, *is_store, *child_counts,\n\
                         *ops, *lit_kinds, *lit_i64, *lit_f64, *edit_kind, *edit_i64;\n\
         }} MirFunction;\n\
         extern void {symbol}(MirFunction*);\n\
         static long long rd64(const char* p) {{\n\
             long long v = 0; int i;\n\
             for (i = 7; i >= 0; i--) v = (v << 8) | (unsigned char)p[i];\n\
             return v;\n\
         }}\n\
         static long long* rd_arr(const char* p, long* pos, long long n) {{\n\
             long long* a = (long long*)malloc((size_t)n * sizeof(long long));\n\
             long long i;\n\
             for (i = 0; i < n; i++) {{ a[i] = rd64(p + *pos); *pos += 8; }}\n\
             return a;\n\
         }}\n\
         static void wr64(char* p, long long v) {{\n\
             int i;\n\
             for (i = 0; i < 8; i++) {{ p[i] = (char)(v & 0xff); v >>= 8; }}\n\
         }}\n\
         static char* g_diag = 0;\n\
         static long g_diag_len = 0;\n\
         void __ayanami_diag_emit(long long level, AyaBuf msg) {{\n\
             char* nb = (char*)realloc(g_diag, (size_t)(g_diag_len + 16 + msg.len));\n\
             if (!nb) return;\n\
             g_diag = nb;\n\
             wr64(g_diag + g_diag_len, level); g_diag_len += 8;\n\
             wr64(g_diag + g_diag_len, msg.len); g_diag_len += 8;\n\
             memcpy(g_diag + g_diag_len, msg.data, (size_t)msg.len); g_diag_len += msg.len;\n\
         }}\n\
         AyaBuf __ayanami_diag_take(void) {{\n\
             AyaBuf b; b.data = g_diag; b.len = g_diag_len;\n\
             g_diag = 0; g_diag_len = 0;\n\
             return b;\n\
         }}\n\
         static MirFunction mir_decode(const char* p, long len) {{\n\
             MirFunction f; long pos = 0; long long n; (void)len;\n\
             rd64(p + pos); pos += 8;\n\
             f.pure = rd64(p + pos); pos += 8;\n\
             f.no_error = rd64(p + pos); pos += 8;\n\
             f.node_count = rd64(p + pos); pos += 8;\n\
             n = f.node_count;\n\
             f.kinds = rd_arr(p, &pos, n);\n\
             f.is_call = rd_arr(p, &pos, n);\n\
             f.callee_pure = rd_arr(p, &pos, n);\n\
             f.is_alloc = rd_arr(p, &pos, n);\n\
             f.is_asm = rd_arr(p, &pos, n);\n\
             f.is_store = rd_arr(p, &pos, n);\n\
             f.child_counts = rd_arr(p, &pos, n);\n\
             f.ops = rd_arr(p, &pos, n);\n\
             f.lit_kinds = rd_arr(p, &pos, n);\n\
             f.lit_i64 = rd_arr(p, &pos, n);\n\
             f.lit_f64 = rd_arr(p, &pos, n);\n\
             f.edit_kind = rd_arr(p, &pos, n);\n\
             f.edit_i64 = rd_arr(p, &pos, n);\n\
             return f;\n\
         }}\n\
         static AyaBuf mir_encode(MirFunction f) {{\n\
             long long n = f.node_count; long long i;\n\
             long total = 4*8 + n*13*8; long pos = 0;\n\
             char* out = (char*)malloc((size_t)total + 1);\n\
             wr64(out+pos, {ver}); pos += 8;\n\
             wr64(out+pos, f.pure); pos += 8;\n\
             wr64(out+pos, f.no_error); pos += 8;\n\
             wr64(out+pos, n); pos += 8;\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.kinds[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.is_call[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.callee_pure[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.is_alloc[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.is_asm[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.is_store[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.child_counts[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.ops[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.lit_kinds[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.lit_i64[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.lit_f64[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.edit_kind[i]); pos += 8; }}\n\
             for (i = 0; i < n; i++) {{ wr64(out+pos, f.edit_i64[i]); pos += 8; }}\n\
             AyaBuf b; b.data = out; b.len = pos; return b;\n\
         }}\n\
         AyaBuf __ayanami_pass_run(AyaBuf input, AyaBufList args) {{\n\
             (void)args;\n\
             MirFunction f = mir_decode(input.data, input.len);\n\
             {symbol}(&f);\n\
             return mir_encode(f);\n\
         }}\n\
         void __ayanami_pass_free(AyaBuf b) {{ free(b.data); }}\n",
        symbol = symbol,
        ver = SCHEMA_VERSION
    )
}

type RunFn = unsafe extern "C" fn(AyaBuf, AyaBufList) -> AyaBuf;
type FreeFn = unsafe extern "C" fn(AyaBuf);

fn call_pass(so_path: &Path, symbol: &str, blob: &[u8]) -> Result<Vec<u8>> {
    let c_path = CString::new(so_path.to_string_lossy().as_bytes())
        .map_err(|_| Error::Compile("pass path contains NUL".into()))?;
    let in_buf = AyaBuf { data: blob.as_ptr() as *mut u8, len: blob.len() };
    let empty = AyaBufList { items: std::ptr::null(), count: 0 };
    unsafe {
        let handle = dl::dlopen(c_path.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            let err = dl::dlerror();
            let detail = if err.is_null() { String::new() } else {
                std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
            };
            return Err(Error::Compile(format!("pass dlopen failed: {}: {}", so_path.display(), detail)));
        }
        let run_sym = CString::new("__ayanami_pass_run").unwrap();
        let free_sym = CString::new("__ayanami_pass_free").unwrap();
        let run_ptr = dl::dlsym(handle, run_sym.as_ptr());
        let free_ptr = dl::dlsym(handle, free_sym.as_ptr());
        if run_ptr.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("pass `{}`: missing __ayanami_pass_run", symbol)));
        }
        let run: RunFn = std::mem::transmute(run_ptr);
        let out = run(in_buf, empty);
        if out.data.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("pass `{}` returned null", symbol)));
        }
        let bytes = std::slice::from_raw_parts(out.data, out.len).to_vec();
        if !free_ptr.is_null() {
            let free: FreeFn = std::mem::transmute(free_ptr);
            free(out);
        }
        dl::dlclose(handle);
        Ok(bytes)
    }
}
