//! 交叉编译目标状态（进程级）。
//!
//! CLI 从 `ayanami.toml [build] arch` 读取三元组后设置；LIR 发射据此写 target triple，
//! llc 据此选择后端；裸机目标不链接宿主 runtime/std 机器码。
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static TARGET_TRIPLE: Mutex<Option<String>> = Mutex::new(None);
static FREESTANDING: AtomicBool = AtomicBool::new(false);

pub fn set_target_triple(triple: Option<&str>) {
    *TARGET_TRIPLE.lock().unwrap() = triple.map(|s| s.to_string());
}

pub fn target_triple() -> Option<String> {
    TARGET_TRIPLE.lock().unwrap().clone()
}

pub fn set_freestanding(on: bool) {
    FREESTANDING.store(on, Ordering::Relaxed);
}

pub fn freestanding() -> bool {
    FREESTANDING.load(Ordering::Relaxed)
}

/// 需要静态代码生成（非 PIE/无 GOT）：freestanding 或裸机三元组。
pub fn static_codegen() -> bool {
    freestanding() || target_triple().as_deref().map_or(false, is_bare_metal_triple)
}

/// 没有宿主 OS/libc 的三元组（如 aarch64-none-elf）。
pub fn is_bare_metal_triple(triple: &str) -> bool {
    triple.contains("-none-") || triple.ends_with("-none") || triple.contains("none-elf")
}

/// 解析目标三元组的第一段；未设置时用宿主架构。
/// 兼容常见 LLVM 别名（amd64/arm64），thumb/arm 归并为 arm，i386 系归并为 x86。
pub fn target_arch() -> String {
    let Some(triple) = target_triple() else {
        return std::env::consts::ARCH.to_string();
    };
    let arch = triple.split('-').next().unwrap_or("");
    if arch == "amd64" {
        "x86_64".into()
    } else if arch == "arm64" {
        "aarch64".into()
    } else if matches!(arch, "i386" | "i486" | "i586" | "i686") {
        "x86".into()
    } else if arch.starts_with("thumb") || arch.starts_with("arm") {
        "arm".into()
    } else {
        arch.to_string()
    }
}

/// 解析目标三元组的 OS 段；未设置时用宿主 OS。
/// `aarch64-none-elf` → none；`x86_64-unknown-linux-gnu` → linux；
/// `x86_64-apple-darwin` → macos。
pub fn target_os() -> String {
    let Some(triple) = target_triple() else {
        return std::env::consts::OS.to_string();
    };
    for part in triple.split('-').skip(1) {
        match part {
            "linux" | "windows" | "macos" | "none" | "unknown" | "freebsd"
            | "android" | "ios" => return part.to_string(),
            "darwin" => return "macos".into(),
            _ => {}
        }
    }
    "unknown".into()
}

/// 目标是否属于 Unix 系（裸机/unknown 不算）。
pub fn target_is_unix() -> bool {
    matches!(
        target_os().as_str(),
        "linux" | "macos" | "freebsd" | "android" | "ios"
    )
}

/// 目标架构与当前宿主不同（`run` 应拒绝）。
pub fn is_cross_target(triple: &str) -> bool {
    let arch = triple.split('-').next().unwrap_or("");
    let host = std::env::consts::ARCH;
    let same = (host == "x86_64" && (arch == "x86_64" || arch == "amd64"))
        || (host == "aarch64" && (arch == "aarch64" || arch == "arm64"));
    !same
}
