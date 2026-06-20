// ── Bridge: Asuka generated parser Node → typed AST enums ──

use asuka::runtime::{Node, Value};
use crate::parser::ast::*;
use crate::parser::ast::vis::Visibility;
use crate::intern::Symbol;
use crate::span::Span;

pub fn node_to_program(node: &Node) -> Result<crate::parser::ast::block::Block, String> {
    let items = node.children("items");
    let mut stmts = Vec::new();
    for item in &items {
        stmts.push(node_to_stmt(item)?);
    }
    Ok(crate::parser::ast::block::Block::new(stmts, default_span()))
}

pub fn node_to_stmt(node: &Node) -> Result<Stmt, String> {
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
            let value = node_to_expr(node.child("value").ok_or("missing value")?)?;
            Ok(Stmt::Assign { name: Symbol::intern(&name), is_mut: false, value, span: default_span() })
        }
        "IfStmt" => {
            let cond = node_to_expr(node.child("cond").ok_or("missing cond")?)?;
            let then_block = block_from_node(node.child("then_block").ok_or("missing then")?)?;
            Ok(Stmt::If { cond, then_block, elifs: vec![], else_block: None, span: default_span() })
        }
        "WhileStmt" => {
            let cond = node_to_expr(node.child("cond").ok_or("missing cond")?)?;
            let body = block_from_node(node.child("body").ok_or("missing body")?)?;
            Ok(Stmt::While { cond, body, span: default_span() })
        }
        "ForStmt" => {
            let name = get_str(node, "name")?;
            let start = node_to_expr(node.child("start").ok_or("missing start")?)?;
            let end = node_to_expr(node.child("end").ok_or("missing end")?)?;
            let body = block_from_node(node.child("body").ok_or("missing body")?)?;
            Ok(Stmt::For { iterator: Symbol::intern(&name), start, end, step: None, body, span: default_span() })
        }
        "BreakStmt" => Ok(Stmt::Break { span: default_span() }),
        "ContinueStmt" => Ok(Stmt::Continue { span: default_span() }),
        "ExprStmt" => {
            let expr = node_to_expr(node.child("expr").ok_or("missing expr")?)?;
            Ok(Stmt::ExprStmt { expr, span: default_span() })
        }
        "Block" => {
            let stmts = node.children("stmts");
            let items: Vec<Stmt> = stmts.iter().map(|n| node_to_stmt(*n)).collect::<Result<_, _>>()?;
            Ok(Stmt::ExprStmt { expr: Expr::Literal(Literal::Int(0, default_span())), span: default_span() })
        }
        kind => Err(format!("unknown stmt kind: {}", kind)),
    }
}

