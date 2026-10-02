use super::*;

impl<'a> Lexer<'a> {
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
                    "null" => TokenKind::Keyword(Keyword::Null),
                    "pub" => TokenKind::Keyword(Keyword::Pub),
                    "crate" => TokenKind::Keyword(Keyword::Crate),
                    "inline" => TokenKind::Keyword(Keyword::Inline),
                    "extern" => TokenKind::Keyword(Keyword::Extern),
                    "asm" => TokenKind::Keyword(Keyword::Asm),
                    "in" => TokenKind::Keyword(Keyword::In),
                    "out" => TokenKind::Keyword(Keyword::Out),
                    "reg" => TokenKind::Keyword(Keyword::Reg),
                    "clobbers" => TokenKind::Keyword(Keyword::Clobbers),
                    "import" => TokenKind::Keyword(Keyword::Import),
                    "fn" => TokenKind::Keyword(Keyword::Fn),
                    "return" => TokenKind::Keyword(Keyword::Return),
                    "for" => TokenKind::Keyword(Keyword::For),
                    "if" => TokenKind::Keyword(Keyword::If),
                    "elif" => TokenKind::Keyword(Keyword::Elif),
                    "else" => TokenKind::Keyword(Keyword::Else),
                    "while" => TokenKind::Keyword(Keyword::While),
                    "break" => TokenKind::Keyword(Keyword::Break),
                    "continue" => TokenKind::Keyword(Keyword::Continue),
                    "int" => TokenKind::Keyword(Keyword::Int),
                    "float" => TokenKind::Keyword(Keyword::Float),
                    "char" => TokenKind::Keyword(Keyword::Char),
                    "bool" => TokenKind::Keyword(Keyword::Bool),
                    "ref" => TokenKind::Keyword(Keyword::Ref),
                    "true" => TokenKind::Keyword(Keyword::True),
                    "false" => TokenKind::Keyword(Keyword::False),
                    "mut" => TokenKind::Keyword(Keyword::Mut),
                    "shared" => TokenKind::Keyword(Keyword::Shared),
                    "unique" => TokenKind::Keyword(Keyword::Unique),
                    "weak" => TokenKind::Keyword(Keyword::Weak),
                    "struct" => TokenKind::Keyword(Keyword::Struct),
                    "enum" => TokenKind::Keyword(Keyword::Enum),
                    "match" => TokenKind::Keyword(Keyword::Match),
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
                    ':' if self.peek_next() != Some(':') => Some(Delimiter::Colon),
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
                    (Some('='), Some('>')) => Some(Delimiter::FatArrow),
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
                    (Some(':'), Some(':')) => Some("::".to_string()),
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
