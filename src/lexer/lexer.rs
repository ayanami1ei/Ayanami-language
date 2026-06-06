use crate::lexer::delimiter::Delimiter;
use crate::lexer::keyword::Keyword;
use crate::lexer::token::Token;
use crate::lexer::token_kind::TokenKind;

pub struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    byte_offset: usize,
    _src: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            byte_offset: 0,
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
        self.byte_offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    self.bump(); self.bump();
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        self.bump();
                    }
                }
                Some('/') if self.peek_next() == Some('*') => {
                    self.bump(); self.bump();
                    while let Some(c) = self.peek() {
                        if c == '*' && self.peek_next() == Some('/') {
                            self.bump(); self.bump();
                            break;
                        }
                        self.bump();
                    }
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

    fn read_identifier_or_keyword(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
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
        (s, start_line, start_col, start_byte)
    }

    fn read_number(&mut self) -> (String, bool, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
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
        (s, is_float, start_line, start_col, start_byte)
    }

    fn read_char_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        // consume opening '
        self.bump();
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
        (s, start_line, start_col, start_byte)
    }

    fn read_string_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
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
        (s, start_line, start_col, start_byte)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        let line = self.line;
        let col = self.col;
        match self.peek() {
            None => Token::new(
                TokenKind::EOF,
                line,
                col,
                self.byte_offset,
                self.byte_offset,
            ),
            Some(c) if Self::is_ident_start(c) => {
                let (s, l, ccol, sbyte) = self.read_identifier_or_keyword();
                let kind = match s.as_str() {
                    "pub" => TokenKind::Keyword(Keyword::Pub),
                    "crate" => TokenKind::Keyword(Keyword::Crate),
                    "import" => TokenKind::Keyword(Keyword::Import),
                    "fn" => TokenKind::Keyword(Keyword::Fn),
                    "return" => TokenKind::Keyword(Keyword::Return),
                    "for" => TokenKind::Keyword(Keyword::For),
                    "if" => TokenKind::Keyword(Keyword::If),
                    "elif" => TokenKind::Keyword(Keyword::Elif),
                    "else" => TokenKind::Keyword(Keyword::Else),
                    "in" => TokenKind::Keyword(Keyword::In),
                    "while" => TokenKind::Keyword(Keyword::While),
                    "int" => TokenKind::Keyword(Keyword::Int),
                    "float" => TokenKind::Keyword(Keyword::Float),
                    "char" => TokenKind::Keyword(Keyword::Char),
                    "bool" => TokenKind::Keyword(Keyword::Bool),
                    "true" => TokenKind::Keyword(Keyword::True),
                    "false" => TokenKind::Keyword(Keyword::False),
                    "mut" => TokenKind::Keyword(Keyword::Mut),
                    "shared" => TokenKind::Keyword(Keyword::Shared),
                    "unique" => TokenKind::Keyword(Keyword::Unique),
                    "weak" => TokenKind::Keyword(Keyword::Weak),
                    "struct" => TokenKind::Keyword(Keyword::Struct),
                    "namespace" => TokenKind::Keyword(Keyword::Namespace),
                    "move" => TokenKind::Keyword(Keyword::Move),
                    "clone" => TokenKind::Keyword(Keyword::Clone),
                    "interface" => TokenKind::Keyword(Keyword::Interface),
                    "impl" => TokenKind::Keyword(Keyword::Impl),
                    "self" => TokenKind::Keyword(Keyword::Self_),
                    _ => TokenKind::Identifier(s),
                };
                Token::new(kind, l, ccol, sbyte, self.byte_offset)
            }
            Some(c) if c.is_ascii_digit() => {
                let (s, is_float, l, ccol, sbyte) = self.read_number();
                let kind = if is_float {
                    TokenKind::FloatLiteral(s)
                } else {
                    TokenKind::IntLiteral(s)
                };
                Token::new(kind, l, ccol, sbyte, self.byte_offset)
            }
            Some('\'') => {
                let (s, l, ccol, sbyte) = self.read_char_literal();
                Token::new(
                    TokenKind::CharLiteral(s),
                    l,
                    ccol,
                    sbyte,
                    self.byte_offset,
                )
            }
            Some('"') => {
                let (s, l, ccol, sbyte) = self.read_string_literal();
                Token::new(
                    TokenKind::StringLiteral(s),
                    l,
                    ccol,
                    sbyte,
                    self.byte_offset,
                )
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
                    '.' => Some(Delimiter::Dot),
                    _ => None,
                };
                if let Some(d) = single_delim {
                    let l = self.line;
                    let ccol = self.col;
                    let sbyte = self.byte_offset;
                    self.bump();
                    return Token::new(
                        TokenKind::Delimiter(d),
                        l,
                        ccol,
                        sbyte,
                        self.byte_offset,
                    );
                }

                // 双字符 token（分隔符和运算符）
                let two_delim = match (self.peek(), self.peek_next()) {
                    (Some('-'), Some('>')) => Some(Delimiter::Arrow),
                    _ => None,
                };
                if let Some(d) = two_delim {
                    let l = self.line;
                    let ccol = self.col;
                    let sbyte = self.byte_offset;
                    self.bump();
                    self.bump();
                    return Token::new(
                        TokenKind::Delimiter(d),
                        l,
                        ccol,
                        sbyte,
                        self.byte_offset,
                    );
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
                    let sbyte = self.byte_offset;
                    self.bump();
                    self.bump();
                    return Token::new(
                        TokenKind::Operator(op),
                        l,
                        ccol,
                        sbyte,
                        self.byte_offset,
                    );
                }

                let l = self.line;
                let ccol = self.col;
                let sbyte = self.byte_offset;
                let ch = self.bump().unwrap();
                Token::new(
                    TokenKind::Operator(ch.to_string()),
                    l,
                    ccol,
                    sbyte,
                    self.byte_offset,
                )
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
            if is_eof {
                break;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token_kind::TokenKind;

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