pub fn node_to_expr(node: &Node) -> Result<Expr, String> {
    match node.kind.as_str() {
        "IntLiteral" => {
            let val = get_str(node, "value")?;
            let n = val.parse::<i64>().map_err(|_| format!("invalid int: {}", val))?;
            Ok(Expr::Literal(Literal::Int(n, default_span())))
        }
        "FloatLiteral" => {
            let val = get_str(node, "value")?;
            let n = val.parse::<f64>().map_err(|_| format!("invalid float: {}", val))?;
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
            let lhs = node_to_expr(node.child("lhs").ok_or("missing lhs")?)?;
            let rhs = node_to_expr(node.child("rhs").ok_or("missing rhs")?)?;
            let op = str_to_binop(&op_str)?;
            Ok(Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span: default_span() })
        }
        "UnaryExpr" => {
            let op_str = get_str(node, "op")?;
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            let op = match op_str.as_str() {
                "-" => UnaryOp::Neg, "!" => UnaryOp::Not,
                _ => return Err(format!("unknown unary op: {}", op_str)),
            };
            Ok(Expr::Unary { op, arg: Box::new(arg), span: default_span() })
        }
        "FnCallExpr" => {
            let name = Symbol::intern(&get_str(node, "name")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<Result<_, _>>()?;
            Ok(Expr::FnCall { name, args, span: default_span() })
        }
        "CallExpr" => {
            let target = node_to_expr(node.child("target").ok_or("missing target")?)?;
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<Result<_, _>>()?;
            Ok(Expr::CallExpr { target: Box::new(target), args, span: default_span() })
        }
        "FieldExpr" => {
            let object = node_to_expr(node.child("object").ok_or("missing object")?)?;
            let field = Symbol::intern(&get_str(node, "field")?);
            Ok(Expr::FieldAccess { object: Box::new(object), field, span: default_span() })
        }
        "IndexExpr" => {
            let object = node_to_expr(node.child("object").ok_or("missing object")?)?;
            let index = node_to_expr(node.child("index").ok_or("missing index")?)?;
            Ok(Expr::Index { object: Box::new(object), index: Box::new(index), span: default_span() })
        }
        "MethodCallExpr" => {
            let object = node_to_expr(node.child("object").ok_or("missing object")?)?;
            let method = Symbol::intern(&get_str(node, "method")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<Result<_, _>>()?;
            Ok(Expr::MethodCall { object: Box::new(object), method, args, span: default_span() })
        }
        "MoveExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::Move(Box::new(arg), default_span()))
        }
        "CloneExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::Clone(Box::new(arg), default_span()))
        }
        "ToUniqueExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::ToUnique(Box::new(arg), default_span()))
        }
        "ToSharedExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::ToShared(Box::new(arg), default_span()))
        }
        "ToWeakExpr" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::ToWeak(Box::new(arg), default_span()))
        }
        "RefExpr" => {
            let mutable = node.get("mutable").map(|v| matches!(v, Value::String(s) if s == "mut")).unwrap_or(false);
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::Ref(Box::new(arg), mutable, default_span()))
        }
        "TryOp" => {
            let arg = node_to_expr(node.child("arg").ok_or("missing arg")?)?;
            Ok(Expr::TryOp(Box::new(arg), default_span()))
        }
        "NullExpr" => Ok(Expr::Null(default_span())),
        "ArrayLiteral" => {
            let elems = node.children("elems").iter().map(|n| node_to_expr(*n)).collect::<Result<_, _>>()?;
            Ok(Expr::ArrayLiteral(elems, default_span()))
        }
        "ArraySized" => {
            let elem_type = type_from_node(node.child("elem_type").ok_or("missing elem_type")?)?;
            let count = node_to_expr(node.child("count").ok_or("missing count")?)?;
            Ok(Expr::ArraySized { elem_type, count: Box::new(count), span: default_span() })
        }
        "StructLiteral" => {
            let type_name = Symbol::intern(&get_str(node, "type_name")?);
            let fields = node.children("fields").iter().map(|n| {
                let name = Symbol::intern(&get_str(*n, "name")?);
                let val = node_to_expr((*n).child("value").ok_or("missing value")?)?;
                Ok::<_, String>((name, val))
            }).collect::<Result<_, _>>()?;
            Ok(Expr::StructLiteral { type_name, generic_args: vec![], fields, span: default_span() })
        }
        "LambdaExpr" => {
            let params_node = node.child("params").ok_or("missing params")?;
            let params = params_from_node(&params_node)?;
            let ret_ty = node.child("return_type").map(|n| type_from_node(&n)).transpose()?
                .unwrap_or(Type::Void(default_span()));
            let body = block_from_node(node.child("body").ok_or("missing body")?)?;
            Ok(Expr::Lambda { params, return_type: ret_ty, body: body.stmts, span: default_span() })
        }
        "EnumConstruct" => {
            let enum_name = Symbol::intern(&get_str(node, "enum_name")?);
            let variant_name = Symbol::intern(&get_str(node, "variant_name")?);
            let args = node.children("args").iter().map(|n| node_to_expr(*n)).collect::<Result<_, _>>()?;
            Ok(Expr::EnumConstruct { enum_name, variant_name, tuple_args: args, named_args: vec![], span: default_span() })
        }
        "AsmExpr" => {
            let template = get_str(node, "template")?;
            let outputs = node.children("outputs").iter().map(|n| {
                let c = get_str(*n, "constraint")?;
                let e = node_to_expr((*n).child("expr").ok_or("missing expr")?)?;
                Ok::<_, String>((c, Box::new(e)))
            }).collect::<Result<_, _>>()?;
            let inputs = node.children("inputs").iter().map(|n| {
                let c = get_str(*n, "constraint")?;
                let e = node_to_expr((*n).child("expr").ok_or("missing expr")?)?;
                Ok::<_, String>((c, Box::new(e)))
            }).collect::<Result<_, _>>()?;
            Ok(Expr::Asm { template, outputs, inputs, span: default_span() })
        }
        kind => Err(format!("unknown expr kind: {}", kind)),
    }
}

