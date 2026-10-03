//! A5d-3b：MIR 只读检查注解（`#[check]`）调用与诊断收集。
//!
//! 与 pass 共用同一插件桥接；检查函数必须保持 blob 不变（只读），
//! 通过 shim 提供的 `__ayanami_diag_emit` 通道上报诊断，编译器 `__ayanami_diag_take` 读取。

use std::ffi::CString;
use std::path::Path;

use super::pass_plugin::build_for;
use super::plugin::{dl, AyaBuf, AyaBufList, RTLD_NOW};
use crate::error::{Error, Result};

type RunFn = unsafe extern "C" fn(AyaBuf, AyaBufList) -> AyaBuf;
type TakeFn = unsafe extern "C" fn() -> AyaBuf;
type FreeFn = unsafe extern "C" fn(AyaBuf);

pub(super) fn invoke_check(lcl_path: &str, check_name: &str, blob: &[u8]) -> Result<(Vec<u8>, Vec<(i64, String)>)> {
    let (so_path, symbol) = build_for(lcl_path, check_name, false)?;
    call_check(&so_path, &symbol, blob)
}

fn read_i64(buf: &[u8], off: usize) -> i64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&buf[off..off + 8]);
    i64::from_le_bytes(b)
}

fn parse_diags(buf: &[u8]) -> Vec<(i64, String)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 16 <= buf.len() {
        let level = read_i64(buf, pos);
        let len = read_i64(buf, pos + 8).max(0) as usize;
        pos += 16;
        if pos + len > buf.len() { break; }
        let msg = String::from_utf8_lossy(&buf[pos..pos + len]).into_owned();
        pos += len;
        out.push((level, msg));
    }
    out
}

fn call_check(so_path: &Path, symbol: &str, blob: &[u8]) -> Result<(Vec<u8>, Vec<(i64, String)>)> {
    let c_path = CString::new(so_path.to_string_lossy().as_bytes())
        .map_err(|_| Error::Compile("check path contains NUL".into()))?;
    let in_buf = AyaBuf { data: blob.as_ptr() as *mut u8, len: blob.len() };
    let empty = AyaBufList { items: std::ptr::null(), count: 0 };
    unsafe {
        let handle = dl::dlopen(c_path.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            let err = dl::dlerror();
            let detail = if err.is_null() { String::new() } else {
                std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
            };
            return Err(Error::Compile(format!("check dlopen failed: {}: {}", so_path.display(), detail)));
        }
        let run_sym = CString::new("__ayanami_pass_run").unwrap();
        let take_sym = CString::new("__ayanami_diag_take").unwrap();
        let free_sym = CString::new("__ayanami_pass_free").unwrap();
        let run_ptr = dl::dlsym(handle, run_sym.as_ptr());
        let take_ptr = dl::dlsym(handle, take_sym.as_ptr());
        let free_ptr = dl::dlsym(handle, free_sym.as_ptr());
        if run_ptr.is_null() || take_ptr.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("check `{}`: missing plugin entry", symbol)));
        }
        let run: RunFn = std::mem::transmute(run_ptr);
        let take: TakeFn = std::mem::transmute(take_ptr);
        let free: Option<FreeFn> = if free_ptr.is_null() { None } else { Some(std::mem::transmute(free_ptr)) };

        let out = run(in_buf, empty);
        let diag = take();
        if out.data.is_null() {
            dl::dlclose(handle);
            return Err(Error::Compile(format!("check `{}` returned null", symbol)));
        }
        let bytes = std::slice::from_raw_parts(out.data, out.len).to_vec();
        let diag_bytes = if diag.data.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(diag.data, diag.len).to_vec()
        };
        if let Some(f) = free {
            f(out);
            if !diag.data.is_null() { f(diag); }
        }
        dl::dlclose(handle);
        Ok((bytes, parse_diags(&diag_bytes)))
    }
}
