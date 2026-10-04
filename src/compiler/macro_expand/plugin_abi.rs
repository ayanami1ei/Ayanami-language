//! 宏/pass 插件 ABI 类型与形参规划（ABI v5）。

use std::ffi::{c_char, c_int, c_void};

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// 插件 ABI 版本：签名变化时必须递增（参与 .so 缓存键，避免复用旧 shim）。
pub(crate) const ABI_VERSION: u32 = 5;

/// A5c-2：宏形参 → 调用值映射
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamKind {
    /// 首个 String 形参：宏输入（attribute 宏为被标注项源码；函数宏为空）
    Input,
    /// 普通 String 形参：第 i 个实参源码
    Arg(usize),
    /// 保留形参 `__line` / `__col` / `__file`：调用点信息
    Line,
    Col,
    File,
}

pub(crate) mod dl {
    use super::*;
    #[link(name = "dl")]
    unsafe extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> c_int;
        pub fn dlerror() -> *mut c_char;
    }
}
pub(crate) const RTLD_NOW: c_int = 2;

/// ABI v3：文本缓冲（与 Ayanami String 布局一致，减少裸指针参数）
#[repr(C)]
pub(crate) struct AyaBuf {
    pub data: *mut u8,
    pub len: usize,
}

/// ABI v3：文本缓冲数组（宏/pass 实参）
#[repr(C)]
pub(crate) struct AyaBufList {
    pub items: *const AyaBuf,
    pub count: usize,
}


pub(crate) fn load_macro_fn(lcl_path: &str, macro_name: &str) -> Result<(crate::lir::ir::LirFn, crate::lir::ir::LirProgram)> {
    let (_syms, _src, lir_binary, _tt) = crate::package::load_package(lcl_path)
        .map_err(|e| Error::Compile(format!("macro package '{}': {}", lcl_path, e)))?;
    let lir = crate::lir::serialize::program_from_bytes(&lir_binary)
        .map_err(|e| Error::Compile(format!("macro LIR decode: {}", e)))?;
    let f = lir.functions.iter().find(|f| f.name.as_str() == macro_name)
        .cloned()
        .ok_or_else(|| Error::Compile(format!("macro `{}` has no compiled body in `{}`", macro_name, lcl_path)))?;
    Ok((f, lir))
}

/// 校验宏形参并生成映射；`allow_reserved` 时支持 `__line/__col/__file`
pub(crate) fn plan_params(f: &crate::lir::ir::LirFn, args_len: usize, allow_reserved: bool) -> Result<Vec<ParamKind>> {
    let string_ty = crate::hir::ty::HirType::Named(crate::intern::Symbol::intern("String"));
    let int_ty = crate::hir::ty::HirType::Int;
    if f.params.is_empty() {
        if args_len == 0 { return Ok(Vec::new()); }
        return Err(Error::Compile(format!(
            "macro `{}` takes no parameters but {} argument(s) given", f.name, args_len
        )));
    }
    let mut out = Vec::new();
    let mut next_arg = 0usize;
    for (idx, (name, ty)) in f.params.iter().enumerate() {
        if idx == 0 {
            if ty != &string_ty {
                return Err(Error::Compile(format!(
                    "macro `{}`: first parameter must be `String input`", f.name
                )));
            }
            out.push(ParamKind::Input);
            continue;
        }
        if allow_reserved && ty == &int_ty && name.as_str() == "__line" {
            out.push(ParamKind::Line);
        } else if allow_reserved && ty == &int_ty && name.as_str() == "__col" {
            out.push(ParamKind::Col);
        } else if allow_reserved && ty == &string_ty && name.as_str() == "__file" {
            out.push(ParamKind::File);
        } else if ty == &string_ty {
            out.push(ParamKind::Arg(next_arg));
            next_arg += 1;
        } else {
            return Err(Error::Compile(format!(
                "macro `{}`: unsupported parameter `{}` (expected String, or reserved __line/__col/__file)",
                f.name, name
            )));
        }
    }
    if next_arg != args_len {
        return Err(Error::Compile(format!(
            "macro `{}` takes {} argument(s) but {} given", f.name, next_arg, args_len
        )));
    }
    Ok(out)
}

/// llc PIC 编译单个 .ll → .o
pub(crate) fn compile_pic(ll_path: &Path, obj_path: &Path) -> Result<()> {
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
pub(crate) fn resolve_dep_lcls(lcl_path: &str) -> Result<Vec<String>> {
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

