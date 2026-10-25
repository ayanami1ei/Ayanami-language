pub mod config;

use crate::error::{Error, Result};
use crate::parser::ast::{Stmt, Type};

mod annotate;
mod bytes;
pub(crate) mod const_codec;
mod load;
mod symbols;
mod target;
mod types;

pub use load::{load_package, load_package_deps, resolve_package_deps, warn_version_drift};
pub use target::TargetType;
pub use types::{ImportedSymbol, Package, PackageSymbol};
