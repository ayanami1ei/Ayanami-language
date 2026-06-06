#[cfg(test)]
pub mod test {
    use crate::lexer::Lexer;
    use crate::lexer::TokenKind;

    #[test]
    fn test() {
        let code = include_str!("../../example/test.aya");
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize_all();
        for t in tokens.iter().filter(|t| !matches!(t.kind, TokenKind::EOF)) {
            println!("{}", t);
        }
    }
}
