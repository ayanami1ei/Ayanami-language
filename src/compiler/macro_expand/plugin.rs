//! A5b-2：宏插件构建（llc PIC + C shim + runtime）与 dlopen 调用。

use std::ffi::{CString, c_char, c_int, c_void};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

mod dl {
    use super::*;
    #[link(name = "dl")]
    unsafe extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> c_int;
    }
}
const RTLD_NOW: c_int = 2;

// ═══════════════════════════════════════════════════════════════════
//  插件构建与调用
// ═══════════════════════════════════════════════════════════════════

pub(super) fn invoke_plugin(lcl_path: &str, macro_name: &str, input: &str) -> Result<String> {
    let (_syms, _src, lir_binary, _tt) = crate::package::load_package(lcl_path)
        .map_err(|e| Error::Compile(format!("macro package '{}': {}", lcl_path, e)))?;
    let lir = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| Error::Compile(format!("macro LIR decode: {}", e)))?;

    let f = lir.functions.iter().find(|f| f.name.as_str() == macro_name)
        .ok_or_else(|| Error::Compile(format!("macro `{}` has no compiled body in `{}`", macro_name, lcl_path)))?;
    let symbol = lir.fn_names.get(&f.fn_id)
        .ok_or_else(|| Error::Compile(format!("macro `{}` has no symbol", macro_name)))?
        .clone();
    let arity = f.params.len();
    if arity > 1 || f.params.iter().any(|(_, t)| t != &crate::hir::ty::HirType::Named(crate::intern::Symbol::intern("String")))
        || f.return_type != crate::hir::ty::HirType::Named(crate::intern::Symbol::intern("String"))
    {
        return Err(Error::Compile(format!(
            "macro `{}` must have signature fn(String) -> String (0 or 1 param), found {} param(s)",
            macro_name, arity
        )));
    }

    let so_path = build_plugin(&lir, &symbol, arity)?;
    call_plugin(&so_path, &symbol, arity, input)
}

fn build_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, arity: usize) -> Result<PathBuf> {
    let ir = crate::lir::emit_program(lir);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    symbol.hash(&mut hasher);
    arity.hash(&mut hasher);
    ir.hash(&mut hasher);
    let hash = hasher.finish();

    let dir = std::env::temp_dir().join("ayanami-macros");
    std::fs::create_dir_all(&dir).ok();
    let stem = symbol.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let so_path = dir.join(format!("mac_{}_{:x}.so", stem, hash));
    if so_path.exists() {
        return Ok(so_path);
    }

    let ll_path = dir.join(format!("mac_{}_{:x}.ll", stem, hash));
    let obj_path = dir.join(format!("mac_{}_{:x}.o", stem, hash));
    let shim_path = dir.join(format!("mac_{}_{:x}.c", stem, hash));
    std::fs::write(&ll_path, &ir).map_err(|e| Error::Compile(format!("macro IR write: {}", e)))?;

    // llc（PIC）
    let (llc, llc_dir) = crate::driver::find_llc()?;
    let mut cmd = Command::new(&llc);
    cmd.arg("-filetype=obj").arg("-relocation-model=pic")
        .arg("-o").arg(&obj_path).arg(&ll_path);
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }
    if !cmd.status().map_err(|e| Error::Compile(format!("macro llc: {}", e)))?.success() {
        return Err(Error::Compile("macro llc failed".into()));
    }

    // C shim：C ABI ↔ Ayanami String（{i8*, i64}）
    let call = if arity == 0 {
        format!("AyaStr r = {}();", symbol)
    } else {
        format!("AyaStr r = {}(s);", symbol)
    };
    let shim = format!(
        "#include <stdlib.h>\n#include <string.h>\n\
         typedef struct {{ char* data; long len; }} AyaStr;\n\
         extern AyaStr {}({});\n\
         char* __ayanami_macro_expand(const char* in, long in_len, long* out_len) {{\n\
             AyaStr s = {{ (char*)in, in_len }};\n\
             (void)s; (void)in_len;\n\
             {}\n\
             *out_len = r.len;\n\
             char* out = (char*)malloc((size_t)r.len + 1);\n\
             if (out) {{ memcpy(out, r.data, (size_t)r.len); out[r.len] = 0; }}\n\
             return out;\n\
         }}\n\
         void __ayanami_macro_free(char* p) {{ free(p); }}\n",
        symbol,
        if arity == 0 { "void" } else { "AyaStr" },
        call
    );
    std::fs::write(&shim_path, shim).map_err(|e| Error::Compile(format!("macro shim write: {}", e)))?;

    let runtime = crate::driver::find_runtime_c()?;
    let status = Command::new("gcc")
        .arg("-shared").arg("-fPIC")
        .arg(&obj_path).arg(&shim_path).arg(&runtime)
        .arg("-o").arg(&so_path)
        .status()
        .map_err(|e| Error::Compile(format!("macro gcc: {}", e)))?;
    if !status.success() {
        return Err(Error::Compile(format!(
            "macro plugin link failed for `{}`", symbol
        )));
    }
    Ok(so_path)
}

type ExpandFn = unsafe extern "C" fn(*const u8, usize, *mut usize) -> *mut u8;
type FreeFn = unsafe extern "C" fn(*mut c_char);

fn call_plugin(so_path: &Path, symbol: &str, _arity: usize, input: &str) -> Result<String> {
    let c_path = CString::new(so_path.to_string_lossy().as_bytes())
        .map_err(|_| Error::Compile("macro path contains NUL".into()))?;
    unsafe {
        let handle = dl::dlopen(c_path.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            return Err(Error::Compile(format!("dlopen failed: {}", so_path.display())));
        }
        let expand_sym = CString::new("__ayanami_macro_expand").unwrap();
        let free_sym = CString::new("__ayanami_macro_free").unwrap();
        let expand_ptr = dl::dlsym(handle, expand_sym.as_ptr());
        let free_ptr = dl::dlsym(handle, free_sym.as_ptr());
        if expand_ptr.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!(
                "macro `{}`: missing __ayanami_macro_expand", symbol
            )));
        }
        let expand: ExpandFn = std::mem::transmute(expand_ptr);
        let mut out_len = 0usize;
        let out = expand(input.as_ptr(), input.len(), &mut out_len);
        if out.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("macro `{}` returned null", symbol)));
        }
        let bytes = std::slice::from_raw_parts(out, out_len).to_vec();
        if !free_ptr.is_null() {
            let free: FreeFn = std::mem::transmute(free_ptr);
            free(out as *mut c_char);
        }
        dl::dlclose(handle);
        String::from_utf8(bytes).map_err(|e| Error::Compile(format!("macro output not UTF-8: {}", e)))
    }
}
