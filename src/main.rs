pub mod driver;
pub mod hir;
pub mod intern;
pub mod lexer;
pub mod lir;
pub mod mir;
pub mod package;
pub mod parser;
pub mod span;

use std::fmt::Write;
use std::fs;

use crate::parser::ast::{
    BinaryOp, Block, Expr, Literal, Program, Stmt, Type, UnaryOp,
};

/// Full compilation pipeline: source → LLVM IR.
///
/// Stages:
/// 1. Lexer: source text → tokens
/// 2. Parser: tokens → AST
/// 3. HIR lowering: AST → HIR (name resolution, interface registration, vtable mapping)
/// 4. MIR lowering: HIR → MIR (flatten control flow, insert memory ops)
/// 5. LIR lowering: MIR → LIR (three-address code)
struct CompileResult {
    llvm_ir: String,
    program: crate::parser::ast::Program,
    lir_program: crate::lir::ir::LirProgram,
}

fn compile_source(code: &str) -> Result<CompileResult, String> {
    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();

    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error: {}", e))?;

    let mir_program = crate::mir::lower_program(&hir_program);
    let lir_program = crate::lir::lower_program(&mir_program);
    let llvm_ir = crate::lir::emit_program(&lir_program);

    Ok(CompileResult { llvm_ir, program, lir_program })
}

fn write_stage_outputs(code: &str, out_dir: &str) -> Result<(), String> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("failed to create output dir '{}': {}", out_dir, e))?;

    let mut lexer = crate::lexer::Lexer::new(code);
    let tokens = lexer.tokenize_all();

    // Tokens
    let mut tokens_out = String::new();
    for t in tokens.iter().filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF)) {
        writeln!(tokens_out, "  {}", t).unwrap();
    }
    fs::write(format!("{}/tokens.txt", out_dir), &tokens_out)
        .map_err(|e| format!("write tokens failed: {}", e))?;

    let filtered: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .collect();

    let mut parser = crate::parser::Parser::new(filtered);
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // AST
    let ast_out = format_program(&program);
    fs::write(format!("{}/ast.txt", out_dir), &ast_out)
        .map_err(|e| format!("write ast failed: {}", e))?;

    let hir_program = crate::hir::lower_program(&program)
        .map_err(|e| format!("HIR error: {}", e))?;
    let hir_out = crate::hir::hir_program_to_string(&hir_program);
    fs::write(format!("{}/hir.txt", out_dir), &hir_out)
        .map_err(|e| format!("write hir failed: {}", e))?;

    let mir_program = crate::mir::lower_program(&hir_program);
    let mir_out = crate::mir::mir_program_to_string(&mir_program);
    fs::write(format!("{}/mir.txt", out_dir), &mir_out)
        .map_err(|e| format!("write mir failed: {}", e))?;

    let lir_program = crate::lir::lower_program(&mir_program);
    let lir_out = crate::lir::lir_program_to_string(&lir_program);
    fs::write(format!("{}/lir.txt", out_dir), &lir_out)
        .map_err(|e| format!("write lir failed: {}", e))?;

    let llvm_ir = crate::lir::emit_program(&lir_program);
    fs::write(format!("{}/llvm_ir.ll", out_dir), &llvm_ir)
        .map_err(|e| format!("write llvm_ir failed: {}", e))?;

    println!("stage output written to {}/", out_dir);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let src_path = if args.len() > 1 {
        &args[1]
    } else {
        "example/test.aya"
    };

    let code = match fs::read_to_string(src_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", src_path, e);
            std::process::exit(1);
        }
    };

    // Write stage diagnostics
    let out_dir = "test_output";
    if let Err(e) = write_stage_outputs(&code, out_dir) {
        eprintln!("error writing stage outputs: {}", e);
    }

    // Compile to executable
    match compile_source(&code) {
        Ok(result) => {
            let exe_name = {
                let p = std::path::Path::new(src_path);
                p.file_stem().unwrap_or(std::ffi::OsStr::new("a")).to_string_lossy().into_owned()
            };
            println!("compiling {} -> {}", src_path, exe_name);

            // Generate executable
            if let Err(e) = crate::driver::ir_to_executable(&result.llvm_ir, &exe_name) {
                eprintln!("error: link failed: {}", e);
            } else {
                println!("done: ./{}", exe_name);
            }

            // Generate package (.lcl) alongside the executable
            let lcl_name = format!("{}.lcl", exe_name);
            let mut pkg = crate::package::Package::new(exe_name.clone(), "0.1.0".into());
            // Determine target type: executable if has main, otherwise static-lib
            let has_main = result.program.stmts.iter().any(|s| matches!(s,
                crate::parser::ast::Stmt::FnDecl { name, .. } if name.as_str() == "main"
            ));
            pkg.target_types = if has_main {
                vec![crate::package::TargetType::Executable]
            } else {
                vec![crate::package::TargetType::StaticLib, crate::package::TargetType::DynamicLib]
            };
            pkg.collect_symbols(&result.program.stmts);
            pkg.lir_data = crate::lir::lir_program_to_string(&result.lir_program);
            if let Err(e) = pkg.write_to_file(&lcl_name) {
                eprintln!("error: package write failed: {}", e);
            } else {
                println!("package: {}", lcl_name);
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

// ---- Pretty printer ----

fn pad(n: usize) -> String {
    "│ ".repeat(n)
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Default => "???".into(),
        Type::Int(_) => "Int".into(),
        Type::Float(_) => "Float".into(),
        Type::Char(_) => "Char".into(),
        Type::Bool(_) => "Bool".into(),
        Type::Void(_) => "Void".into(),
        Type::Named(s, _) => format!("Named({})", s),
        Type::Array(inner, _) => format!("[{}]", format_type(inner)),
        Type::Unique(inner, _) => format!("unique {}", format_type(inner)),
        Type::Shared(inner, _) => format!("shared {}", format_type(inner)),
        Type::Weak(inner, _) => format!("weak {}", format_type(inner)),
        Type::Self_(_) => "Self".into(),
    }
}

fn format_op(op: &BinaryOp) -> &str {
    match op {
        BinaryOp::Add => "Add",
        BinaryOp::Sub => "Sub",
        BinaryOp::Mul => "Mul",
        BinaryOp::Div => "Div",
        BinaryOp::Mod => "Mod",
        BinaryOp::Eq => "Eq",
        BinaryOp::Neq => "Neq",
        BinaryOp::Lt => "Lt",
        BinaryOp::Gt => "Gt",
        BinaryOp::Le => "Le",
        BinaryOp::Ge => "Ge",
        BinaryOp::And => "And",
        BinaryOp::Or => "Or",
    }
}

fn format_unary(op: &UnaryOp) -> &str {
    match op {
        UnaryOp::Neg => "Neg",
        UnaryOp::Not => "Not",
    }
}

fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n, _) => format!("Int({})", n),
        Literal::Float(n, _) => format!("Float({})", n),
        Literal::Char(c, _) => format!("Char('{}')", c),
        Literal::String(s, _) => format!("String(\"{}\")", s),
        Literal::Bool(b, _) => format!("Bool({})", b),
    }
}

