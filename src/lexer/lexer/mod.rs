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
    /// #179：跳过的注释（供格式化保留）
    comments: Vec<crate::lexer::Comment>,
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
            comments: Vec::new(),
        }
    }

    /// #179：本次词法收集到的注释（按出现顺序）。
    pub fn comments(&self) -> &[crate::lexer::Comment] {
        &self.comments
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
                    let (l, c) = (self.line, self.col);
                    self.bump(); self.bump();
                    let mut text = String::from("//");
                    while let Some(ch) = self.peek() {
                        if ch == '\n' { break; }
                        text.push(ch);
                        self.bump();
                    }
                    self.comments.push(crate::lexer::Comment {
                        text,
                        span: crate::span::Span::new(l, c, self.line, self.col, 0, 0),
                    });
                }
                Some('/') if self.peek_next() == Some('*') => {
                    let (l, c) = (self.line, self.col);
                    self.bump(); self.bump();
                    let mut text = String::from("/*");
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_next() == Some('/') {
                            text.push_str("*/");
                            self.bump(); self.bump();
                            break;
                        }
                        text.push(ch);
                        self.bump();
                    }
                    self.comments.push(crate::lexer::Comment {
                        text,
                        span: crate::span::Span::new(l, c, self.line, self.col, 0, 0),
                    });
                }
                _ => break,
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

mod next;
mod reading;

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
