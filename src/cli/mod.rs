use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{find_project, load_config, project_entry, resolve_path};

pub(crate) mod build;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod defs;
pub(crate) mod fmt;
pub(crate) mod new;
pub(crate) mod package_install;

pub(crate) use build::{cmd_build, cmd_run};
pub(crate) use check::cmd_check;
pub(crate) use clean::cmd_clean;
pub(crate) use defs::cmd_defs;
pub(crate) use fmt::cmd_fmt;
pub(crate) use new::cmd_new;
pub(crate) use package_install::{cmd_install, cmd_package};

/// A2：提取 `--release` 标志（其余参数保持顺序）。
pub(crate) fn split_release(args: &[String]) -> (Vec<String>, bool) {
    let mut rest = Vec::new();
    let mut release = false;
    for a in args {
        if a == "--release" {
            release = true;
        } else {
            rest.push(a.clone());
        }
    }
    (rest, release)
}

/// 设置编译模式（进程级）。
pub(crate) fn apply_mode(release: bool) {
    ayanami::hir::contracts::set_release(release);
}
