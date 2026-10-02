use super::*;

pub fn node_to_expr(node: &Node) -> Result<Expr> {
    match node.kind.as_str() {
        "IntLiteral" => {
            let val = get_str(node, "value")?;
            let n = val.parse::<i64>().map_err(|_| Error::Parse(format!("invalid int: {}", val)))?;
            Ok(Expr::Literal(Literal::Int(n, default_span())))
        }
        "FloatLiteral" => {
            let val = get_str(node, "value")?;
            let n = val.parse::<f64>().map_err(|_| Error::Parse(format!("invalid float: {}", val)))?;
            Ok(Expr::Literal(Literal::Float(n, default_span())))
        }
        "StringLiteral" => {
            Ok(Expr::Literal(Literal::String(get_str(node, "value")?, default_span())))
        }
        "BoolLiteral" => {
            let val = get_str(node, "value")?;
            Ok(Expr::Literal(Literal::Bool(val == "true", default_span())))
        }
        "CharLiteral" => {
            let val = get_str(node, "value")?;
            let c = val.chars().next().unwrap_or('\0');
            Ok(Expr::Literal(Literal::Char(c, default_span())))
        }
        "Ident" => {
            let name = get_str(node, "name")?;
            Ok(Expr::Ident(Symbol::intern(&name), default_span()))
        }
        "BinaryExpr" => {
            let op_str = get_str(node, "op")?;
            let lhs = node_to_expr(node.child("lhs").ok_or_else(|| Error::Parse("missing lhs".into()))?)?;
            let rhs = node_to_expr(node.child("rhs").ok_or_else(|| Error::Parse("missing rhs".into()))?)?;
            let op = str_to_binop(&op_str)?;
            Ok(Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span: default_span() })
        }
        "UnaryExpr" => {
            let op_str = get_str(node, "op")?;
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            let op = match op_str.as_str() {
                "-" => UnaryOp::Neg, "!" => UnaryOp::Not,
                _ => return Err(Error::Parse(format!("unknown unary op: {}", op_str))),
            };
            Ok(Expr::Unary { op, arg: Box::new(arg), span: default_span() })
        }
        "FnCallExpr" => {
            let name = Symbol::intern(&get_str(node, "name")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::FnCall { name, args, span: default_span() })
        }
        "CallExpr" => {
            let target = node_to_expr(node.child("target").ok_or_else(|| Error::Parse("missing target".into()))?)?;
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::CallExpr { target: Box::new(target), args, span: default_span() })
        }
        "FieldExpr" => {
            let object = node_to_expr(node.child("object").ok_or_else(|| Error::Parse("missing object".into()))?)?;
            let field = Symbol::intern(&get_str(node, "field")?);
            Ok(Expr::FieldAccess { object: Box::new(object), field, span: default_span() })
        }
        "IndexExpr" => {
            let object = node_to_expr(node.child("object").ok_or_else(|| Error::Parse("missing object".into()))?)?;
            let index = node_to_expr(node.child("index").ok_or_else(|| Error::Parse("missing index".into()))?)?;
            Ok(Expr::Index { object: Box::new(object), index: Box::new(index), span: default_span() })
        }
        "MethodCallExpr" => {
            let object = node_to_expr(node.child("object").ok_or_else(|| Error::Parse("missing object".into()))?)?;
            let method = Symbol::intern(&get_str(node, "method")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::MethodCall { object: Box::new(object), method, args, span: default_span() })
        }
        "MoveExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::Move(Box::new(arg), default_span()))
        }
        "CloneExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::Clone(Box::new(arg), default_span()))
        }
        "ToUniqueExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::ToUnique(Box::new(arg), default_span()))
        }
        "ToSharedExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::ToShared(Box::new(arg), default_span()))
        }
        "ToWeakExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::ToWeak(Box::new(arg), default_span()))
        }
        "RefExpr" => {
            let mutable = node.get("mutable").map(|v| matches!(v, Value::String(s) if s == "mut")).unwrap_or(false);
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::Ref(Box::new(arg), mutable, default_span()))
        }
        "TryOp" => {
            let arg = node_to_expr(node.child("arg").ok_or_else(|| Error::Parse("missing arg".into()))?)?;
            Ok(Expr::TryOp(Box::new(arg), default_span()))
        }
        "NullExpr" => Ok(Expr::Null(default_span())),
        "ArrayLiteral" => {
            let elems = node.children("elems").iter().map(|n| node_to_expr(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::ArrayLiteral(elems, default_span()))
        }
        "ArraySized" => {
            let elem_type = type_from_node(node.child("elem_type").ok_or_else(|| Error::Parse("missing elem_type".into()))?)?;
            let count = node_to_expr(node.child("count").ok_or_else(|| Error::Parse("missing count".into()))?)?;
            Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: default_span() })
        }
        "StructLiteral" => {
            let type_name = Symbol::intern(&get_str(node, "type_name")?);
            let fields = node.children("fields").iter().map(|n| {
                let name = Symbol::intern(&get_str(*n, "name")?);
                let val = node_to_expr((*n).child("value").ok_or_else(|| Error::Parse("missing value".into()))?)?;
                Ok::<_, Error>((name, val))
            }).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::StructLiteral { type_name, generic_args: vec![], fields, span: default_span() })
        }
        "LambdaExpr" => {
            let params_node = node.child("params").ok_or_else(|| Error::Parse("missing params".into()))?;
            let params = params_from_node(&params_node)?;
            let ret_ty = node.child("return_type").map(|n| type_from_node(&n)).transpose()?
                .unwrap_or(Type::Void(default_span()));
            let body = block_from_node(node.child("body").ok_or_else(|| Error::Parse("missing body".into()))?)?;
            Ok(Expr::Lambda { params, return_type: ret_ty, body: body.stmts, span: default_span() })
        }
        "EnumConstruct" => {
            let enum_name = Symbol::intern(&get_str(node, "enum_name")?);
            let variant_name = Symbol::intern(&get_str(node, "variant_name")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::EnumConstruct { enum_name, variant_name, tuple_args: args, named_args: vec![], span: default_span() })
        }
        "AsmExpr" => {
            let template = get_str(node, "template")?;
            let outputs = node.children("outputs").iter().map(|n| {
                let c = get_str(*n, "constraint")?;
                let e = node_to_expr((*n).child("expr").ok_or_else(|| Error::Parse("missing expr".into()))?)?;
                Ok::<_, Error>((c, Box::new(e)))
            }).collect::<std::result::Result<_, _>>()?;
            let inputs = node.children("inputs").iter().map(|n| {
                let c = get_str(*n, "constraint")?;
                let e = node_to_expr((*n).child("expr").ok_or_else(|| Error::Parse("missing expr".into()))?)?;
                Ok::<_, Error>((c, Box::new(e)))
            }).collect::<std::result::Result<_, _>>()?;
            Ok(Expr::Asm { template, outputs, inputs, span: default_span() })
        }
        kind => Err(Error::Parse(format!("unknown expr kind: {}", kind))),
    }
}

