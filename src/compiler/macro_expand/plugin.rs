//! A5b-2/A5b-3：宏插件构建（llc PIC + C shim + runtime）与 dlopen 调用（ABI v3，结构体封装）。

use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

pub(crate) use super::plugin_abi::{ABI_VERSION, AyaBuf, AyaBufList, ParamKind, RTLD_NOW, compile_pic, dl, resolve_dep_lcls};
use super::plugin_abi::{load_macro_fn, plan_params};

// ═══════════════════════════════════════════════════════════════════
//  插件构建与调用
// ═══════════════════════════════════════════════════════════════════

pub(super) fn invoke_plugin(lcl_path: &str, macro_name: &str, input: &str, args: &[String]) -> Result<String> {
    let (f, lir) = load_macro_fn(lcl_path, macro_name)?;
    // attribute 宏：全部 String 形参（input + args），不支持保留参数
    let mapping = plan_params(&f, args.len(), false)?;
    let dep_lcls = resolve_dep_lcls(lcl_path)?;
    let symbol = lir.fn_names.get(&f.fn_id)
        .ok_or_else(|| Error::Compile(format!("macro `{}` has no symbol", macro_name)))?
        .clone();
    let so_path = build_plugin(&lir, &symbol, &mapping, &dep_lcls)?;
    call_plugin(&so_path, &symbol, input, args, 0, 0, "")
}

/// A5c-2：函数宏展开（`#name(args)`），携带调用点信息
pub(crate) fn invoke_macro_expr(
    lcl_path: &str,
    macro_name: &str,
    args: &[String],
    line: usize,
    col: usize,
    file: &str,
) -> Result<String> {
    let (f, lir) = load_macro_fn(lcl_path, macro_name)?;
    let mapping = plan_params(&f, args.len(), true)?;
    let dep_lcls = resolve_dep_lcls(lcl_path)?;
    let symbol = lir.fn_names.get(&f.fn_id)
        .ok_or_else(|| Error::Compile(format!("macro `{}` has no symbol", macro_name)))?
        .clone();
    let so_path = build_plugin(&lir, &symbol, &mapping, &dep_lcls)?;
    call_plugin(&so_path, &symbol, "", args, line as i64, col as i64, file)
}

