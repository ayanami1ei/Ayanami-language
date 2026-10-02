// @generated
#[allow(unused)]

pub const KEYWORDS: &[&str] = &["fn", "return", "if", "else", "true", "false", "while", "for", "in", "match", "enum", "struct", "interface", "impl", "pub", "unique", "ref", "mut", "extern", "import", "as", "asm", "break", "continue", "self", "move", "clone", "inline", "let"];

pub fn tokenize(input: &str) -> Vec<asuka::runtime::Token> {
    let mut lex = asuka::runtime::Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        lex.skip_ws();
        if lex.pos >= lex.chars.len() { break; }
        let c = lex.chars[lex.pos];
        match c {
            '"' => tokens.push(lex.read_string()),
            c if c.is_ascii_digit() => tokens.push(lex.read_number()),
            c if c.is_alphabetic() || c == '_' => tokens.push(lex.read_ident(KEYWORDS)),
            '!' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("!=", "!=")); }
                else { tokens.push(lex.read_fixed("!", "!")); }
            }
            '%' => tokens.push(lex.read_fixed("%", "%")),
            '&' => tokens.push(lex.read_fixed("&&", "&&")),
            '(' => tokens.push(lex.read_fixed("(", "(")),
            ')' => tokens.push(lex.read_fixed(")", ")")),
            '*' => tokens.push(lex.read_fixed("*", "*")),
            '+' => tokens.push(lex.read_fixed("+", "+")),
            ',' => tokens.push(lex.read_fixed(",", ",")),
            '-' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='>' { tokens.push(lex.read_fixed("->", "->")); }
                else { tokens.push(lex.read_fixed("-", "-")); }
            }
            '.' => tokens.push(lex.read_fixed(".", ".")),
            '/' => tokens.push(lex.read_fixed("/", "/")),
            ':' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]==':' { tokens.push(lex.read_fixed("::", "::")); }
                else { tokens.push(lex.read_fixed(":", ":")); }
            }
            ';' => tokens.push(lex.read_fixed(";", ";")),
            '<' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("<=", "<=")); }
                else { tokens.push(lex.read_fixed("<", "<")); }
            }
            '=' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("==", "==")); }
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='>' { tokens.push(lex.read_fixed("=>", "=>")); }
                else { tokens.push(lex.read_fixed("=", "=")); }
            }
            '>' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed(">=", ">=")); }
                else { tokens.push(lex.read_fixed(">", ">")); }
            }
            '[' => tokens.push(lex.read_fixed("[", "[")),
            ']' => tokens.push(lex.read_fixed("]", "]")),
            '{' => tokens.push(lex.read_fixed("{", "{")),
            '|' => tokens.push(lex.read_fixed("||", "||")),
            '}' => tokens.push(lex.read_fixed("}", "}")),
            _ => panic!("unexpected '{}'", c),
        }
    }
    tokens.push(asuka::runtime::Token { kind: "EOF".into(), span: asuka::runtime::Span::new(), value: String::new() });
    tokens
}

pub struct Parser(pub asuka::runtime::Parser);

mod parser_01;
mod parser_02;
mod parser_03;
mod parser_04;
mod parser_05;
mod parser_06;
mod parser_07;
mod parser_08;
mod parser_09;
