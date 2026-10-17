//! `ayanami --version` / `ayanami update`：从 GitHub Release 联网更新编译器与 std（#177）。
//!
//! Release 资产约定：`ayanami-<ver>-linux-<arch>.tar.gz`（内含自包含 `install/`）。
//! 网络走 `curl`（无新增 crate 依赖）；`--from` 支持本地 tar 或自定义 URL（测试/离线）。

use super::*;
use ayanami::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const REPO: &str = "ayanami1ei/Ayanami-language";

/// 打印版本（`env!("CARGO_PKG_VERSION")` 单一来源）
pub(crate) fn cmd_version() {
    println!("ayanami {}", env!("CARGO_PKG_VERSION"));
}

struct UpdateArgs {
    check: bool,
    std_only: bool,
    dir: Option<String>,
    from: Option<String>,
    force: bool,
}

fn parse_args(args: &[String]) -> UpdateArgs {
    let mut u = UpdateArgs { check: false, std_only: false, dir: None, from: None, force: false };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => u.check = true,
            "--std-only" => u.std_only = true,
            "--force" => u.force = true,
            "--dir" => { i += 1; u.dir = args.get(i).cloned(); }
            "--from" => { i += 1; u.from = args.get(i).cloned(); }
            other => {
                eprintln!("error: unknown update flag `{}`", other);
                eprintln!("usage: ayanami update [--check] [--std-only] [--dir <path>] [--from <url|tar>] [--force]");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    u
}

fn asset_arch() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64".into(),
        "aarch64" => "aarch64".into(),
        other => other.into(),
    }
}

/// curl 取文本（失败返回 None；离线/无 curl 时给出可读提示）
fn curl_text(url: &str) -> Option<String> {
    let out = Command::new("curl").args(["-fsSL", "--retry", "2", "--max-time", "30"]).arg(url).output().ok()?;
    if !out.status.success() { return None; }
    String::from_utf8(out.stdout).ok()
}

/// 从 GitHub API JSON 中提取 `"tag_name": "vX.Y.Z"`
fn parse_tag(json: &str) -> Option<String> {
    let key = "\"tag_name\":";
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let q1 = rest.find('"')? + 1;
    let q2 = rest[q1..].find('"')? + q1;
    Some(rest[q1..q2].trim_start_matches('v').to_string())
}

