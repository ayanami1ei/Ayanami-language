//! import 路径解析：本地路径 → `.lcl` → `.aya` → 可执行文件旁 `std/`（从 collect_import.rs 拆出）。

/// 解析 import 路径：本地文件、`.lcl`/`.aya` 扩展、可执行文件旁的 `std/` 目录。
pub(crate) fn resolve_import_pkg_path(path: &str) -> String {
    if std::path::Path::new(path).exists() {
        return path.to_string();
    }
    // 尝试 `.lcl` 扩展
    let with_lcl = format!("{}.lcl", path);
    if std::path::Path::new(&with_lcl).exists() {
        return with_lcl;
    }
    // 尝试 `.aya` 扩展
    let with_aya = if path.ends_with(".aya") {
        path.to_string()
    } else {
        format!("{}.aya", path)
    };
    if std::path::Path::new(&with_aya).exists() {
        return with_aya;
    }
    // 标准库目录（取文件名，去掉目录前缀）
    let Some(exe_dir) = std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf())) else {
        return path.to_string();
    };
    let stem = std::path::Path::new(path).file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let std_candidates = [
        exe_dir.join("std").join(format!("{}.lcl", stem)),
        exe_dir.join("../std").join(format!("{}.lcl", stem)),
        exe_dir.join("std").join(format!("{}.aya", stem)),
        exe_dir.join("../std").join(format!("{}.aya", stem)),
    ];
    for candidate in &std_candidates {
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    path.to_string()
}
