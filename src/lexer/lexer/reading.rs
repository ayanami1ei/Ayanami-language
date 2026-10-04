use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
    }
    pub(super) fn is_ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || (c as u32) >= 0x80
    }

    pub(super) fn read_identifier_or_keyword(&mut self) -> (String, usize, usize, usize) {
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

    pub(super) fn read_number(&mut self) -> (String, bool, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        let mut s = String::new();
        let mut is_float = false;
        // 进制前缀：0x / 0b / 0o（M1.5）
        let radix = if self.peek() == Some('0')
            && matches!(self.peek_next(), Some('x' | 'X' | 'b' | 'B' | 'o' | 'O'))
        {
            s.push(self.bump().unwrap());
            let p = self.bump().unwrap();
            let r = match p { 'x' | 'X' => 16, 'b' | 'B' => 2, _ => 8 };
            s.push(p);
            r
        } else {
            10
        };
        let digit_ok = |c: char| match radix {
            16 => c.is_ascii_hexdigit(),
            2 => c == '0' || c == '1',
            8 => ('0'..='7').contains(&c),
            _ => c.is_ascii_digit(),
        };
        while let Some(c) = self.peek() {
            if digit_ok(c) || c == '_' { s.push(self.bump().unwrap()); } else { break; }
        }
        if radix == 10 {
            // 小数部分
            if self.peek() == Some('.')
                && self.peek_next().map(|c| c.is_ascii_digit() || c == '_').unwrap_or(false)
            {
                is_float = true;
                s.push(self.bump().unwrap());
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '_' { s.push(self.bump().unwrap()); } else { break; }
                }
            }
            // 指数部分 e[+-]digits
            if matches!(self.peek(), Some('e' | 'E')) {
                let mut j = self.pos + 1;
                if matches!(self.chars.get(j), Some('+' | '-')) { j += 1; }
                if self.chars.get(j).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    is_float = true;
                    s.push(self.bump().unwrap());
                    if matches!(self.peek(), Some('+' | '-')) { s.push(self.bump().unwrap()); }
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() || c == '_' { s.push(self.bump().unwrap()); } else { break; }
                    }
                }
            }
        }
        // 后缀：u8 / i32 / f64 / usize 等
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' { s.push(self.bump().unwrap()); } else { break; }
        }
        (s, is_float, start_line, start_col, start_byte)
    }

    /// 解码一个转义序列（已消费 `\\`），返回对应字符
    fn read_escape(&mut self) -> Option<char> {
        let c = self.bump()?;
        Some(match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'x' => {
                let h1 = self.bump().unwrap_or('0');
                let h2 = self.bump().unwrap_or('0');
                let hex = format!("{}{}", h1, h2);
                u8::from_str_radix(&hex, 16).map(|b| b as char).unwrap_or('?')
            }
            other => other,
        })
    }

    pub(super) fn read_char_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        // consume opening '
        self.bump();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    self.bump();
                    if let Some(ch) = self.read_escape() {
                        s.push(ch);
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

    pub(super) fn read_string_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        self.bump(); // consume '"'
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    self.bump();
                    if let Some(ch) = self.read_escape() {
                        s.push(ch);
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
}
