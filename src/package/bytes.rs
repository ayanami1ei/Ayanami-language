use super::*;

impl Package {
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
                    PackageSymbol::Fn { name, signature, flags } => {
                        // 格式：name,flags,signature（flags 为 '+' 连接 tokens，无逗号）
                        body.push_str(&format!("fn=\"{},{},{}\"\n", name, flags.join("+"), signature));
                    }
                    PackageSymbol::Struct { name } => {
                        body.push_str(&format!("struct=\"{}\"\n", name));
                    }
                    PackageSymbol::Namespace { name } => {
                        body.push_str(&format!("namespace=\"{}\"\n", name));
                    }
                    PackageSymbol::Interface { name } => {
                        body.push_str(&format!("interface=\"{}\"\n", name));
                    }
                    PackageSymbol::Macro { name } => {
                        body.push_str(&format!("macro=\"{}\"\n", name));
                    }
                }
            }
            body.push_str("\n");
        }

        body.push_str("[generics]\n");
        for src in &self.generic_sources {
            let escaped = src.replace('\\', "\\\\").replace('\n', "\\n").replace('"', "\\\"");
            body.push_str(&format!("source=\"{}\"\n", escaped));
        }
        body.push_str("\n");

        // A5b-3：依赖包 stem（宏插件构建时递归解析并链接）
        if !self.deps.is_empty() {
            body.push_str("[deps]\n");
            for d in &self.deps {
                body.push_str(&format!("import=\"{}\"\n", d));
            }
            body.push_str("\n");
        }

        // LIR metadata (empty, binary follows after marker)
        body.push_str("[lir]\n");
        body.push_str("\n");
        buf.extend_from_slice(body.as_bytes());
        buf.extend_from_slice(b"===LIR===\n");
        buf.extend_from_slice(&self.lir_data);
        buf
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        std::fs::write(path, self.to_bytes())
            .map_err(|e| Error::Package(format!("failed to write package: {}", e)))
    }
}
