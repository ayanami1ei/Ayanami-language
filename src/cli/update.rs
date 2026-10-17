//! `ayanami --version` / `ayanami update`：从 GitHub Release 联网更新编译器与 std（#177）。
//!
//! Release 资产约定：`ayanami-<ver>-linux-<arch>.tar.gz`（内含自包含 `install/`）。
//! 网络走 `curl`（无新增 crate 依赖）；`--from` 支持本地 tar 或自定义 URL（测试/离线）。

use super::*;
use ayanami::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;
use super::update_fetch::*;

/// 更新阶段进度（stderr；插件/日志按 `[n/m]` 解析）。
fn stage(step: usize, total: usize, msg: &str) {
    eprintln!("[{}/{}] {}", step, total, msg);
}

/// 当前构建指纹（build.rs 注入：git 短哈希 + dirty；缺失回退 "unknown"）
pub(crate) fn build_id() -> &'static str {
    option_env!("AYANAMI_BUILD_ID").unwrap_or("unknown")
}

/// 打印版本（`env!("CARGO_PKG_VERSION")` 单一来源 + 构建指纹）
pub(crate) fn cmd_version() {
    println!("ayanami {} (build {})", env!("CARGO_PKG_VERSION"), build_id());
}

struct UpdateArgs {
    check: bool,
    std_only: bool,
    dir: Option<String>,
    from: Option<String>,
    force: bool,
    /// 下载限速（curl `--limit-rate` 语法，如 `10M` / `10240k`；env AYANAMI_UPDATE_LIMIT 兜底）
    limit_rate: Option<String>,
}

fn parse_args(args: &[String]) -> UpdateArgs {
    let mut u = UpdateArgs {
        check: false, std_only: false, dir: None, from: None, force: false,
        limit_rate: std::env::var("AYANAMI_UPDATE_LIMIT").ok().filter(|v| !v.trim().is_empty()),
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => u.check = true,
            "--std-only" => u.std_only = true,
            "--force" => u.force = true,
            "--dir" => { i += 1; u.dir = args.get(i).cloned(); }
            "--from" => { i += 1; u.from = args.get(i).cloned(); }
            "--limit-rate" => { i += 1; u.limit_rate = args.get(i).cloned(); }
            other => {
                eprintln!("error: unknown update flag `{}`", other);
                eprintln!("usage: ayanami update [--check] [--std-only] [--dir <path>] [--from <url|tar>] [--limit-rate <10M|10240k>] [--force]");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    u
}

pub(crate) fn cmd_update(args: &[String]) {
    let u = parse_args(args);
    let current = env!("CARGO_PKG_VERSION");

    // 版本与下载源
    let started = std::time::Instant::now();
    let mut size_hint: Option<f64> = None;
    let (latest, url) = match u.from.as_deref() {
        Some(src) if src.starts_with("http") => {
            (version_from_name(src).unwrap_or_else(|| "custom".into()), src.to_string())
        }
        Some(src) => (version_from_name(src).unwrap_or_else(|| "local".into()), src.to_string()),
        None => {
            stage(1, 6, "查询最新版本（GitHub Release）…");
            match fetch_latest() {
                Ok((v, u, _, sz)) => { size_hint = sz; (v, u) }
                Err(e) => { ayanami::diagnostics::error(&e.to_string()); std::process::exit(1); }
            }
        }
    };

    // 同版本 respin：比对远端构建指纹（无该资产时退化为版本比较）
    let remote_build: Option<String> = if u.from.is_none() {
        fetch_latest().ok().and_then(|(_, _, bu, _)| bu).and_then(|bu| curl_text(&bu))
    } else {
        None
    };

    if u.check {
        println!("当前版本: v{} (build {})", current, build_id());
        println!("最新版本: v{}", latest);
        if let Some(rb) = &remote_build {
            println!("远端构建: {}", rb);
        }
        let same_version = latest == current;
        let same_build = match &remote_build {
            Some(rb) => rb.trim() == build_id(),
            None => true, // 无指纹资产 → 按版本判断
        };
        if same_version && same_build {
            println!("已是最新");
            std::process::exit(0);
        } else if same_version {
            println!("有可用更新（同版本修复构建，ayanami update 更新）");
            std::process::exit(1);
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
    let size_str = size_hint.map(|s| format!("（{:.1} MB）", s)).unwrap_or_default();
    let what = std::path::Path::new(&url).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| url.clone());
    stage(2, 6, &format!("下载 {}{}…", what, size_str));
    if let Err(e) = download(&url, &tar_path, u.limit_rate.as_deref()) {
        ayanami::diagnostics::error(&e.to_string());
        let _ = fs::remove_dir_all(&tmp);
        std::process::exit(1);
    }
    stage(3, 6, "校验包内容…");
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
    stage(4, 6, "解压…");
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
        stage(5, 6, &format!("备份旧版本 → {}", bak.display()));
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

    stage(6, 6, "安装…");
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
        println!("已更新: v{} → v{}（安装目录: {}；备份: {}；耗时 {:.1}s）",
            current, latest, target.display(), bak.display(), started.elapsed().as_secs_f64());
        if let Some(rate) = &u.limit_rate {
            println!("（下载限速: {}/s）", rate);
        }
    }
}
