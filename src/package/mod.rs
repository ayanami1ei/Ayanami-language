use crate::parser::ast::{Stmt, Type};

/// What kind of artifact this package can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Executable,
    StaticLib,
    DynamicLib,
}

impl TargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetType::Executable => "executable",
            TargetType::StaticLib => "static-lib",
            TargetType::DynamicLib => "dynamic-lib",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "executable" => Some(TargetType::Executable),
            "static-lib" => Some(TargetType::StaticLib),
            "dynamic-lib" => Some(TargetType::DynamicLib),
            _ => None,
        }
    }
}

/// Compiled package (.lcl) containing public symbols, generic source (reserved), LIR, and target metadata.
pub struct Package {
    pub name: String,
    pub version: String,
    pub target_types: Vec<TargetType>,
    pub symbols: Vec<PackageSymbol>,
    pub generic_sources: Vec<String>,
    pub lir_data: Vec<u8>,
}

#[derive(Debug, Clone)]
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
}

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            target_types: Vec::new(),
            symbols: Vec::new(),
            generic_sources: Vec::new(),
            lir_data: Vec::new(),
        }
    }

    /// Collect public symbols from top-level AST statements.
    pub fn collect_symbols(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.collect_stmt_symbols(stmt);
        }
    }

    fn collect_stmt_symbols(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FnDecl { vis, name, params, return_type, .. } => {
                if vis.is_public() {
                    let sig = format!("{}({})->{}",
                        name,
                        params.iter().map(|(_, t)| type_to_string(t)).collect::<Vec<_>>().join(","),
                        type_to_string(return_type));
                    self.symbols.push(PackageSymbol::Fn {
                        name: name.as_str().to_string(),
                        signature: sig,
                    });
                }
            }
            Stmt::StructDef { vis, name, .. } => {
                if vis.is_public() {
                    self.symbols.push(PackageSymbol::Struct {
                        name: name.as_str().to_string(),
                    });
                }
            }
            Stmt::Namespace { vis, name, items, .. } => {
                if vis.is_public() {
                    self.symbols.push(PackageSymbol::Namespace {
                        name: name.as_str().to_string(),
                    });
                }
                // Collect symbol from inside namespace too
                for item in items {
                    self.collect_stmt_symbols(item);
                }
            }
            _ => {}
        }
    }

    /// Serialize the package to .lcl format bytes.
    /// Format: binary header + UTF-8 INI body.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Header: magic "LCL1" + version u32 + flags u32
        buf.extend_from_slice(b"LCL1");
        buf.extend_from_slice(&1u32.to_le_bytes()); // version
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags

        // Body: INI-style sections
        let mut body = String::new();
        body.push_str("[pakage-info]\n");
        body.push_str(&format!("name=\"{}\"\n", self.name));
        body.push_str(&format!("version=\"{}\"\n", self.version));
        body.push_str("\n");

        if !self.target_types.is_empty() {
            body.push_str("[target]\n");
            for tt in &self.target_types {
                body.push_str(&format!("type=\"{}\"\n", tt.as_str()));
            }
            body.push_str("\n");
        }

        if !self.symbols.is_empty() {
            body.push_str("[symbols]\n");
            for sym in &self.symbols {
                match sym {
                    PackageSymbol::Fn { name, signature } => {
                        // signature format: "name(param_types...)->ret_type"
                        body.push_str(&format!("fn=\"{},{}\"\n", name, signature));
                    }
                    PackageSymbol::Struct { name } => {
                        body.push_str(&format!("struct=\"{}\"\n", name));
                    }
                    PackageSymbol::Namespace { name } => {
                        body.push_str(&format!("namespace=\"{}\"\n", name));
                    }
                }
            }
            body.push_str("\n");
        }

        // Generic source code slot (reserved for future)
        body.push_str("[generics]\n");
        for src in &self.generic_sources {
            body.push_str(&format!("source=\"{}\"\n", src));
        }
        body.push_str("\n");

        // LIR metadata (empty, binary follows after marker)
        body.push_str("[lir]\n");
        body.push_str("\n");
        buf.extend_from_slice(body.as_bytes());
        buf.extend_from_slice(b"===LIR===\n");
        buf.extend_from_slice(&self.lir_data);
        buf
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_bytes())
            .map_err(|e| format!("failed to write package: {}", e))
    }
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
}

/// Parse a .lcl file and return the imported symbols, generic sources, LIR (binary), and target types.
pub fn load_package(path: &str) -> Result<(Vec<ImportedSymbol>, Vec<String>, Vec<u8>, Vec<TargetType>), String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("failed to read package '{}': {}", path, e))?;

    // Skip 12-byte binary header
    let body = if data.len() >= 12 {
        std::str::from_utf8(&data[12..]).map_err(|e| format!("invalid UTF-8 in package: {}", e))?
    } else {
        return Err("invalid package: too short".into());
    };

    // Split at ===LIR=== marker to separate INI body from binary LIR
    // The marker is at byte offset `pos` within `body` (after the 12-byte header).
    // Its position in the raw `data` is `pos + 12`.
    let body_str = body;
    let (ini_body, lir_binary) = if let Some(pos) = body_str.find("===LIR===\n") {
        let lir_start = pos + 10; // skip "===LIR===\n"
        let data_offset = 12 + lir_start;
        (&body_str[..pos], data[data_offset..].to_vec())
    } else {
        (body_str, Vec::new())
    };

    let mut symbols = Vec::new();
    let mut sources = Vec::new();
    let mut target_types = Vec::new();
    let mut in_symbols = false;
    let mut in_generics = false;
    let mut in_target = false;

    for line in ini_body.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_symbols = line.starts_with("[symbols]");
            in_generics = line.starts_with("[generics]");
            in_target = line.starts_with("[target]");
            continue;
        }
        if in_symbols {
            if let Some(rest) = line.strip_prefix("fn=") {
                let val = parse_ini_value(rest);
                if let Some((name, sig)) = val.split_once(',') {
                    symbols.push(ImportedSymbol::Fn { name: name.to_string(), sig: sig.to_string() });
                }
            } else if let Some(rest) = line.strip_prefix("struct=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Struct { name: val });
            } else if let Some(rest) = line.strip_prefix("namespace=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Namespace { name: val });
            }
        } else if in_generics {
            if let Some(rest) = line.strip_prefix("source=") {
                sources.push(parse_ini_value(rest));
            }
        } else if in_target {
            if let Some(rest) = line.strip_prefix("type=") {
                let val = parse_ini_value(rest);
                if let Some(tt) = TargetType::from_str(&val) {
                    target_types.push(tt);
                }
            }
        }
    }

    Ok((symbols, sources, lir_binary, target_types))
}

fn parse_ini_value(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Array(inner, _) => format!("[{}]", type_to_string(inner)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string(inner)),
        Type::Shared(inner, _) => format!("shared {}", type_to_string(inner)),
        Type::Weak(inner, _) => format!("weak {}", type_to_string(inner)),
        Type::Self_(_) => "Self".into(),
    }
}
