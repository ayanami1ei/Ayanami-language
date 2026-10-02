use super::*;

pub fn node_to_stmt(node: &Node) -> Result<Stmt> {
    match node.kind.as_str() {
        "FnDecl" => {
            let name = get_str(node, "name")?;
            let params_node = node.child("params");
            let body_node = node.child("body");
            let params = if let Some(pn) = params_node {
                params_from_node(pn)?
            } else {
                vec![]
            };
            let body = if let Some(bn) = body_node {
                block_from_node(&bn)?
            } else {
                crate::parser::ast::block::Block::new(vec![], default_span())
            };
            let ret_ty = node.child("return_type").map(|n| type_from_node(&n)).transpose()?
                .unwrap_or(Type::Void(default_span()));
            Ok(Stmt::FnDecl {
                vis: Visibility::Pub,
                is_inline: false,
                extern_c: false,
                name: Symbol::intern(&get_str(node, "name")?),
                generic_params: vec![],
                params,
                return_type: ret_ty,
                body,
                span: default_span(),
            })
        }
        "Import" => {
            let path = get_str(node, "path")?;
            Ok(Stmt::Import { path, span: default_span() })
        }
        "ReturnStmt" => {
            let value = node.child("value").map(|n| node_to_expr(&n)).transpose()?;
            Ok(Stmt::Return { value, span: default_span() })
        }
        "VarDecl" => {
            let name = get_str(node, "name")?;
            let value = node_to_expr(node.child("value").ok_or_else(|| Error::Parse("missing value".into()))?)?;
            Ok(Stmt::Assign { name: Symbol::intern(&name), is_mut: false, value, span: default_span() })
        }
        "IfStmt" => {
            let cond = node_to_expr(node.child("cond").ok_or_else(|| Error::Parse("missing cond".into()))?)?;
            let then_block = block_from_node(node.child("then_block").ok_or_else(|| Error::Parse("missing then".into()))?)?;
            Ok(Stmt::If { cond, then_block, elifs: vec![], else_block: None, span: default_span() })
        }
        "WhileStmt" => {
            let cond = node_to_expr(node.child("cond").ok_or_else(|| Error::Parse("missing cond".into()))?)?;
            let body = block_from_node(node.child("body").ok_or_else(|| Error::Parse("missing body".into()))?)?;
            Ok(Stmt::While { cond, body, span: default_span() })
        }
        "ForStmt" => {
            let name = get_str(node, "name")?;
            let start = node_to_expr(node.child("start").ok_or_else(|| Error::Parse("missing start".into()))?)?;
            let end = node_to_expr(node.child("end").ok_or_else(|| Error::Parse("missing end".into()))?)?;
            let body = block_from_node(node.child("body").ok_or_else(|| Error::Parse("missing body".into()))?)?;
            Ok(Stmt::For { iterator: Symbol::intern(&name), start, end, step: None, body, span: default_span() })
        }
        "BreakStmt" => Ok(Stmt::Break { span: default_span() }),
        "ContinueStmt" => Ok(Stmt::Continue { span: default_span() }),
        "ExprStmt" => {
            let expr = node_to_expr(node.child("expr").ok_or_else(|| Error::Parse("missing expr".into()))?)?;
            Ok(Stmt::ExprStmt { expr, span: default_span() })
        }
        "Block" => {
            let stmts = node.children("stmts");
            let items: Vec<Stmt> = stmts.iter().map(|n| node_to_stmt(*n)).collect::<std::result::Result<_, _>>()?;
            Ok(Stmt::ExprStmt { expr: Expr::Literal(Literal::Int(0, default_span())), span: default_span() })
        }
        kind => Err(Error::Parse(format!("unknown stmt kind: {}", kind))),
    }
}

