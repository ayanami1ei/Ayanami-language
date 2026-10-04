use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::{load_config, resolve_path};

pub(crate) mod build;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod defs;
pub(crate) mod fmt;
pub(crate) mod new;
pub(crate) mod types;
pub(crate) mod package_install;

pub(crate) use build::{cmd_build, cmd_run};
pub(crate) use check::cmd_check;
pub(crate) use clean::cmd_clean;
pub(crate) use defs::cmd_defs;
pub(crate) use fmt::cmd_fmt;
pub(crate) use new::cmd_new;
pub(crate) use types::cmd_types;
pub(crate) use package_install::{cmd_install, cmd_package};

/// 编译器 CLI 标志。
pub(crate) struct CliFlags {
    pub release: bool,
    pub verify_effects: bool,
}

/// A2/A3：提取标志（`--release` / `--verify-effects`），其余参数保持顺序。
pub(crate) fn split_flags(args: &[String]) -> (Vec<String>, CliFlags) {
    let mut rest = Vec::new();
    let mut flags = CliFlags { release: false, verify_effects: false };
    for a in args {
        match a.as_str() {
            "--release" => flags.release = true,
            "--verify-effects" => flags.verify_effects = true,
            _ => rest.push(a.clone()),
        }
    }
    (rest, flags)
}

/// 设置编译模式（进程级）。
pub(crate) fn apply_mode(flags: &CliFlags) {
    ayanami::hir::contracts::set_release(flags.release);
    ayanami::hir::effects::set_verify_effects(flags.verify_effects);
}