fn build_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, mapping: &[ParamKind], dep_lcls: &[String]) -> Result<PathBuf> {
    let ir = crate::lir::emit_program(lir);
    // 依赖包 IR（递归收集；与主 IR 一起参与缓存键与链接）
    let mut dep_irs: Vec<String> = Vec::new();
    for dep in dep_lcls {
        let (_s, _src, bin, _tt) = crate::package::load_package(dep)
            .map_err(|e| Error::Compile(format!("macro dep '{}': {}", dep, e)))?;
        let dlir = crate::lir::serialize::program_from_bytes(&bin)
            .map_err(|e| Error::Compile(format!("macro dep LIR decode '{}': {}", dep, e)))?;
        dep_irs.push(crate::lir::emit_program(&dlir));
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ABI_VERSION.hash(&mut hasher);
    symbol.hash(&mut hasher);
    format!("{:?}", mapping).hash(&mut hasher);
    ir.hash(&mut hasher);
    for d in &dep_irs { d.hash(&mut hasher); }
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

    // llc（PIC）：主对象 + 依赖对象（依赖对象按 IR 内容哈希缓存）
    compile_pic(&ll_path, &obj_path)?;
    let mut link_objs = vec![obj_path.clone()];
    for dep_ir in &dep_irs {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        dep_ir.hash(&mut h);
        let dhash = h.finish();
        let dep_obj = dir.join(format!("dep_{:x}.o", dhash));
        if !dep_obj.exists() {
            let dep_ll = dir.join(format!("dep_{:x}.ll", dhash));
            std::fs::write(&dep_ll, dep_ir)
                .map_err(|e| Error::Compile(format!("macro dep IR write: {}", e)))?;
            compile_pic(&dep_ll, &dep_obj)?;
        }
        link_objs.push(dep_obj);
    }

    // C shim：C ABI ↔ Ayanami String（{char*, long}）；ABI v5 带调用点（line/col/file）
    let mut locals = String::new();
    let mut call_args = String::new();
    let mut param_types: Vec<&str> = Vec::new();
    for (i, kind) in mapping.iter().enumerate() {
        if i > 0 { call_args.push_str(", "); }
        match kind {
            ParamKind::Input => {
                locals.push_str("    AyaBuf s = dup_buf(input);\n");
                call_args.push_str("s");
                param_types.push("AyaBuf");
            }
            ParamKind::Arg(n) => {
                locals.push_str(&format!("    AyaBuf a{n} = dup_buf(args.items[{n}]);\n"));
                call_args.push_str(&format!("a{n}"));
                param_types.push("AyaBuf");
            }
            ParamKind::Line => { call_args.push_str("line"); param_types.push("long"); }
            ParamKind::Col => { call_args.push_str("col"); param_types.push("long"); }
            ParamKind::File => {
                locals.push_str("    AyaBuf f = dup_buf(file);\n");
                call_args.push_str("f");
                param_types.push("AyaBuf");
            }
        }
    }
    let param_types = if param_types.is_empty() { "void".to_string() } else { param_types.join(", ") };
    let shim = format!(
        "#include <stdlib.h>\n#include <string.h>\n\
         typedef struct {{ char* data; long len; }} AyaBuf;\n\
         typedef struct {{ const AyaBuf* items; long count; }} AyaBufList;\n\
         extern AyaBuf {symbol}({param_types});\n\
         static AyaBuf dup_buf(AyaBuf b) {{\n\
             AyaBuf r; r.len = b.len;\n\
             r.data = (char*)malloc((size_t)b.len + 1);\n\
             if (r.data) {{ memcpy(r.data, b.data, (size_t)b.len); r.data[b.len] = 0; }}\n\
             return r;\n\
         }}\n\
         AyaBuf __ayanami_macro_expand(AyaBuf input, AyaBufList args, long line, long col, AyaBuf file) {{\n\
             (void)args; (void)line; (void)col; (void)file;\n\
         {locals}\
             return {symbol}({call_args});\n\
         }}\n\
         void __ayanami_macro_free(AyaBuf b) {{ free(b.data); }}\n",
        symbol = symbol,
        param_types = param_types,
        locals = locals,
        call_args = call_args
    );
    std::fs::write(&shim_path, shim).map_err(|e| Error::Compile(format!("macro shim write: {}", e)))?;

    let runtime = crate::driver::find_runtime_c()?;
    let mut gcc = Command::new("gcc");
    gcc.arg("-shared").arg("-fPIC");
    for o in &link_objs {
        gcc.arg(o);
    }
    let status = gcc
        .arg(&shim_path).arg(&runtime).arg("-lm")
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

type ExpandFn = unsafe extern "C" fn(AyaBuf, AyaBufList, i64, i64, AyaBuf) -> AyaBuf;
type FreeFn = unsafe extern "C" fn(AyaBuf);

fn call_plugin(so_path: &Path, symbol: &str, input: &str, args: &[String], line: i64, col: i64, file: &str) -> Result<String> {
    let c_path = CString::new(so_path.to_string_lossy().as_bytes())
        .map_err(|_| Error::Compile("macro path contains NUL".into()))?;
    let arg_bufs: Vec<AyaBuf> = args.iter()
        .map(|a| AyaBuf { data: a.as_ptr() as *mut u8, len: a.len() })
        .collect();
    let arg_list = AyaBufList { items: arg_bufs.as_ptr(), count: arg_bufs.len() };
    let in_buf = AyaBuf { data: input.as_ptr() as *mut u8, len: input.len() };
    let file_buf = AyaBuf { data: file.as_ptr() as *mut u8, len: file.len() };
    unsafe {
        let handle = dl::dlopen(c_path.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            let err = dl::dlerror();
            let detail = if err.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
            };
            return Err(Error::Compile(format!("dlopen failed: {}: {}", so_path.display(), detail)));
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
        let out = expand(in_buf, arg_list, line, col, file_buf);
        if out.data.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("macro `{}` returned null", symbol)));
        }
        let bytes = std::slice::from_raw_parts(out.data, out.len).to_vec();
        if !free_ptr.is_null() {
            let free: FreeFn = std::mem::transmute(free_ptr);
            free(out);
        }
        dl::dlclose(handle);
        String::from_utf8(bytes).map_err(|e| Error::Compile(format!("macro output not UTF-8: {}", e)))
    }
}
