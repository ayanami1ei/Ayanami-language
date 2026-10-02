use super::*;

pub fn type_from_node(node: &Node) -> Result<Type> {
    match node.kind.as_str() {
        "IntLiteral" => Ok(Type::Int(default_span())),
        "TypeBase" => {
            let name_str = node.get("name").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .ok_or_else(|| Error::Parse("missing type name".into()))?;
            match name_str.as_str() {
                "int" => Ok(Type::Int(default_span())),
                "float" => Ok(Type::Float(default_span())),
                "char" => Ok(Type::Char(default_span())),
                "bool" => Ok(Type::Bool(default_span())),
                "void" => Ok(Type::Void(default_span())),
                _ => Ok(Type::Named(Symbol::intern(&name_str), default_span())),
            }
        }
        "UniqueType" => {
            let inner = type_from_node(node.child("inner").ok_or_else(|| Error::Parse("missing inner".into()))?)?;
            Ok(Type::Unique(Box::new(inner), default_span()))
        }
        "ArrayType" => {
            let inner = type_from_node(node.child("inner").ok_or_else(|| Error::Parse("missing inner".into()))?)?;
            Ok(Type::Array(Box::new(inner), default_span()))
        }
        "FnType" => {
            let params = node.children("params").iter().map(|n| type_from_node(*n)).collect::<std::result::Result<_, _>>()?;
            let ret = node.child("return_type").map(|n| type_from_node(&n)).transpose()?
                .unwrap_or(Type::Void(default_span()));
            Ok(Type::FnPtr(params, Box::new(ret), default_span()))
        }
        kind => Err(Error::Parse(format!("unknown type kind: {}", kind))),
    }
}

// ── Helpers ──