pub fn type_from_node(node: &Node) -> Result<Type, String> {
    match node.kind.as_str() {
        "IntLiteral" => Ok(Type::Int(default_span())),
        "TypeBase" => {
            let name_str = node.get("name").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .ok_or("missing type name")?;
            match name_str.as_str() {
                "int" => Ok(Type::Int(default_span())),
                "float" => Ok(Type::Float(default_span())),
                "char" => Ok(Type::Char(default_span())),
                "bool" => Ok(Type::Bool(default_span())),
                "void" => Ok(Type::Void(default_span())),
                _ => Ok(Type::Named(Symbol::intern(&name_str), default_span())),
            }
        }
        "SharedType" => {
            let inner = type_from_node(node.child("inner").ok_or("missing inner")?)?;
            Ok(Type::Shared(Box::new(inner), default_span()))
        }
        "UniqueType" => {
            let inner = type_from_node(node.child("inner").ok_or("missing inner")?)?;
            Ok(Type::Unique(Box::new(inner), default_span()))
        }
        "WeakType" => {
            let inner = type_from_node(node.child("inner").ok_or("missing inner")?)?;
            Ok(Type::Weak(Box::new(inner), default_span()))
        }
        "ArrayType" => {
            let inner = type_from_node(node.child("inner").ok_or("missing inner")?)?;
            Ok(Type::Array(Box::new(inner), default_span()))
        }
        "FnType" => {
            let params = node.children("params").iter().map(|n| type_from_node(*n)).collect::<Result<_, _>>()?;
            let ret = node.child("return_type").map(|n| type_from_node(&n)).transpose()?
                .unwrap_or(Type::Void(default_span()));
            Ok(Type::FnPtr(params, Box::new(ret), default_span()))
        }
        kind => Err(format!("unknown type kind: {}", kind)),
    }
}

// ── Helpers ──

fn get_str(node: &Node, field: &str) -> Result<String, String> {
    node.get(field).and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .ok_or_else(|| format!("missing field '{}' in node '{}'", field, node.kind))
}

fn str_to_binop(s: &str) -> Result<BinaryOp, String> {
    match s {
        "+" => Ok(BinaryOp::Add), "-" => Ok(BinaryOp::Sub),
        "*" => Ok(BinaryOp::Mul), "/" => Ok(BinaryOp::Div), "%" => Ok(BinaryOp::Mod),
        "==" => Ok(BinaryOp::Eq), "!=" => Ok(BinaryOp::Neq),
        "<" => Ok(BinaryOp::Lt), ">" => Ok(BinaryOp::Gt),
        "<=" => Ok(BinaryOp::Le), ">=" => Ok(BinaryOp::Ge),
        "&&" => Ok(BinaryOp::And), "||" => Ok(BinaryOp::Or),
        _ => Err(format!("unknown operator: {}", s)),
    }
}

fn params_from_node(node: &Node) -> Result<Vec<(Symbol, Type)>, String> {
    let mut params = Vec::new();
    for child in node.children("items") {
        let ty = type_from_node(child.child("type").ok_or("missing param type")?)?;
        let name = get_str(child, "name")?;
        params.push((Symbol::intern(&name), ty));
    }
    Ok(params)
}

fn block_from_node(node: &Node) -> Result<crate::parser::ast::block::Block, String> {
    let stmts = node.children("stmts").iter().map(|n| node_to_stmt(*n)).collect::<Result<_, _>>()?;
    Ok(crate::parser::ast::block::Block::new(stmts, default_span()))
}

fn default_span() -> Span {
    Span::default()
}
