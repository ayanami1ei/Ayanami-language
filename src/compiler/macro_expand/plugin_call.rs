//! 宏插件调用与诊断读取（从 plugin.rs 拆出，保持文件 ≤300 行）。
use super::*;
use super::plugin_abi::{AyaBuf, AyaBufList, RTLD_NOW, dl};
use std::ffi::CString;
use std::path::Path;

type ExpandFn = unsafe extern "C" fn(AyaBuf, AyaBufList, i64, i64, AyaBuf) -> AyaBuf;
type FreeFn = unsafe extern "C" fn(AyaBuf);

pub(super) fn call_plugin(so_path: &Path, symbol: &str, input: &[u8], args: &[Vec<u8>], line: i64, col: i64, file: &[u8]) -> Result<Vec<u8>> {
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
        // mcl.2：读取宏诊断（warn/error）
        let mut diags: Vec<(i64, String)> = Vec::new();
        let cnt_ptr = dl::dlsym(handle, CString::new("__ayanami_macro_diag_count").unwrap().as_ptr());
        if !cnt_ptr.is_null() {
            let count_fn: unsafe extern "C" fn() -> i64 = std::mem::transmute(cnt_ptr);
            let level_ptr = dl::dlsym(handle, CString::new("__ayanami_macro_diag_level").unwrap().as_ptr());
            let msg_ptr = dl::dlsym(handle, CString::new("__ayanami_macro_diag_msg").unwrap().as_ptr());
            let level_fn: Option<unsafe extern "C" fn(i64) -> i64> = (!level_ptr.is_null()).then(|| std::mem::transmute(level_ptr));
            let msg_fn: Option<unsafe extern "C" fn(i64) -> AyaBuf> = (!msg_ptr.is_null()).then(|| std::mem::transmute(msg_ptr));
            let n = count_fn();
            for i in 0..n {
                let level = level_fn.map(|f| f(i)).unwrap_or(0);
                let msg = msg_fn.map(|f| f(i)).unwrap_or(AyaBuf { data: std::ptr::null_mut(), len: 0 });
                let text = if msg.data.is_null() {
                    String::new()
                } else {
                    let t = String::from_utf8_lossy(std::slice::from_raw_parts(msg.data, msg.len)).into_owned();
                    if !free_ptr.is_null() {
                        let free: FreeFn = std::mem::transmute(free_ptr);
                        free(msg);
                    }
                    t
                };
                diags.push((level, text));
            }
        }
        dl::dlclose(handle);
        // mcl.2：诊断带调用点位置（line/col 为 0 时省略）
        let at = if line > 0 { format!(" at {}:{}", line, col) } else { String::new() };
        for (level, msg) in &diags {
            if *level >= 2 {
                return Err(Error::Compile(format!("{} (in macro `{}`{})", msg, symbol, at)));
            }
            eprintln!("warning: {} (in macro `{}`{})", msg, symbol, at);
        }
        Ok(bytes)
    }
}
