/// 使用 Asuka 生成的 parser 解析 Ayanami 源代码
/// 运行: cargo run --example asuka_parse -- path/to/file.aya

use asuka::runtime;
use ayanami::generated::{tokenize, Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).unwrap_or_else(|_| {
            eprintln!("usage: asuka_parse <file.aya>");
            std::process::exit(1);
        })
    } else {
        "fn main() { return 0 }".into()
    };

    let tokens = tokenize(&source);
    println!("tokens: {}", tokens.len());

    let mut p = Parser::new(tokens);
    match p.pprogram() {
        Ok(val) => {
            println!("parse OK");
            if let runtime::Value::Node(node) = &val {
                print_node(node, 0);
            }
        }
        Err(e) => eprintln!("parse failed: {}", e),
    }
}

fn print_node(node: &runtime::Node, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{}", indent, node.kind);
    for (name, val) in &node.fields {
        match val {
            runtime::Value::Node(child) => {
                println!("{}  {}:", indent, name);
                print_node(child, depth + 2);
            }
            runtime::Value::Nodes(children) => {
                println!("{}  {}: [{}]", indent, name, children.len());
                for child in children {
                    print_node(child, depth + 2);
                }
            }
            runtime::Value::String(s) => println!("{}  {} = {:?}", indent, name, s),
            runtime::Value::Int(n) => println!("{}  {} = {}", indent, name, n),
            _ => println!("{}  {} = {:?}", indent, name, val),
        }
    }
}