fn fetch_latest() -> Result<(String, String)> {
    let json = curl_text(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .ok_or_else(|| Error::Compile(format!(
            "无法查询最新版本（网络不可用或缺少 curl）；可手动下载: https://github.com/{REPO}/releases")))?;
    let latest = parse_tag(&json)
        .ok_or_else(|| Error::Compile("无法解析 GitHub Release 响应（tag_name 缺失）".into()))?;
    let asset = format!("ayanami-{}-linux-{}.tar.gz", latest, asset_arch());
    let url = format!("https://github.com/{REPO}/releases/download/v{latest}/{asset}");
    Ok((latest, url))
}

/// 从 tar 文件名推断版本（`ayanami-<ver>-linux-<arch>.tar.gz`）
fn version_from_name(path: &str) -> Option<String> {
    let name = std::path::Path::new(path).file_name()?.to_str()?;
    let rest = name.strip_prefix("ayanami-")?;
    let (ver, _) = rest.split_once("-linux-")?;
    Some(ver.to_string())
}

fn download(src: &str, dest: &Path) -> Result<()> {
    if src.starts_with("http://") || src.starts_with("https://") {
        let status = Command::new("curl")
            .args(["-fsSL", "--retry", "3", "-o"])
            .arg(dest)
            .arg(src)
            .status()
            .map_err(|e| Error::Compile(format!("无法执行 curl（更新需要 curl）: {}", e)))?;
        if !status.success() {
            return Err(Error::Compile(format!("下载失败: {}", src)));
        }
    } else {
        fs::copy(src, dest).map_err(|e| Error::Compile(format!("无法读取本地包 '{}': {}", src, e)))?;
    }
    Ok(())
}

fn tar_listing(tar: &Path) -> Result<String> {
    let out = Command::new("tar").arg("-tzf").arg(tar).output()
        .map_err(|e| Error::Compile(format!("无法执行 tar: {}", e)))?;
    if !out.status.success() {
        return Err(Error::Compile("tar 列表失败（包损坏？）".into()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// 目标目录安全校验：拒绝 cargo target/ 下的开发构建（避免误覆盖调试二进制）
fn ensure_target_ok(target: &Path, force: bool) -> Result<()> {
    if force {
        return Ok(());
    }
    if target.components().any(|c| c.as_os_str() == "target") {
        return Err(Error::Compile(format!(
            "目标目录 `{}` 看起来是 cargo 构建目录；如确要覆盖请加 --force（或 --dir 指定安装目录）",
            target.display())));
    }
    if !target.exists() {
        return Ok(()); // 全新安装目录
    }
    let has_bin = target.join("ayanami").exists();
    let empty = fs::read_dir(target).map(|mut d| d.next().is_none()).unwrap_or(false);
    if !has_bin && !empty {
        return Err(Error::Compile(format!(
            "目标目录 `{}` 不含 ayanami（非发布布局）；请用 --dir 指定或加 --force",
            target.display())));
    }
    Ok(())
}

pub(crate) fn cmd_update(args: &[String]) {
    let u = parse_args(args);
    let current = env!("CARGO_PKG_VERSION");

    // 版本与下载源
    let (latest, url) = match u.from.as_deref() {
        Some(src) if src.starts_with("http") => {
            (version_from_name(src).unwrap_or_else(|| "custom".into()), src.to_string())
        }
        Some(src) => (version_from_name(src).unwrap_or_else(|| "local".into()), src.to_string()),
        None => match fetch_latest() {
            Ok(v) => v,
            Err(e) => { ayanami::diagnostics::error(&e.to_string()); std::process::exit(1); }
        },
    };

    if u.check {
        println!("当前版本: v{}", current);
        println!("最新版本: v{}", latest);
        if latest == current {
            println!("已是最新");
            std::process::exit(0);
        } else {
            println!("有可用更新（ayanami update 更新）");
            std::process::exit(1);
        }
    }

    // 目标目录：--dir 或当前可执行文件所在目录
    let target = match &u.dir {
        Some(d) => PathBuf::from(d),
        None => std::env::current_exe().ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    if let Err(e) = ensure_target_ok(&target, u.force) {
        ayanami::diagnostics::error(&e.to_string());
        std::process::exit(1);
    }
    fs::create_dir_all(&target).ok();

    // 下载 + 解压到临时目录
    let tmp = unique_tmp_dir("update");
    fs::create_dir_all(&tmp).ok();
    let tar_path = tmp.join("ayanami.tar.gz");
    if let Err(e) = download(&url, &tar_path) {
        ayanami::diagnostics::error(&e.to_string());
        let _ = fs::remove_dir_all(&tmp);
        std::process::exit(1);
    }
    match tar_listing(&tar_path) {
        Ok(l) if l.contains("install/ayanami") => {}
        Ok(_) => {
            ayanami::diagnostics::error("包内容校验失败（缺少 install/ayanami）");
            let _ = fs::remove_dir_all(&tmp);
            std::process::exit(1);
        }
        Err(e) => {
            ayanami::diagnostics::error(&e.to_string());
            let _ = fs::remove_dir_all(&tmp);
            std::process::exit(1);
        }
    }
    let status = Command::new("tar").args(["-xzf"]).arg(&tar_path).arg("-C").arg(&tmp).status()
        .expect("tar extract");
    if !status.success() {
        ayanami::diagnostics::error("解压失败");
        let _ = fs::remove_dir_all(&tmp);
        std::process::exit(1);
    }
    let new_root = tmp.join("install");

    // 备份（同级 `<dir>.bak-<current>`）
    let dir_name = target.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "install".into());
    let bak = target.with_file_name(format!("{}.bak-{}", dir_name, current));
    let need_bak = target.exists() && fs::read_dir(&target).map(|mut d| d.next().is_some()).unwrap_or(false);
    if need_bak {
        if bak.exists() {
            let _ = fs::remove_dir_all(&bak);
        }
        let cp_bak = Command::new("cp").arg("-a").arg(&target).arg(&bak).status();
        if !cp_bak.map(|s| s.success()).unwrap_or(false) {
            ayanami::diagnostics::error(&format!("备份失败: {}", bak.display()));
            let _ = fs::remove_dir_all(&tmp);
            std::process::exit(1);
        }
    }

    // 覆盖：std-only 仅替换 std；全量清空后复制新内容
    let copy_result = (|| -> Result<()> {
        if u.std_only {
            let dst = target.join("std");
            let _ = fs::remove_dir_all(&dst);
            let st = Command::new("cp").arg("-a").arg(new_root.join("std")).arg(&dst).status()
                .map_err(|e| Error::Compile(format!("cp 失败: {}", e)))?;
            if !st.success() { return Err(Error::Compile("std 复制失败".into())); }
        } else {
            for entry in fs::read_dir(&target).map_err(|e| Error::Compile(e.to_string()))? {
                let p = entry.map_err(|e| Error::Compile(e.to_string()))?.path();
                let _ = if p.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
            }
            let st = Command::new("cp").arg("-a")
                .arg(format!("{}/.", new_root.display()))
                .arg(&target)
                .status()
                .map_err(|e| Error::Compile(format!("cp 失败: {}", e)))?;
            if !st.success() { return Err(Error::Compile("复制失败".into())); }
        }
        Ok(())
    })();
    if let Err(e) = copy_result {
        // 回滚
        for entry in fs::read_dir(&target).into_iter().flatten().flatten() {
            let p = entry.path();
            let _ = if p.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
        }
        let _ = Command::new("cp").arg("-a").arg(format!("{}/.", bak.display())).arg(&target).status();
        let _ = fs::remove_dir_all(&tmp);
        ayanami::diagnostics::error(&format!("更新失败（已回滚）: {}", e));
        std::process::exit(1);
    }
    let _ = fs::remove_dir_all(&tmp);

    if u.std_only {
        println!("已更新 std（v{}；备份: {}）", latest, bak.display());
    } else {
        println!("已更新: v{} → v{}（安装目录: {}；备份: {}）", current, latest, target.display(), bak.display());
    }
}
