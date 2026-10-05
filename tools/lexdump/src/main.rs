//! lexdump — 用 rust 版词法器输出 token 流（selfhost 的 golden 生成器）
//! 用法：cargo run -q --manifest-path tools/lexdump/Cargo.toml -- FILE.aya
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let src = std::fs::read_to_string(path).expect("read file");
    let mut lexer = ayanami::lexer::Lexer::new(&src);
    for t in lexer.tokenize_all() {
        println!("{}", t);
    }
}