fn write_expr(expr: &Expr, level: usize, w: &mut impl Write) {
    match expr {
        Expr::Literal(lit) => {
            writeln!(w, "{}{}", pad(level), format_literal(lit)).unwrap();
        }
        Expr::Ident(name, _) => {
            writeln!(w, "{}Ident({})", pad(level), name).unwrap();
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            writeln!(w, "{}Binary {{ op: {} }}", pad(level), format_op(op)).unwrap();
            writeln!(w, "{}  lhs:", pad(level)).unwrap();
            write_expr(lhs, level + 1, w);
            writeln!(w, "{}  rhs:", pad(level)).unwrap();
            write_expr(rhs, level + 1, w);
        }
        Expr::Unary { op, arg, .. } => {
            writeln!(w, "{}Unary {{ op: {} }}", pad(level), format_unary(op)).unwrap();
            write_expr(arg, level + 1, w);
        }
        Expr::FnCall { name, args, .. } => {
            writeln!(w, "{}FnCall {{ name: {} }}", pad(level), name).unwrap();
            writeln!(w, "{}  args:", pad(level)).unwrap();
            if args.is_empty() {
                writeln!(w, "{}    (empty)", pad(level)).unwrap();
            } else {
                for arg in args {
                    write_expr(arg, level + 1, w);
                }
            }
        }
        Expr::Move(expr, _) => {
            writeln!(w, "{}Move", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::Clone(expr, _) => {
            writeln!(w, "{}Clone", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToUnique(expr, _) => {
            writeln!(w, "{}ToUnique", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToShared(expr, _) => {
            writeln!(w, "{}ToShared", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::ToWeak(expr, _) => {
            writeln!(w, "{}ToWeak", pad(level)).unwrap();
            write_expr(expr, level + 1, w);
        }
        Expr::MethodCall { object, method, args, .. } => {
            writeln!(w, "{}MethodCall {{ method: {} }}", pad(level), method).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  args:", pad(level)).unwrap();
            for arg in args {
                write_expr(arg, level + 1, w);
            }
        }
        Expr::FieldAccess { object, field, .. } => {
            writeln!(w, "{}FieldAccess {{ field: {} }}", pad(level), field).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
        }
        Expr::StructLiteral { type_name, fields, .. } => {
            writeln!(w, "{}StructLiteral {{ type: {} }}", pad(level), type_name).unwrap();
            for (name, val) in fields {
                writeln!(w, "{}  {} =", pad(level), name).unwrap();
                write_expr(val, level + 1, w);
            }
        }
        Expr::ArrayLiteral(elems, _) => {
            writeln!(w, "{}ArrayLiteral", pad(level)).unwrap();
            for e in elems {
                write_expr(e, level + 1, w);
            }
        }
        Expr::Index { object, index, .. } => {
            writeln!(w, "{}Index", pad(level)).unwrap();
            writeln!(w, "{}  object:", pad(level)).unwrap();
            write_expr(object, level + 1, w);
            writeln!(w, "{}  index:", pad(level)).unwrap();
            write_expr(index, level + 1, w);
        }
    }
}

fn write_block(block: &Block, level: usize, w: &mut impl Write) {
    writeln!(w, "{}Block {{", pad(level)).unwrap();
    if block.stmts.is_empty() {
        writeln!(w, "{}  (empty)", pad(level)).unwrap();
    } else {
        for stmt in &block.stmts {
            write_stmt(stmt, level + 1, w);
        }
    }
    writeln!(w, "{}}}", pad(level)).unwrap();
}

fn write_stmt(stmt: &Stmt, level: usize, w: &mut impl Write) {
    let p = pad(level);
    match stmt {
        Stmt::FnDecl {
            name,
            params,
            return_type,
            body,
            ..
        } => {
            writeln!(w, "{}FnDecl {{ name: {}, return: {} }}", p, name, format_type(return_type)).unwrap();
            if params.is_empty() {
                writeln!(w, "{}  params: (empty)", p).unwrap();
            } else {
                writeln!(w, "{}  params:", p).unwrap();
                for (n, t) in params {
                    writeln!(w, "{}    {}: {}", p, n, format_type(t)).unwrap();
                }
            }
            writeln!(w, "{}  body:", p).unwrap();
            write_block(body, level + 1, w);
        }
        Stmt::Assign { name, value, .. } => {
            writeln!(w, "{}Assign {{ name: {} }}", p, name).unwrap();
            writeln!(w, "{}  value:", p).unwrap();
            write_expr(value, level + 1, w);
        }
        Stmt::Return { value, .. } => {
            writeln!(w, "{}Return", p).unwrap();
            if let Some(val) = value {
                writeln!(w, "{}  value:", p).unwrap();
                write_expr(val, level + 1, w);
            } else {
                writeln!(w, "{}  value: (none)", p).unwrap();
            }
        }
        Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            ..
        } => {
            writeln!(w, "{}If", p).unwrap();
            writeln!(w, "{}  cond:", p).unwrap();
            write_expr(cond, level + 1, w);
            writeln!(w, "{}  then:", p).unwrap();
            write_block(then_block, level + 1, w);
            for (i, (c, b)) in elifs.iter().enumerate() {
                writeln!(w, "{}  elif[{}]:", p, i).unwrap();
                writeln!(w, "{}    cond:", p).unwrap();
                write_expr(c, level + 2, w);
                write_block(b, level + 1, w);
            }
            if let Some(b) = else_block {
                writeln!(w, "{}  else:", p).unwrap();
                write_block(b, level + 1, w);
            }
        }
        Stmt::For {
            iterator,
            start,
            end,
            step,
            body,
            ..
        } => {
            writeln!(w, "{}For {{ iterator: {} }}", p, iterator).unwrap();
            writeln!(w, "{}  start:", p).unwrap();
            write_expr(start, level + 1, w);
            writeln!(w, "{}  end:", p).unwrap();
            write_expr(end, level + 1, w);
            if let Some(s) = step {
                writeln!(w, "{}  step:", p).unwrap();
                write_expr(s, level + 1, w);
            }
            writeln!(w, "{}  body:", p).unwrap();
            write_block(body, level + 1, w);
        }
        Stmt::While { cond, body, .. } => {
            writeln!(w, "{}While", p).unwrap();
            writeln!(w, "{}  cond:", p).unwrap();
            write_expr(cond, level + 1, w);
            writeln!(w, "{}  body:", p).unwrap();
            write_block(body, level + 1, w);
        }
        Stmt::Namespace { name, items, .. } => {
            writeln!(w, "{}Namespace {{ name: {} }}", p, name).unwrap();
            if items.is_empty() {
                writeln!(w, "{}  (empty)", p).unwrap();
            } else {
                writeln!(w, "{}  items:", p).unwrap();
                for item in items {
                    write_stmt(item, level + 1, w);
                }
            }
        }
        Stmt::ExprStmt { expr, .. } => {
            writeln!(w, "{}ExprStmt", p).unwrap();
            write_expr(expr, level + 1, w);
        }
        Stmt::StructDef { name, fields, .. } => {
            writeln!(w, "{}StructDef {{ name: {} }}", p, name).unwrap();
            for (fname, fty) in fields {
                writeln!(w, "{}  {}: {}", p, fname, format_type(fty)).unwrap();
            }
        }
        Stmt::InterfaceDef { name, methods, .. } => {
            writeln!(w, "{}InterfaceDef {{ name: {} }}", p, name).unwrap();
            for m in methods {
                let params: Vec<String> = m.params.iter()
                    .map(|(n, t)| format!("{}: {}", n, format_type(t)))
                    .collect();
                writeln!(w, "{}  fn {}({}) -> {}",
                    p, m.name, params.join(", "), format_type(&m.return_type)).unwrap();
            }
        }
        Stmt::ImplBlock { type_name, methods, .. } => {
            writeln!(w, "{}ImplBlock {{ name: {} }}", p, type_name).unwrap();
            for m in methods {
                write_stmt(m, level + 1, w);
            }
        }
        Stmt::Import { path, .. } => {
            writeln!(w, "{}Import {{ path: {} }}", p, path).unwrap();
        }
    }
}

fn format_program(program: &Program) -> String {
    let mut s = String::new();
    writeln!(s, "Program").unwrap();
    for stmt in &program.stmts {
        write_stmt(stmt, 1, &mut s);
    }
    s
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::ir::*;
    use crate::lexer::Lexer;

    fn parse(code: &str) -> Program {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize_all();
        let filtered: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
            .collect();
        let mut parser = crate::parser::Parser::new(filtered);
        parser.parse_program().expect("parse failed")
    }

    fn lower(program: &Program) -> HirProgram {
        crate::hir::lower_program(program).expect("lower failed")
    }

    // ----------------------------------------------------------------
    //  Basic function lowering
    // ----------------------------------------------------------------

    #[test]
    fn test_simple_fn() {
        let program = parse("fn main()->int{ return 1; }");
        let hir = lower(&program);

        assert_eq!(hir.items.len(), 1);
        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.name.as_str(), "main");
                assert_eq!(f.return_type, HirType::Int);
                assert!(f.params.is_empty());
                assert!(f.locals.is_empty());
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0] {
                    HirStmt::Return { value } => {
                        let val = value.as_ref().expect("expected return value");
                        match val {
                            HirExpr::Literal(HirLiteral::Int(1), HirType::Int) => {}
                            _ => panic!("expected Int(1) literal"),
                        }
                    }
                    _ => panic!("expected Return stmt"),
                }
            }
            _ => panic!("expected HirItem::Fn"),
        }
    }

    #[test]
    fn test_fn_with_params() {
        let program = parse("fn add(int a, int b)->int{ return a+b; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.name.as_str(), "add");
                assert_eq!(f.params.len(), 2);
                assert_eq!(f.params[0].0.as_str(), "a");
                assert_eq!(f.params[0].1, HirType::Int);
                assert_eq!(f.params[1].0.as_str(), "b");
                assert_eq!(f.params[1].1, HirType::Int);
                assert_eq!(f.locals.len(), 2); // params become locals

                // body: return a+b
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Binary { op: BinaryOp::Add, lhs, rhs, ty }) } => {
                        assert_eq!(*ty, HirType::Int);
                        match (lhs.as_ref(), rhs.as_ref()) {
                            (HirExpr::Local(VarId(0), _), HirExpr::Local(VarId(1), _)) => {}
                            _ => panic!("expected Local(0) + Local(1)"),
                        }
                    }
                    _ => panic!("expected Return with Binary Add"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_fn_with_unique_shared_weak_params() {
        let program = parse("fn foo(unique int x, shared float y, weak char z)->int{ return 0; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.params[0].1, HirType::Unique(Box::new(HirType::Int)));
                assert_eq!(f.params[1].1, HirType::Shared(Box::new(HirType::Float)));
                assert_eq!(f.params[2].1, HirType::Weak(Box::new(HirType::Char)));
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  Assignment
    // ----------------------------------------------------------------

    #[test]
    fn test_assign() {
        let program = parse("fn main()->int{ a=1; return a; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                assert_eq!(f.locals.len(), 1);
                assert_eq!(f.locals[0].name.as_str(), "a");

                // stmt 0: assign a = 1
                match &f.body.stmts[0] {
                    HirStmt::Assign { target, value } => {
                        match target {
                            HirExpr::Local(VarId(0), HirType::Int) => {}
                            _ => panic!("expected Local(0, Int)"),
                        }
                        match value {
                            HirExpr::Literal(HirLiteral::Int(1), HirType::Int) => {}
                            _ => panic!("expected Int(1)"),
                        }
                    }
                    _ => panic!("expected Assign"),
                }

                // stmt 1: return a
                match &f.body.stmts[1] {
                    HirStmt::Return { value: Some(HirExpr::Local(VarId(0), HirType::Int)) } => {}
                    _ => panic!("expected Return Local(0)"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  All literal types
    // ----------------------------------------------------------------

    #[test]
    fn test_literals() {
        let program = parse(r#"fn main()->int{ a=1; b=2.0; c='x'; d="hello"; return 0; }"#);
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                // a=1
                match &f.body.stmts[0] {
                    HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Int(1), HirType::Int), .. } => {}
                    _ => panic!("expected Int literal"),
                }
                // b=2.0
                match &f.body.stmts[1] {
                    HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Float(v), HirType::Float), .. } => {
                        assert!((*v - 2.0).abs() < 1e-10);
                    }
                    _ => panic!("expected Float literal"),
                }
                // c='x'
                match &f.body.stmts[2] {
                    HirStmt::Assign { value: HirExpr::Literal(HirLiteral::Char('x'), HirType::Char), .. } => {}
                    _ => panic!("expected Char literal"),
                }
                // d="hello"
                match &f.body.stmts[3] {
                    HirStmt::Assign { value: HirExpr::Literal(HirLiteral::String(s), HirType::Named(_)), .. } => {
                        assert_eq!(s, "hello");
                    }
                    _ => panic!("expected String literal"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  If / elif / else
    // ----------------------------------------------------------------

    #[test]
    fn test_if_else() {
        let program = parse("fn main()->int{ if 1>0{ return 1; }else{ return 2; } }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::If { cond, then_block, elifs, else_block, .. } => {
                        matches!(cond, HirExpr::Binary { op: BinaryOp::Gt, .. });
                        assert_eq!(then_block.stmts.len(), 1);
                        assert!(elifs.is_empty());
                        assert!(else_block.is_some());
                        let eb = else_block.as_ref().unwrap();
                        assert_eq!(eb.stmts.len(), 1);
                        matches!(&eb.stmts[0], HirStmt::Return { .. });
                    }
                    _ => panic!("expected If"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn test_if_elif_else() {
        let program = parse(
            "fn cmp(int a, int b)->int{ if a>b{ return 1; }elif a<b{ return 2; }elif a==b{ return 3; }else{ return 4; } }",
        );
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::If { elifs, else_block, .. } => {
                        assert_eq!(elifs.len(), 2);
                        assert!(else_block.is_some());
                    }
                    _ => panic!("expected If"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  While loop
    // ----------------------------------------------------------------

    #[test]
    fn test_while() {
        let program = parse("fn main()->int{ a=0; while a<10{ a=a+1; } return a; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                match &f.body.stmts[1] {
                    HirStmt::While { cond, body } => {
                        match cond {
                            HirExpr::Binary { op: BinaryOp::Lt, .. } => {}
                            _ => panic!("expected Lt condition"),
                        }
                        assert_eq!(body.stmts.len(), 1);
                    }
                    _ => panic!("expected While"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  For loop desugaring
    // ----------------------------------------------------------------

    #[test]
    fn test_for_loop() {
        let program = parse("fn main(int a)->int{ for i in (1, 10){ a = a + i; } return a; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                // for desugars into Block([init, While])
                match &f.body.stmts[0] {
                    HirStmt::Block(stmts) => {
                        assert_eq!(stmts.len(), 2);
                        // init: assign i = 1
                        match &stmts[0] {
                            HirStmt::Assign { target, value } => {
                                matches!(target, HirExpr::Local(_, HirType::Int));
                                matches!(value, HirExpr::Literal(HirLiteral::Int(1), HirType::Int));
                            }
                            _ => panic!("expected Assign in for-init"),
                        }
                        // while i < 10
                        match &stmts[1] {
                            HirStmt::While { cond, body } => {
                                match cond {
                                    HirExpr::Binary { op: BinaryOp::Lt, .. } => {}
                                    _ => panic!("expected Lt condition"),
                                }
                                // body has a = a + i + i = i + 1
                                assert_eq!(body.stmts.len(), 2);
                            }
                            _ => panic!("expected While"),
                        }
                    }
                    _ => panic!("expected Block for for-loop"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  Namespace
    // ----------------------------------------------------------------

    #[test]
    fn test_namespace() {
        let program = parse(
            "namespace math{ fn add(int a, int b)->int{ return a+b; } fn sub(int a, int b)->int{ return a-b; } }",
        );
        let hir = lower(&program);

        assert_eq!(hir.items.len(), 1);
        match &hir.items[0] {
            HirItem::Namespace { name, items } => {
                assert_eq!(name.as_str(), "math");
                assert_eq!(items.len(), 2);
                match &items[0] {
                    HirItem::Fn(f) => assert_eq!(f.name.as_str(), "add"),
                    _ => panic!("expected Fn in namespace"),
                }
                match &items[1] {
                    HirItem::Fn(f) => assert_eq!(f.name.as_str(), "sub"),
                    _ => panic!("expected Fn in namespace"),
                }
            }
            _ => panic!("expected Namespace"),
        }
    }

    // ----------------------------------------------------------------
    //  Function call
    // ----------------------------------------------------------------

    #[test]
    fn test_fn_call() {
        let program = parse("fn add(int a, int b)->int{ return a+b; } fn main()->int{ return add(1,2); }");
        let hir = lower(&program);

        assert_eq!(hir.items.len(), 2);
        match &hir.items[1] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Call { fn_id, args, ty }) } => {
                        assert_eq!(fn_id.0, 0); // calls `add` which is FnId(0)
                        assert_eq!(args.len(), 2);
                        assert_eq!(*ty, HirType::Int);
                    }
                    _ => panic!("expected Return with Call"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  Move / Clone
    // ----------------------------------------------------------------

    #[test]
    fn test_move_clone() {
        let program = parse("fn foo(unique int x)->unique int{ return move x; } fn bar(shared int y)->int{ return clone y; }");
        let hir = lower(&program);

        // foo: return move x
        match &hir.items[0] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Move(inner, ty)) } => {
                        assert_eq!(*ty, HirType::Unique(Box::new(HirType::Int)));
                        match inner.as_ref() {
                            HirExpr::Local(VarId(0), _) => {}
                            _ => panic!("expected Local(0) inside Move"),
                        }
                    }
                    _ => panic!("expected Return with Move"),
                }
            }
            _ => panic!("expected Fn"),
        }

        // bar: return clone y
        match &hir.items[1] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Clone(inner, ty)) } => {
                        assert_eq!(*ty, HirType::Shared(Box::new(HirType::Int)));
                        match inner.as_ref() {
                            HirExpr::Local(VarId(0), _) => {}
                            _ => panic!("expected Local(0) inside Clone"),
                        }
                    }
                    _ => panic!("expected Return with Clone"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  Error cases
    // ----------------------------------------------------------------

    #[test]
    fn test_undefined_variable() {
        let program = parse("fn main()->int{ return x; }");
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("undefined variable"), "got: {}", err);
    }

    #[test]
    fn test_undefined_function() {
        let program = parse("fn main()->int{ foo(); return 0; }");
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("undefined function"), "got: {}", err);
    }

    #[test]
    fn test_fn_decl_inside_body() {
        let program = parse("fn main()->int{ fn inner()->int{ return 1; } return 0; }");
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("unexpected declaration"), "got: {}", err);
    }

    #[test]
    fn test_non_fn_top_level() {
        let program = parse("a=1;");
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("unexpected top-level statement"),
            "got: {}",
            err
        );
    }

    // ----------------------------------------------------------------
    //  For loop with step
    // ----------------------------------------------------------------

    #[test]
    fn test_for_loop_with_step() {
        let program = parse("fn main(int a)->int{ for i in (0, 100, 2){ a = a + i; } return a; }");
        let hir = lower(&program);

        match &hir.items[0] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Block(stmts) => {
                        // step expression should be Int(2)
                        match &stmts[1] {
                            HirStmt::While { body, .. } => {
                                // last stmt in body = i = i + 2
                                assert_eq!(body.stmts.len(), 2);
                            }
                            _ => panic!("expected While"),
                        }
                    }
                    _ => panic!("expected Block"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    // ----------------------------------------------------------------
    //  Function overloading
    // ----------------------------------------------------------------

    #[test]
    fn test_overload_by_param_types() {
        let program = parse(
            "fn foo(int a)->int{ return a; }
             fn foo(float b)->float{ return b; }
             fn main()->int{ return foo(1); }",
        );
        let hir = lower(&program);
        // main calls foo(int), which should resolve to FnId(0)
        match &hir.items[2] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Call { fn_id, args, ty }) } => {
                        assert_eq!(fn_id.0, 0, "foo(int) should resolve to first overload");
                        assert_eq!(*ty, HirType::Int);
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("expected Return with Call"),
                }
            }
            _ => panic!("expected Fn for main"),
        }
    }

    #[test]
    fn test_overload_resolve_float() {
        let program = parse(
            "fn foo(int a)->int{ return a; }
             fn foo(float b)->float{ return b; }
             fn main()->float{ return foo(1.0); }",
        );
        let hir = lower(&program);
        // main calls foo(float), which should resolve to FnId(1)
        match &hir.items[2] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Call { fn_id, args, ty }) } => {
                        assert_eq!(fn_id.0, 1, "foo(float) should resolve to second overload");
                        assert_eq!(*ty, HirType::Float);
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("expected Return with Call"),
                }
            }
            _ => panic!("expected Fn for main"),
        }
    }

    #[test]
    fn test_overload_three_overloads() {
        let program = parse(
            "fn add(int a, int b)->int{ return a+b; }
             fn add(float a, float b)->float{ return a+b; }
             fn add(int a, int b, int c)->int{ return a+b+c; }
             fn main()->float{ return add(1.0, 2.0); }",
        );
        let hir = lower(&program);
        match &hir.items[3] {
            HirItem::Fn(f) => {
                match &f.body.stmts[0] {
                    HirStmt::Return { value: Some(HirExpr::Call { fn_id, args, ty }) } => {
                        assert_eq!(fn_id.0, 1, "add(float,float) should be FnId(1)");
                        assert_eq!(*ty, HirType::Float);
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("expected Return with Call"),
                }
            }
            _ => panic!("expected Fn for main"),
        }
    }

    #[test]
    fn test_overload_no_match_error() {
        let program = parse(
            "fn foo(int a)->int{ return a; }
             fn foo(float b)->float{ return b; }
             fn main()->int{ return foo('x'); }",
        );
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("no matching overload"), "got: {}", err);
    }

    #[test]
    fn test_overload_duplicate_sig_error() {
        // Two functions with same name, same params → ambiguous / duplicate → error
        let program = parse(
            "fn foo(int a)->int{ return a; }
             fn foo(int b)->int{ return b; }
             fn main()->int{ return foo(1); }",
        );
        let result = crate::hir::lower_program(&program);
        assert!(result.is_err());
    }
}
