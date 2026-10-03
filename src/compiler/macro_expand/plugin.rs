//! A5b-2/A5b-3：宏插件构建（llc PIC + C shim + runtime）与 dlopen 调用（ABI v3，结构体封装）。

use std::ffi::{CString, c_char, c_int, c_void};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// 插件 ABI 版本：签名变化时必须递增（参与 .so 缓存键，避免复用旧 shim）。
pub(super) const ABI_VERSION: u32 = 4;

pub(super) mod dl {
    use super::*;
    #[link(name = "dl")]
    unsafe extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> c_int;
        pub fn dlerror() -> *mut c_char;
    }
}
pub(super) const RTLD_NOW: c_int = 2;

/// ABI v3：文本缓冲（与 Ayanami String 布局一致，减少裸指针参数）
#[repr(C)]
pub(super) struct AyaBuf {
    pub data: *mut u8,
    pub len: usize,
}

/// ABI v3：文本缓冲数组（宏/pass 实参）
#[repr(C)]
pub(super) struct AyaBufList {
    pub items: *const AyaBuf,
    pub count: usize,
}

// ═══════════════════════════════════════════════════════════════════
//  插件构建与调用
// ═══════════════════════════════════════════════════════════════════

pub(super) fn invoke_plugin(lcl_path: &str, macro_name: &str, input: &str, args: &[String]) -> Result<String> {
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
    let string_ty = crate::hir::ty::HirType::Named(crate::intern::Symbol::intern("String"));
    // 约定：fn(String input, String a1..aN) -> String；无实参时也允许 fn() -> String
    let signature_ok = f.return_type == string_ty
        && f.params.iter().all(|(_, t)| t == &string_ty)
        && (arity == args.len() + 1 || (args.is_empty() && arity == 0));
    if !signature_ok {
        return Err(Error::Compile(format!(
            "macro `{}` must have signature fn(String input, String ...args) -> String \
             ({} arg(s) passed), found {} param(s)",
            macro_name, args.len(), arity
        )));
    }

    let dep_lcls = resolve_dep_lcls(lcl_path)?;
    let so_path = build_plugin(&lir, &symbol, arity, &dep_lcls)?;
    call_plugin(&so_path, &symbol, input, args)
}

/// llc PIC 编译单个 .ll → .o
pub(super) fn compile_pic(ll_path: &Path, obj_path: &Path) -> Result<()> {
    let (llc, llc_dir) = crate::driver::find_llc()?;
    let mut cmd = Command::new(&llc);
    cmd.arg("-filetype=obj").arg("-relocation-model=pic")
        .arg("-o").arg(obj_path).arg(ll_path);
    if !llc_dir.as_os_str().is_empty() {
        cmd.env("LD_LIBRARY_PATH", llc_dir.to_string_lossy().as_ref());
    }
    if !cmd.status().map_err(|e| Error::Compile(format!("macro llc: {}", e)))?.success() {
        return Err(Error::Compile("macro llc failed".into()));
    }
    Ok(())
}

/// A5b-3：递归解析宏库依赖的 .lcl（`.lcl` 的 `[deps]` 段按 stem 记录）。
/// 每个 stem 依次在引用方目录、std 目录、cwd 查找；返回去重后的依赖路径。
pub(super) fn resolve_dep_lcls(lcl_path: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![PathBuf::from(lcl_path)];
    while let Some(p) = queue.pop() {
        let dir = p.parent().map(|d| d.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
        let deps = crate::package::load_package_deps(&p.to_string_lossy()).unwrap_or_default();
        for stem in deps {
            let Some(dp) = resolve_dep(&stem, &dir) else {
                return Err(Error::Compile(format!(
                    "macro dependency `{}` not found for `{}`", stem, p.display()
                )));
            };
            if seen.insert(dp.clone()) {
                out.push(dp.to_string_lossy().into_owned());
                queue.push(dp);
            }
        }
    }
    Ok(out)
}

fn resolve_dep(stem: &str, dir: &Path) -> Option<PathBuf> {
    let local = dir.join(format!("{}.lcl", stem));
    if local.exists() {
        return Some(local);
    }
    if let Some(std) = crate::compiler::find_std_dir() {
        let p = std.join(format!("{}.lcl", stem));
        if p.exists() {
            return Some(p);
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let p = cwd.join(format!("{}.lcl", stem));
    if p.exists() { Some(p) } else { None }
}

fn build_plugin(lir: &crate::lir::ir::LirProgram, symbol: &str, arity: usize, dep_lcls: &[String]) -> Result<PathBuf> {
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
    arity.hash(&mut hasher);
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

    // C shim：C ABI ↔ Ayanami String（{char*, long}）；ABI v3 用结构体封装
    let mut locals = String::new();
    let mut call_args = String::new();
    if arity >= 1 {
        locals.push_str("    AyaBuf s = dup_buf(input);\n");
        call_args.push_str("s");
        for i in 0..(arity - 1) {
            locals.push_str(&format!("    AyaBuf a{i} = dup_buf(args.items[{i}]);\n"));
            call_args.push_str(&format!(", a{i}"));
        }
    }
    let param_types = if arity == 0 { "void".to_string() } else { vec!["AyaBuf"; arity].join(", ") };
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
         AyaBuf __ayanami_macro_expand(AyaBuf input, AyaBufList args) {{\n\
             (void)args;\n\
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
        .arg(&shim_path).arg(&runtime)
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

type ExpandFn = unsafe extern "C" fn(AyaBuf, AyaBufList) -> AyaBuf;
type FreeFn = unsafe extern "C" fn(AyaBuf);

fn call_plugin(so_path: &Path, symbol: &str, input: &str, args: &[String]) -> Result<String> {
    let c_path = CString::new(so_path.to_string_lossy().as_bytes())
        .map_err(|_| Error::Compile("macro path contains NUL".into()))?;
    let arg_bufs: Vec<AyaBuf> = args.iter()
        .map(|a| AyaBuf { data: a.as_ptr() as *mut u8, len: a.len() })
        .collect();
    let arg_list = AyaBufList { items: arg_bufs.as_ptr(), count: arg_bufs.len() };
    let in_buf = AyaBuf { data: input.as_ptr() as *mut u8, len: input.len() };
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
        let out = expand(in_buf, arg_list);
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
