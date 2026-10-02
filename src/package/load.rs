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
                    // 新格式 name,flags,sig；旧格式 name,sig（无 flags）。
                    // flags 只可能由 t/e 组成，否则视为旧格式。
                    let (flags, sig) = match rest.split_once(',') {
                        Some((f, s)) if f.chars().all(|c| c == 't' || c == 'e') => (f, s),
                        _ => ("", rest),
                    };
                    symbols.push(ImportedSymbol::Fn {
                        name: name.to_string(),
                        sig: sig.to_string(),
                        no_throws: flags.contains('t'),
                        no_effects: flags.contains('e'),
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
