use crate::lexer::token::{Delimiter, Keyword, Token, TokenKind};

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _src: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _src: src,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                _ => break,
            }
        }
    }

    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
    }
    fn is_ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || (c as u32) >= 0x80
    }

    fn read_identifier_or_keyword(&mut self) -> (String, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        if let Some(c) = self.peek()
            && Self::is_ident_start(c)
        {
            s.push(self.bump().unwrap());
        }
        while let Some(c) = self.peek() {
            if Self::is_ident_continue(c) {
                s.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        (s, start_line, start_col)
    }

    fn read_number(&mut self) -> (String, bool, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        if let Some('.') = self.peek()
            && let Some(nxt) = self.peek_next()
            && nxt.is_ascii_digit()
        {
            is_float = true;
            s.push(self.bump().unwrap());
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    s.push(self.bump().unwrap());
                } else {
                    break;
                }
            }
        }
        (s, is_float, start_line, start_col)
    }

    fn read_char_literal(&mut self) -> (String, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        // consume opening '
        self.bump(); // consume '\''
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    s.push(self.bump().unwrap());
                    if let Some(_n) = self.peek() {
                        s.push(self.bump().unwrap());
                    }
                }
                '\'' => {
                    self.bump();
                    break;
                }
                _ => {
                    s.push(self.bump().unwrap());
                }
            }
        }
        (s, start_line, start_col)
    }

    fn read_string_literal(&mut self) -> (String, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        self.bump(); // consume '"'
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    s.push(self.bump().unwrap());
                    if let Some(_n) = self.peek() {
                        s.push(self.bump().unwrap());
                    }
                }
                '"' => {
                    self.bump();
                    break;
                }
                _ => {
                    s.push(self.bump().unwrap());
                }
            }
        }
        (s, start_line, start_col)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let line = self.line;
        let col = self.col;
        match self.peek() {
            None => Token::new(TokenKind::EOF, line, col),
            Some(c) if Self::is_ident_start(c) => {
                let (s, l, ccol) = self.read_identifier_or_keyword();
                let kind = match s.as_str() {
                    "fn" => TokenKind::Keyword(Keyword::Fn),
                    "return" => TokenKind::Keyword(Keyword::Return),
                    "for" => TokenKind::Keyword(Keyword::For),
                    "if" => TokenKind::Keyword(Keyword::If),
                    "while" => TokenKind::Keyword(Keyword::While),
                    "int" => TokenKind::Keyword(Keyword::Int),
                    "float" => TokenKind::Keyword(Keyword::Float),
                    "char" => TokenKind::Keyword(Keyword::Char),
                    "mut" => TokenKind::Keyword(Keyword::Mut),
                    "shared" => TokenKind::Keyword(Keyword::Shared),
                    _ => TokenKind::Identifier(s),
                };
                Token::new(kind, l, ccol)
            }
            Some(c) if c.is_ascii_digit() => {
                let (s, is_float, l, ccol) = self.read_number();
                let kind = if is_float {
                    TokenKind::FloatLiteral(s)
                } else {
                    TokenKind::IntLiteral(s)
                };
                Token::new(kind, l, ccol)
            }
            Some('\'') => {
                let (s, l, ccol) = self.read_char_literal();
                Token::new(TokenKind::CharLiteral(s), l, ccol)
            }
            Some('"') => {
                let (s, l, ccol) = self.read_string_literal();
                Token::new(TokenKind::StringLiteral(s), l, ccol)
            }
            Some(c) => {
                // 单字符分隔符
                let single_delim = match c {
                    '(' => Some(Delimiter::LParen),
                    ')' => Some(Delimiter::RParen),
                    '{' => Some(Delimiter::LBrace),
                    '}' => Some(Delimiter::RBrace),
                    '[' => Some(Delimiter::LBracket),
                    ']' => Some(Delimiter::RBracket),
                    ',' => Some(Delimiter::Comma),
                    ';' => Some(Delimiter::Semicolon),
                    _ => None,
                };
                if let Some(d) = single_delim {
                    let l = self.line;
                    let ccol = self.col;
                    self.bump();
                    return Token::new(TokenKind::Delimiter(d), l, ccol);
                }

                // 双字符 token（分隔符和运算符）
                let two_delim = match (self.peek(), self.peek_next()) {
                    (Some('-'), Some('>')) => Some(Delimiter::Arrow),
                    _ => None,
                };
                if let Some(d) = two_delim {
                    let l = self.line;
                    let ccol = self.col;
                    self.bump();
                    self.bump();
                    return Token::new(TokenKind::Delimiter(d), l, ccol);
                }

                let two_op = match (self.peek(), self.peek_next()) {
                    (Some('='), Some('=')) => Some("==".to_string()),
                    (Some('!'), Some('=')) => Some("!=".to_string()),
                    (Some('<'), Some('=')) => Some("<=".to_string()),
                    (Some('>'), Some('=')) => Some(">=".to_string()),
                    (Some('&'), Some('&')) => Some("&&".to_string()),
                    (Some('|'), Some('|')) => Some("||".to_string()),
                    _ => None,
                };
                if let Some(op) = two_op {
                    let l = self.line;
                    let ccol = self.col;
                    self.bump();
                    self.bump();
                    return Token::new(TokenKind::Operator(op), l, ccol);
                }

                let l = self.line;
                let ccol = self.col;
                let ch = self.bump().unwrap();
                Token::new(TokenKind::Operator(ch.to_string()), l, ccol)
            }
        }
    }
}

impl<'a> Lexer<'a> {
    /// 收集并返回直到 EOF 的所有 token（包含 EOF）
    pub fn tokenize_all(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        loop {
            let t = self.next_token();
            let is_eof = matches!(t.kind, TokenKind::EOF);
            out.push(t);
            if is_eof { break; }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenKind;

    #[test]
    fn smoke() {
        let src = r#"fn f()->int { return 123; }"#;
        let mut l = Lexer::new(src);
        let mut found = Vec::new();
        loop {
            let t = l.next_token();
            match t.kind {
                TokenKind::EOF => break,
                _ => found.push(t),
            }
        }
        assert!(found.len() > 0);
    }
}
