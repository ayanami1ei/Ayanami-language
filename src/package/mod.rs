use crate::parser::ast::{Stmt, Type};

/// Compiled package (.lcl) containing public symbols, generic source (reserved), and LIR.
pub struct Package {
    pub name: String,
    pub version: String,
    pub symbols: Vec<PackageSymbol>,
    pub generic_sources: Vec<String>,
    pub lir_data: String,
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
            symbols: Vec::new(),
            generic_sources: Vec::new(),
            lir_data: String::new(),
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

        if !self.symbols.is_empty() {
            body.push_str("[symbols]\n");
            for sym in &self.symbols {
                match sym {
                    PackageSymbol::Fn { name, signature } => {
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

        // LIR data
        body.push_str("[lir]\n");
        body.push_str(&self.lir_data);
        body.push_str("\n");

        buf.extend_from_slice(body.as_bytes());
        buf
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_bytes())
            .map_err(|e| format!("failed to write package: {}", e))
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
