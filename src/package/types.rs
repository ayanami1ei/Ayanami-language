use super::*;


/// Compiled package (.lcl) containing public symbols, generic source (reserved), LIR, and target metadata.
pub struct Package {
    pub name: String,
    pub version: String,
    pub target_types: Vec<TargetType>,
    pub symbols: Vec<PackageSymbol>,
    pub generic_sources: Vec<String>,
    pub lir_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSymbol {
    Fn {
        name: String,
        signature: String,
    },
    Struct {
        name: String,
    },
    Namespace {
        name: String,
    },
    Interface {
        name: String,
    },
}

/// Parsed symbol from a .lcl package file.
#[derive(Debug, Clone)]
pub enum ImportedSymbol {
    Fn {
        name: String,
        sig: String,
    },
    Struct {
        name: String,
    },
    Namespace {
        name: String,
    },
    Interface {
        name: String,
    },
}
