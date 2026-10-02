use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::ast::*;

use super::check::check_hir_returns;
use super::import::{load_config_for_file, resolve_import_path};
use super::CompiledFile;

/// Compile a .aya source file with recursive import resolution.

mod compile;
mod deps;
mod lir;
mod package;
mod target;

pub use target::build_source_with_target;
pub use compile::compile_file;
pub use package::{install_package, package_source};
pub use target::run_executable;

pub fn build_source(src_path: &str, code: &str) -> Result<()> {
    build_source_to(src_path, code, "build")
}

/// Build with explicit output directory, using recursive import compilation.
pub fn build_source_to(src_path: &str, _code: &str, out_dir: &str) -> Result<()> {
    build_source_with_target(src_path, _code, out_dir, None)
}
