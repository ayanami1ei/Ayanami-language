use super::*;

pub fn load_package(path: &str) -> Result<(Vec<ImportedSymbol>, Vec<String>, Vec<u8>, Vec<TargetType>)> {
    let data = std::fs::read(path)
        .map_err(|e| Error::Package(format!("failed to read package '{}': {}", path, e)))?;

    // Find ===LIR=== marker in raw bytes (before UTF-8 decoding)
    let marker = b"===LIR===\n";
    let (ini_bytes, lir_binary) = if let Some(pos) = data[12..].windows(marker.len()).position(|w| w == marker) {
        let ini_end = 12 + pos;
        let lir_start = ini_end + marker.len();
        (&data[12..ini_end], data[lir_start..].to_vec())
    } else {
        (&data[12..], Vec::new())
    };

    // Decode INI portion as UTF-8
    let body_str = std::str::from_utf8(ini_bytes)
        .map_err(|e| Error::Package(format!("invalid UTF-8 in package INI: {}", e)))?;

    let mut symbols = Vec::new();
    let mut sources = Vec::new();
    let mut target_types = Vec::new();
    let mut in_symbols = false;
    let mut in_generics = false;
    let mut in_target = false;

    for line in body_str.lines() {
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
                if let Some((name, rest)) = val.split_once(',') {
                    // 新格式 name,flags,sig；旧格式 name,sig。
                    // flags 可能为旧单字符 t/e，或 '+' 连接的 artifacts tokens。
                    let (flags, sig) = match rest.split_once(',') {
                        Some((f, s)) if !f.contains('(') && !f.contains("->") => (f, s),
                        _ => ("", rest),
                    };
                    let tokens: Vec<String> = if flags.is_empty() {
                        Vec::new()
                    } else if flags.contains('+') {
                        flags.split('+').map(|s| s.to_string()).collect()
                    } else {
                        // 旧格式可能直接是 t/e 的组合
                        flags.chars().map(|c| c.to_string()).collect()
                    };
                    symbols.push(ImportedSymbol::Fn {
                        name: name.to_string(),
                        sig: sig.to_string(),
                        flags: tokens,
                    });
                }
            } else if let Some(rest) = line.strip_prefix("struct=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Struct { name: val });
            } else if let Some(rest) = line.strip_prefix("namespace=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Namespace { name: val });
            } else if let Some(rest) = line.strip_prefix("interface=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Interface { name: val });
            } else if let Some(rest) = line.strip_prefix("macro=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Macro { name: val });
            } else if let Some(rest) = line.strip_prefix("pass=") {
                let val = parse_ini_value(rest);
                symbols.push(ImportedSymbol::Pass { name: val });
            }
        } else if in_generics {
            if let Some(rest) = line.strip_prefix("source=") {
                let val = parse_ini_value(rest);
                let unescaped = val.replace("\\n", "\n").replace("\\\\", "\\").replace("\\\"", "\"");
                sources.push(unescaped);
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

/// A5b-3：读取 .lcl 的依赖包 stem 列表（`[deps]` 段；旧包/无依赖返回空）。
pub fn load_package_deps(path: &str) -> Result<Vec<String>> {
    let data = std::fs::read(path)
        .map_err(|e| Error::Package(format!("failed to read package '{}': {}", path, e)))?;
    if data.len() <= 12 {
        return Ok(Vec::new());
    }
    let marker = b"===LIR===\n";
    let ini_bytes = match data[12..].windows(marker.len()).position(|w| w == marker) {
        Some(pos) => &data[12..12 + pos],
        None => &data[12..],
    };
    let body_str = std::str::from_utf8(ini_bytes)
        .map_err(|e| Error::Package(format!("invalid UTF-8 in package INI: {}", e)))?;
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in body_str.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line.starts_with("[deps]");
            continue;
        }
        if in_deps {
            if let Some(rest) = line.strip_prefix("import=") {
                deps.push(parse_ini_value(rest));
            }
        }
    }
    Ok(deps)
}

pub(super) fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Default | Type::FnPtr(..) => "???".into(),
        Type::Int(_) => "int".into(),
        Type::Float(_) => "float".into(),
        Type::Char(_) => "char".into(),
        Type::Bool(_) => "bool".into(),
        Type::Void(_) => "void".into(),
        Type::Named(s, _) => s.as_str().to_string(),
        Type::Array(inner, _) => format!("[{}]", type_to_string(inner)),
        Type::Unique(inner, _) => format!("unique {}", type_to_string(inner)),
        Type::Generic(name, args, _) => format!("{}[{}]", name, args.iter().map(|a| type_to_string(a)).collect::<Vec<_>>().join(",")),
        Type::Ref(inner, mutable, _) => {
            if *mutable {
                format!("ref mut {}", type_to_string(inner))
            } else {
                format!("ref {}", type_to_string(inner))
            }
        }
        Type::Self_(_) => "Self".into(),
    }
}
