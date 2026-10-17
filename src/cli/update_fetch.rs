//! `ayanami update` 的抓取/校验/安装辅助（从 update.rs 拆出，保持文件 ≤300 行）。

use super::*;
use ayanami::error::{Error, Result};
use std::path::Path;
use std::process::Command;

const REPO: &str = "ayanami1ei/Ayanami-language";

pub(super) fn asset_arch() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64".into(),
        "aarch64" => "aarch64".into(),
        other => other.into(),
    }
}

/// curl 取文本（失败返回 None；离线/无 curl 时给出可读提示）
pub(super) fn curl_text(url: &str) -> Option<String> {
    let out = Command::new("curl").args(["-fsSL", "--retry", "2", "--max-time", "30"]).arg(url).output().ok()?;
    if !out.status.success() { return None; }
    String::from_utf8(out.stdout).ok()
}

/// 从 GitHub API JSON 中提取 `"tag_name": "vX.Y.Z"`
pub(super) fn parse_tag(json: &str) -> Option<String> {
    let key = "\"tag_name\":";
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let q1 = rest.find('"')? + 1;
    let q2 = rest[q1..].find('"')? + q1;
    Some(rest[q1..q2].trim_start_matches('v').to_string())
}

/// 查询最新 Release：版本、tar 资产 URL、构建指纹资产 URL（可能缺失）
pub(super) fn fetch_latest() -> Result<(String, String, Option<String>, Option<f64>)> {
    let json = curl_text(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .ok_or_else(|| Error::Compile(format!(
            "无法查询最新版本（网络不可用或缺少 curl）；可手动下载: https://github.com/{REPO}/releases")))?;
    let latest = parse_tag(&json)
        .ok_or_else(|| Error::Compile("无法解析 GitHub Release 响应（tag_name 缺失）".into()))?;
    let asset = format!("ayanami-{}-linux-{}.tar.gz", latest, asset_arch());
    let url = format!("https://github.com/{REPO}/releases/download/v{latest}/{asset}");
    // 构建指纹资产（同版本 respin 检测）：ayanami-<ver>-build.txt
    let build_name = format!("ayanami-{}-build.txt", latest);
    let build_url = json.contains(&build_name)
        .then(|| format!("https://github.com/{REPO}/releases/download/v{latest}/{build_name}"));
    let size_mb = asset_size_mb(&json, &asset);
    Ok((latest, url, build_url, size_mb))
}

/// 资产名 → 大小（MB，1 位小数）；解析失败返回 None。
pub(super) fn asset_size_mb(json: &str, asset_name: &str) -> Option<f64> {
    let idx = json.find(asset_name)?;
    let rest = &json[idx..];
    let si = rest.find("\"size\":")? + "\"size\":".len();
    let rest2 = rest[si..].trim_start();
    let end = rest2.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest2.len());
    rest2[..end].parse::<u64>().ok().map(|b| b as f64 / (1024.0 * 1024.0))
}

/// 从 tar 文件名推断版本（`ayanami-<ver>-linux-<arch>.tar.gz`）
pub(super) fn version_from_name(path: &str) -> Option<String> {
    let name = std::path::Path::new(path).file_name()?.to_str()?;
    let rest = name.strip_prefix("ayanami-")?;
    let (ver, _) = rest.split_once("-linux-")?;
    Some(ver.to_string())
}

pub(super) fn download(src: &str, dest: &Path, limit_rate: Option<&str>) -> Result<()> {
    if src.starts_with("http://") || src.starts_with("https://") {
        // 交互终端显示 curl 进度条；非 TTY（插件/日志）静默，由阶段信息报告
        let tty = std::io::IsTerminal::is_terminal(&std::io::stderr());
        let mut cmd = Command::new("curl");
        if tty {
            cmd.args(["-fL", "--retry", "3", "--progress-bar", "-o"]).arg(dest);
        } else {
            cmd.args(["-fsSL", "--retry", "3", "-o"]).arg(dest);
        }
        if let Some(rate) = limit_rate {
            cmd.arg("--limit-rate").arg(rate);
        }
        let status = cmd
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

pub(super) fn tar_listing(tar: &Path) -> Result<String> {
    let out = Command::new("tar").arg("-tzf").arg(tar).output()
        .map_err(|e| Error::Compile(format!("无法执行 tar: {}", e)))?;
    if !out.status.success() {
        return Err(Error::Compile("tar 列表失败（包损坏？）".into()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// 目标目录安全校验：拒绝 cargo target/ 下的开发构建（避免误覆盖调试二进制）
pub(super) fn ensure_target_ok(target: &Path, force: bool) -> Result<()> {
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

